mod clients;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::sse::Event;
use axum::response::{IntoResponse, Response, Sse};
use axum::routing::{get, post};
use axum::Json;
use axum::{extract::Request, Router};
use axum_server::tls_rustls::RustlsConfig;
use clients::*;
use futures::{Stream};
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::iter::once;
use std::net::SocketAddr;
use tracing::info;
const SYSTEM_PROMPT:&str = "So you're a mathematician, and you like to communicate with people using mathematical formulas.";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cert = include_bytes!("../cert/backend.raycast.com.cert.pem");
    let key = include_bytes!("../cert/backend.raycast.com.key.pem");
    let config = RustlsConfig::from_pem(cert.to_vec(), key.to_vec())
        .await
        .unwrap();

    let client = clients::ChatMLClient::builder().build();
    let shared_state = SharedState {
        client,
    };
    //

    let app = Router::new()
        .route("/api/v1/me", get(handle_me))
        .route("/api/v1/me/sync", get(handle_me_sync))
        .route("/api/v1/ai/models", get(handle_ai_models))
        .route(
            "/api/v1/ai/chat_completions",
            post(handle_ai_chat_completions).with_state(shared_state),
        )
        .route("/api/v1/translations", post(handle_translations));

    let addr = SocketAddr::from(([127, 0, 0, 1], 443));
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug, Clone)]
struct SharedState {
    client: ChatMLClient,
}

#[derive(Debug, Serialize, Deserialize)]
struct PostData {
    debug: bool,
    image_generation_tool: bool,
    locale: String,
    messages: Vec<Message>,
    model: String,
    provider: String,
    source: String,
    system_instruction: String,
    temperature: f32,
    web_search_tool: bool,
}

impl PostData {
    fn get_messages(&self) -> Vec<PromptMessage> {
        let mut messages = vec![];

        for message in &self.messages {
            if message.author == "user" {
                messages.push(PromptMessage {
                    role: clients::Role::User,
                    content: message.content.text.clone(),
                });
            }

            if message.author == "assistant" {
                messages.push(PromptMessage {
                    role: clients::Role::Assistant,
                    content: message.content.text.clone(),
                });
            }
        }

        messages
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    author: String,
    content: Content,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    text: String,
}

async fn handle_ai_chat_completions(
    State(state): State<SharedState>,
    Json(payload): Json<PostData>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!(?payload, "handle_ai_chat_completions==>");
    let messages = payload.get_messages();
    let mut rng = SmallRng::from_entropy();
    let seed = rng.gen::<u32>();

    let system_prompt = PromptMessage {
        role: Role::System,
        content: SYSTEM_PROMPT.to_string(),
    };
    let messages = once(system_prompt).chain(messages).collect();
    info!(?messages, "===>");

    let mut iter = state.client.chat(messages, seed).await.unwrap();

    let stream = async_stream::stream! {
        while let Some(chunk) = iter.next().await {
            let data = json!({
                "text": chunk
            });
            yield Ok(Event::default().data(data.to_string()));
        }
    };

    Sse::new(stream)
}

async fn handle_translations(Json(payload): Json<Value>) -> impl IntoResponse {
    info!(?payload, "===>");
    "hello".to_string()
}

async fn handle_ai_models(headers: HeaderMap, request: Request) -> impl IntoResponse {
    let models = json!({
        "default_models":{
            "chat": "openai-gpt-3.5-turbo",
            "quick_ai": "openai-gpt-3.5-turbo",
            "commands": "openai-gpt-3.5-turbo",
            "api": "openai-gpt-3.5-turbo",
        },
        "models": [
            {
                "id": "openai-gpt-3.5-turbo",
                "model": "moonshot-v1-8k",
                "name": "GPT-3.5 Turbo",
                "provider": "openai",
                "provider_name": "OpenAI",
                "requires_better_ai": true,
                "features": [
                    "chat",
                    "quick_ai",
                    "commands",
                    "api",
                ],
            },
        ]
    });
    models.to_string()
}

async fn handle_me_sync(headers: HeaderMap, request: Request) -> impl IntoResponse {
    info!(?request, "handle_me_sync==>");
    "".to_string()
}
async fn handle_me(headers: HeaderMap, request: Request) -> impl IntoResponse {
    let url = "https://backend.raycast.com/api/v1/me";
    let mut header_map = headers.clone();
    header_map.insert("host", "backend.raycast.com".parse().unwrap());
    header_map.insert("accept-encoding", "identify".parse().unwrap());
    let addr = SocketAddr::from(([18, 238, 192, 112], 8081));
    let client = reqwest::Client::builder()
        .resolve("backend.raycast.com", addr)
        .build()
        .unwrap();
    let response = client.get(url).headers(header_map).send().await.unwrap();
    let text = response.text().await.unwrap();

    let mut data: Value = serde_json::from_str(&text).unwrap();
    data["eligible_for_pro_features"] = Value::Bool(true);
    data["has_active_subscription"] = Value::Bool(true);
    data["eligible_for_ai"] = Value::Bool(true);
    data["eligible_for_gpt4"] = Value::Bool(true);
    data["eligible_for_ai_citations"] = Value::Bool(true);
    data["eligible_for_developer_hub"] = Value::Bool(true);
    data["eligible_for_application_settings"] = Value::Bool(true);
    data["publishing_bot"] = Value::Bool(true);
    data["has_pro_features"] = Value::Bool(true);
    data["has_better_ai"] = Value::Bool(true);
    data["can_upgrade_to_pro"] = Value::Bool(false);
    data["admin"] = Value::Bool(true);
    data.to_string()
}
