use crate::utils::ai_client::{AIClient, ChatMessage};
use std::sync::Arc;

pub struct AIIntegration {
    ai_client: Arc<AIClient>,
}

impl AIIntegration {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let ai_client = Arc::new(AIClient::new()?);
        Ok(Self { ai_client })
    }

    pub async fn process_query_with_ai(&self, query: &str) -> Result<String, Box<dyn std::error::Error>> {
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: query.to_string(),
            }
        ];

        let response = self.ai_client.chat_completion(messages).await?;

        if let Some(choice) = response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err("No response from AI".into())
        }
    }

    // 提供公共访问AI客户端的方法
    pub fn get_ai_client(&self) -> &AIClient {
        &self.ai_client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_integration() {
        // 注意：这个测试需要AI服务实际可用才能运行成功
        let ai_integration = AIIntegration::new();

        if let Ok(ai_int) = ai_integration {
            let result = ai_int.process_query_with_ai("你好").await;
            match result {
                Ok(response) => println!("AI Response: {}", response),
                Err(e) => println!("Error calling AI: {}", e),
            }
        } else {
            println!("Failed to create AI integration");
        }
    }
}