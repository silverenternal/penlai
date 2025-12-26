use penlai::utils::ai_client::{AIClient, ChatMessage};
use penlai::utils::ai_integration::AIIntegration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化AI集成
    let ai_integration = AIIntegration::new()?;
    
    // 示例：向AI提问
    let response = ai_integration.process_query_with_ai("你是什么模型？").await?;
    println!("AI Response: {}", response);
    
    // 直接使用AI客户端
    let ai_client = AIClient::new()?;
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "你能做什么？".to_string(),
        }
    ];
    
    let response = ai_client.chat_completion(messages).await?;
    if let Some(choice) = response.choices.first() {
        println!("Direct AI Response: {}", choice.message.content);
    }
    
    Ok(())
}