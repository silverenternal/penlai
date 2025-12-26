use penlai::context::llm_context::ContextManager;
use penlai::utils::ai_client::ChatMessage;
use penlai::utils::ai_integration::AIIntegration;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化上下文管理器
    let context_manager = Arc::new(ContextManager::new(100, 3600));

    // 初始化AI集成
    let ai_integration = Arc::new(AIIntegration::new()?);

    // 创建示例上下文
    let medical_context = context_manager
        .create_context(
            "session_medical_001".to_string(),
            "user_medical_001".to_string(),
            "medical".to_string(),
            "Medical guidelines for treating respiratory infections, including pneumonia, bronchitis, and other lung conditions. Treatment typically involves antibiotics, rest, and supportive care.".to_string(),
            9,
        )
        .await
        .map_err(|e| format!("Failed to create context: {}", e))?;

    println!("Created context: {}", medical_context.id);

    // 使用AI客户端处理查询
    let query = "根据上下文，肺炎的治疗方法是什么？";

    // 获取相关上下文
    let session_contexts = context_manager.get_session_contexts("session_medical_001").await;
    println!("Found {} contexts for session", session_contexts.len());

    // 构建AI消息，包含上下文信息
    let mut messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "你是一个专业助手，使用提供的上下文信息回答用户问题。".to_string(),
        }
    ];

    // 添加上下文信息
    for ctx in session_contexts {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: format!("上下文信息: {}", ctx.context_data),
        });
    }

    // 添加用户查询
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: query.to_string(),
    });

    // 调用AI
    let response = ai_integration.get_ai_client().chat_completion(messages).await?;

    if let Some(choice) = response.choices.first() {
        println!("AI Response: {}", choice.message.content);
        println!("Tokens used: {} prompt, {} completion, {} total",
                 response.usage.prompt_tokens,
                 response.usage.completion_tokens,
                 response.usage.total_tokens);
    }

    Ok(())
}