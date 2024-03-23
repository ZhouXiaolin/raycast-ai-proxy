use std::{borrow::Cow, io::Write, iter::once};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PromptMessage {
    pub role: Role,
    pub content: String,
}


#[derive(Debug, Deserialize)]
pub struct ChatCompletion {
    choices: Vec<Choice>,
    created: u64,
    id: String,
    model: String,
    object: String,
}

impl ChatCompletion {
    pub fn get_content(&self) -> Cow<'_, str> {
        if let Some(content) = self.choices[0].delta.content.as_ref() {
            Cow::Borrowed(content)
        } else {
            Cow::Borrowed("")
        }
    }
}

#[derive(Debug, Deserialize)]
struct Choice {
    delta: Delta,
    finish_reason: Option<String>,
    index: u32,
}

#[derive(Debug, Deserialize)]
struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChatHistory {
    system_prompt: PromptMessage,
    messages: Vec<PromptMessage>,
}

impl ChatHistory {
    pub fn new(system_prompt: &str) -> Self {
        Self {
            system_prompt: PromptMessage {
                role: Role::System,
                content: system_prompt.to_string(),
            },
            messages: vec![],
        }
    }
    pub fn get_messages(&self) -> Vec<PromptMessage> {
        self.messages.clone()
    }

    pub fn generate_messages(&self, prompt: &str) -> Vec<PromptMessage> {
        let prompt = PromptMessage {
            role: Role::User,
            content: prompt.to_string(),
        };

        once(self.system_prompt.clone())
            .chain(self.get_messages())
            .chain(once(prompt))
            .collect()
    }

    pub fn append_user_prompt(&mut self, prompt: &str) {
        let prompt = PromptMessage {
            role: Role::User,
            content: prompt.to_string(),
        };
        self.messages.push(prompt);
        if self.messages.len() > 10 {
            self.messages.remove(0);
        }
    }
    pub fn append_assistant_prompt(&mut self, prompt: &str) {
        let prompt = PromptMessage {
            role: Role::Assistant,
            content: prompt.to_string(),
        };
        self.messages.push(prompt);
        if self.messages.len() > 10 {
            self.messages.remove(0);
        }
    }
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}
