use async_trait::async_trait;

use crate::clients::types::PromptMessage;

#[async_trait]
pub trait AsyncChatChunkIter: Send + Sync {
    async fn next(&mut self) -> Option<String>;
}

#[async_trait]
pub trait AsyncChatClient {
    async fn chat(
        &self,
        messages: Vec<PromptMessage>,
        seed: u32,
    ) -> Option<Box<dyn AsyncChatChunkIter>>;
}
