use reqwest;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f64,
    pub max_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: u64,
    pub model: String,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct AIClient {
    client: reqwest::Client,
    base_url: String,
    model: String,
    temperature: f64,
    max_tokens: u32,
}

impl AIClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 从环境变量加载配置
        dotenv::dotenv().ok(); // 加载.env文件
        
        let base_url = env::var("AI_BASE_URL")
            .unwrap_or_else(|_| "http://103.203.140.12:7578/v1".to_string());
        let model = env::var("AI_MODEL")
            .unwrap_or_else(|_| "qwen3-8b-union".to_string());
        let temperature = env::var("AI_TEMPERATURE")
            .unwrap_or_else(|_| "0.7".to_string())
            .parse::<f64>()
            .unwrap_or(0.7);
        let max_tokens = env::var("AI_MAX_TOKENS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .unwrap_or(100);

        Ok(Self {
            client: reqwest::Client::new(),
            base_url,
            model,
            temperature,
            max_tokens,
        })
    }

    pub async fn chat_completion(&self, messages: Vec<ChatMessage>) -> Result<ChatCompletionResponse, reqwest::Error> {
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        let url = format!("{}/chat/completions", self.base_url);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let completion_response: ChatCompletionResponse = response.json().await?;
        Ok(completion_response)
    }
}