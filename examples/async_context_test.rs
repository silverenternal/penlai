use penlai::context::llm_context::ContextManager;
use penlai::selection::async_context_selector::ContextSelector;
use penlai::utils::ai_client::ChatMessage;
use penlai::utils::ai_integration::AIIntegration;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化上下文管理器
    let context_manager = Arc::new(ContextManager::new(100, 3600)); // 100并发，1小时TTL

    // 初始化上下文选择器
    let context_selector = Arc::new(ContextSelector::new(context_manager.clone()));

    // 初始化AI集成
    let ai_integration = Arc::new(AIIntegration::new()?);

    // 创建一个会话ID用于测试
    let session_id = "test_session_001".to_string();
    let user_id = "test_user_001".to_string();

    // 1. 创建医疗领域上下文
    println!("1. 创建医疗领域上下文...");
    let medical_context = context_manager
        .create_context(
            session_id.clone(),
            user_id.clone(),
            "medical".to_string(),
            "Medical guidelines for treating respiratory infections, including pneumonia, bronchitis, and other lung conditions. Treatment typically involves antibiotics, rest, and supportive care.".to_string(),
            9,
        )
        .await
        .map_err(|e| format!("Failed to create medical context: {}", e))?;

    println!("   Created medical context: {}", medical_context.id);

    // 2. 创建技术领域上下文
    println!("2. 创建技术领域上下文...");
    let tech_context = context_manager
        .create_context(
            session_id.clone(),
            user_id.clone(),
            "technical".to_string(),
            "Rust async programming patterns, including async/await, futures, and concurrent processing. Best practices for building high-performance async applications.".to_string(),
            8,
        )
        .await
        .map_err(|e| format!("Failed to create tech context: {}", e))?;

    println!("   Created tech context: {}", tech_context.id);

    // 3. 创建法律领域上下文
    println!("3. 创建法律领域上下文...");
    let legal_context = context_manager
        .create_context(
            session_id.clone(),
            user_id.clone(),
            "legal".to_string(),
            "Contract law fundamentals, including offer, acceptance, consideration, and legal capacity. Requirements for valid contracts and common legal issues.".to_string(),
            7,
        )
        .await
        .map_err(|e| format!("Failed to create legal context: {}", e))?;

    println!("   Created legal context: {}", legal_context.id);

    // 4. 测试医疗相关问题
    println!("\n4. 测试医疗相关问题...");
    let medical_query = "What is the treatment for pneumonia?";
    
    // 让上下文选择器选择相关上下文
    let selected_contexts = context_selector
        .select_contexts(&user_id, &session_id, medical_query, "medical")
        .await
        .unwrap();

    println!("   Selected {} contexts for medical query", selected_contexts.len());
    for ctx in &selected_contexts {
        println!("   - Context ID: {}, Domain: {}, Priority: {}", ctx.id, ctx.domain, ctx.priority);
    }

    // 构建AI消息，包含选中的上下文
    let mut messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "你是一个专业助手，使用提供的上下文信息回答用户问题。".to_string(),
        }
    ];

    // 添加选中的上下文信息
    for ctx in &selected_contexts {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: format!("上下文信息 (Domain: {}): {}", ctx.domain, ctx.context_data),
        });
    }

    // 添加用户查询
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: medical_query.to_string(),
    });

    // 调用AI
    let response = ai_integration.get_ai_client().chat_completion(messages).await?;
    if let Some(choice) = response.choices.first() {
        println!("   AI Response: {}", choice.message.content);
    }

    // 5. 测试技术相关问题
    println!("\n5. 测试技术相关问题...");
    let tech_query = "Explain Rust async programming patterns.";
    
    let selected_contexts = context_selector
        .select_contexts(&user_id, &session_id, tech_query, "technical")
        .await
        .unwrap();

    println!("   Selected {} contexts for tech query", selected_contexts.len());
    for ctx in &selected_contexts {
        println!("   - Context ID: {}, Domain: {}, Priority: {}", ctx.id, ctx.domain, ctx.priority);
    }

    // 构建AI消息
    let mut messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "你是一个专业助手，使用提供的上下文信息回答用户问题。".to_string(),
        }
    ];

    for ctx in &selected_contexts {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: format!("上下文信息 (Domain: {}): {}", ctx.domain, ctx.context_data),
        });
    }

    messages.push(ChatMessage {
        role: "user".to_string(),
        content: tech_query.to_string(),
    });

    let response = ai_integration.get_ai_client().chat_completion(messages).await?;
    if let Some(choice) = response.choices.first() {
        println!("   AI Response: {}", choice.message.content);
    }

    // 6. 测试法律相关问题
    println!("\n6. 测试法律相关问题...");
    let legal_query = "What are the requirements for a valid contract?";
    
    let selected_contexts = context_selector
        .select_contexts(&user_id, &session_id, legal_query, "legal")
        .await
        .unwrap();

    println!("   Selected {} contexts for legal query", selected_contexts.len());
    for ctx in &selected_contexts {
        println!("   - Context ID: {}, Domain: {}, Priority: {}", ctx.id, ctx.domain, ctx.priority);
    }

    // 构建AI消息
    let mut messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "你是一个专业助手，使用提供的上下文信息回答用户问题。".to_string(),
        }
    ];

    for ctx in &selected_contexts {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: format!("上下文信息 (Domain: {}): {}", ctx.domain, ctx.context_data),
        });
    }

    messages.push(ChatMessage {
        role: "user".to_string(),
        content: legal_query.to_string(),
    });

    let response = ai_integration.get_ai_client().chat_completion(messages).await?;
    if let Some(choice) = response.choices.first() {
        println!("   AI Response: {}", choice.message.content);
    }

    // 7. 测试会话中的所有上下文
    println!("\n7. 获取会话中的所有上下文...");
    let session_contexts = context_manager.get_session_contexts(&session_id).await;
    println!("   Total contexts in session: {}", session_contexts.len());
    for ctx in &session_contexts {
        println!("   - Context ID: {}, Domain: {}, Priority: {}", ctx.id, ctx.domain, ctx.priority);
    }

    // 8. 测试用户的所有上下文
    println!("\n8. 获取用户的所有上下文...");
    let user_contexts = context_manager.get_user_contexts(&user_id).await;
    println!("   Total contexts for user: {}", user_contexts.len());
    for ctx in &user_contexts {
        println!("   - Context ID: {}, Domain: {}, Priority: {}", ctx.id, ctx.domain, ctx.priority);
    }

    println!("\n异步上下文管理测试完成！");
    Ok(())
}