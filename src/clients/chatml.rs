use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex_lite::Regex;
use reqwest::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Response,
};
use serde_json::json;
use tracing::info;
use typed_builder::TypedBuilder;

use super::{
    traits::{AsyncChatChunkIter, AsyncChatClient},
    types::{ChatCompletion, PromptMessage},
};

#[derive(Debug, TypedBuilder, Clone)]
pub struct ChatMLClient {
    #[builder(default = "http://localhost:8080/v1/chat/completions".to_string())]
    server_url: String,
    #[builder(default = "no-key".to_string())]
    secret_key: String,
    #[builder(default = "gpt-3.5-turbo".to_string())]
    model: String,
}

#[async_trait]
impl AsyncChatClient for ChatMLClient {
    async fn chat(
        &self,
        messages: Vec<PromptMessage>,
        seed: u32,
    ) -> Option<Box<dyn AsyncChatChunkIter>> {
        let client = reqwest::Client::default();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", &self.secret_key).parse().unwrap(),
        );
        let json = json!({
            "model": self.model,
            "messages": messages,
            "temperature":0.8,
            "seed":seed,
            "stream":true,
            "cache_prompt":true,
        });

        let builder = client.post(&self.server_url).headers(headers);
        if let Ok(res) = builder.json(&json).send().await {
            Some(Box::new(ChatMLIterator { res }))
        } else {
            None
        }
    }
}

// chunk

pub struct ChatMLIterator {
    res: Response,
}

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"data:\s*(.*)").unwrap());


#[async_trait::async_trait]
impl AsyncChatChunkIter for ChatMLIterator {
    async fn next(&mut self) -> Option<String> {
        if let Some(bytes) = self.res.chunk().await.unwrap() {
            let response = String::from_utf8_lossy(&bytes);
            let mut chunk = "".to_string();
            for capture in REGEX.captures_iter(&response) {
                let data = capture.get(1).unwrap().as_str();
                if let Ok(content) = serde_json::from_str::<ChatCompletion>(data) {
                    chunk.push_str(&content.get_content());
                }
            }
            Some(chunk)
        } else {
            None
        }
    }
}
