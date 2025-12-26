use penlai::context::llm_context::ContextManager;
use penlai::selection::async_context_selector::ContextSelector;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化上下文管理器，设置较短的TTL用于测试
    let context_manager = Arc::new(ContextManager::new(10, 10)); // 10秒TTL，用于测试过期
    let context_selector = Arc::new(ContextSelector::new(context_manager.clone()));

    println!("1. 创建上下文...");
    let ctx1 = context_manager
        .create_context(
            "session_001".to_string(),
            "user_001".to_string(),
            "medical".to_string(),
            "Medical context for testing lifecycle".to_string(),
            8,
        )
        .await
        .map_err(|e| format!("Failed to create context 1: {}", e))?;

    let ctx2 = context_manager
        .create_context(
            "session_001".to_string(),
            "user_001".to_string(),
            "technical".to_string(),
            "Technical context for testing lifecycle".to_string(),
            7,
        )
        .await
        .map_err(|e| format!("Failed to create context 2: {}", e))?;

    println!("   Created context 1: {}", ctx1.id);
    println!("   Created context 2: {}", ctx2.id);

    // 验证上下文存在
    println!("\n2. 验证上下文存在...");
    let retrieved_ctx1 = context_manager.get_context(ctx1.id).await;
    let retrieved_ctx2 = context_manager.get_context(ctx2.id).await;
    println!("   Context 1 exists: {}", retrieved_ctx1.is_some());
    println!("   Context 2 exists: {}", retrieved_ctx2.is_some());

    // 验证会话和用户上下文
    let session_contexts = context_manager.get_session_contexts("session_001").await;
    let user_contexts = context_manager.get_user_contexts("user_001").await;
    println!("   Session contexts: {}", session_contexts.len());
    println!("   User contexts: {}", user_contexts.len());

    // 等待一段时间，让上下文过期
    println!("\n3. 等待上下文过期...");
    println!("   Waiting 12 seconds for contexts to expire...");
    tokio::time::sleep(tokio::time::Duration::from_secs(12)).await;

    // 验证上下文是否已过期
    println!("\n4. 验证上下文是否已过期...");
    let expired_ctx1 = context_manager.get_context(ctx1.id).await;
    let expired_ctx2 = context_manager.get_context(ctx2.id).await;
    println!("   Context 1 exists after TTL: {}", expired_ctx1.is_some());
    println!("   Context 2 exists after TTL: {}", expired_ctx2.is_some());

    // 验证会话和用户上下文是否也已过期
    let session_contexts_after = context_manager.get_session_contexts("session_001").await;
    let user_contexts_after = context_manager.get_user_contexts("user_001").await;
    println!("   Session contexts after TTL: {}", session_contexts_after.len());
    println!("   User contexts after TTL: {}", user_contexts_after.len());

    // 手动清理过期上下文
    println!("\n5. 手动清理过期上下文...");
    context_manager.cleanup_expired_contexts().await
        .map_err(|e| format!("Failed to cleanup expired contexts: {}", e))?;
    println!("   Expired contexts cleaned up");

    // 验证清理后的状态
    let session_contexts_cleaned = context_manager.get_session_contexts("session_001").await;
    let user_contexts_cleaned = context_manager.get_user_contexts("user_001").await;
    println!("   Session contexts after cleanup: {}", session_contexts_cleaned.len());
    println!("   User contexts after cleanup: {}", user_contexts_cleaned.len());

    // 创建新的上下文以验证系统继续工作
    println!("\n6. 创建新上下文以验证系统功能...");
    let new_ctx = context_manager
        .create_context(
            "session_002".to_string(),
            "user_002".to_string(),
            "general".to_string(),
            "New context after cleanup".to_string(),
            6,
        )
        .await
        .map_err(|e| format!("Failed to create new context: {}", e))?;

    println!("   Created new context: {}", new_ctx.id);

    // 测试上下文选择
    println!("\n7. 测试上下文选择...");
    let selected = context_selector
        .select_contexts("user_002", "session_002", "general query", "general")
        .await
        .map_err(|e| format!("Failed to select contexts: {}", e))?;
    println!("   Selected {} contexts for new session", selected.len());

    // 检查系统统计
    let stats = context_manager.get_stats().await;
    println!("\n8. 系统统计:");
    println!("   总上下文数: {}", stats.total_contexts);
    println!("   最大并发数: {}", stats.max_concurrent);
    println!("   可用许可数: {}", stats.available_permits);

    println!("\n上下文生命周期管理测试完成！");
    Ok(())
}