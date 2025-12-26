use penlai::context::llm_context::ContextManager;
use penlai::selection::async_context_selector::ContextSelector;
use penlai::utils::ai_client::{AIClient, ChatMessage};
use penlai::utils::ai_integration::AIIntegration;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化上下文管理器
    let context_manager = Arc::new(ContextManager::new(100, 3600)); // 100并发，1小时TTL
    let context_selector = Arc::new(ContextSelector::new(context_manager.clone()));
    let ai_integration = Arc::new(AIIntegration::new()?);

    // 创建多个用户和会话进行并发测试
    let users = vec!["user_001", "user_002", "user_003"];
    let sessions = vec!["session_001", "session_002", "session_003"];
    
    // 创建多个任务进行并发处理
    let mut handles = vec![];
    
    for (i, &user) in users.iter().enumerate() {
        let cm = context_manager.clone();
        let cs = context_selector.clone();
        let ai = ai_integration.clone();
        let session = sessions[i];
        
        let handle = tokio::spawn(async move {
            println!("用户 {} 开始创建上下文和处理请求...", user);
            
            // 为每个用户创建不同的上下文
            let medical_ctx = cm.create_context(
                session.to_string(),
                user.to_string(),
                "medical".to_string(),
                format!("Medical guidelines for user {} - Treatment protocols for various conditions.", user),
                9,
            ).await.unwrap();
            
            let tech_ctx = cm.create_context(
                session.to_string(),
                user.to_string(),
                "technical".to_string(),
                format!("Technical documentation for user {} - System architecture and best practices.", user),
                8,
            ).await.unwrap();
            
            println!("   用户 {} 创建了上下文: {} 和 {}", user, medical_ctx.id, tech_ctx.id);
            
            // 处理医疗相关查询
            let medical_query = format!("Medical query from {} - What is the treatment?", user);
            let selected_medical = cs.select_contexts(user, session, &medical_query, "medical").await.unwrap();
            println!("   用户 {} 医疗查询选择了 {} 个上下文", user, selected_medical.len());
            
            // 处理技术相关查询
            let tech_query = format!("Technical query from {} - How to implement?", user);
            let selected_tech = cs.select_contexts(user, session, &tech_query, "technical").await.unwrap();
            println!("   用户 {} 技术查询选择了 {} 个上下文", user, selected_tech.len());
            
            // 验证上下文隔离
            let user_contexts = cm.get_user_contexts(user).await;
            println!("   用户 {} 总共有 {} 个上下文", user, user_contexts.len());
            
            println!("用户 {} 完成", user);
        });
        
        handles.push(handle);
    }
    
    // 等待所有并发任务完成
    for handle in handles {
        handle.await.unwrap();
    }
    
    // 检查总的上下文数量
    let stats = context_manager.get_stats().await;
    println!("\n系统统计:");
    println!("   总上下文数: {}", stats.total_contexts);
    println!("   最大并发数: {}", stats.max_concurrent);
    println!("   可用许可数: {}", stats.available_permits);
    
    // 检查特定会话的上下文
    println!("\n检查会话 session_001 的上下文:");
    let session_contexts = context_manager.get_session_contexts("session_001").await;
    for ctx in &session_contexts {
        println!("   - ID: {}, Domain: {}, User: {}", ctx.id, ctx.domain, ctx.user_id);
    }
    
    println!("\n并发异步上下文管理测试完成！");
    Ok(())
}