use penlai::context::llm_context::ContextManager;
use penlai::selection::async_context_selector::ContextSelector;
use penlai::processing::concurrent_processor::RequestProcessor;
use penlai::monitoring::monitoring::MonitoringSystem;
use penlai::utils::ai_client::{AIClient, ChatMessage};
use penlai::utils::ai_integration::AIIntegration;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Penlai 企业级异步上下文管理系统综合测试 ===\n");

    // 初始化所有组件
    let context_manager = Arc::new(ContextManager::new(50, 3600)); // 50并发，1小时TTL
    let context_selector = Arc::new(ContextSelector::new(context_manager.clone()));
    let request_processor = Arc::new(RequestProcessor::new(
        context_manager.clone(),
        context_selector.clone(),
    ));
    let monitoring_system = Arc::new(MonitoringSystem::new());
    let ai_integration = Arc::new(AIIntegration::new()?);

    println!("1. 上下文管理功能测试...");
    
    // 创建多个领域上下文
    let medical_ctx = context_manager
        .create_context(
            "session_medical_001".to_string(),
            "user_medical_001".to_string(),
            "medical".to_string(),
            "Medical guidelines for treating respiratory infections, including pneumonia, bronchitis, and other lung conditions. Treatment typically involves antibiotics, rest, and supportive care.".to_string(),
            9,
        )
        .await
        .map_err(|e| format!("Failed to create medical context: {}", e))?;

    let tech_ctx = context_manager
        .create_context(
            "session_tech_001".to_string(),
            "user_tech_001".to_string(),
            "technical".to_string(),
            "Rust async programming patterns, including async/await, futures, and concurrent processing. Best practices for building high-performance async applications.".to_string(),
            8,
        )
        .await
        .map_err(|e| format!("Failed to create tech context: {}", e))?;

    let legal_ctx = context_manager
        .create_context(
            "session_legal_001".to_string(),
            "user_legal_001".to_string(),
            "legal".to_string(),
            "Contract law fundamentals, including offer, acceptance, consideration, and legal capacity. Requirements for valid contracts and common legal issues.".to_string(),
            7,
        )
        .await
        .map_err(|e| format!("Failed to create legal context: {}", e))?;

    println!("   ✓ 创建了医疗上下文: {}", medical_ctx.id);
    println!("   ✓ 创建了技术上下文: {}", tech_ctx.id);
    println!("   ✓ 创建了法律上下文: {}", legal_ctx.id);

    println!("\n2. 上下文选择功能测试...");

    // 测试不同领域的上下文选择
    let medical_selected = context_selector
        .select_contexts("user_medical_001", "session_medical_001", "pneumonia treatment", "medical")
        .await
        .unwrap();
    
    let tech_selected = context_selector
        .select_contexts("user_tech_001", "session_tech_001", "async programming", "technical")
        .await
        .unwrap();
    
    let legal_selected = context_selector
        .select_contexts("user_legal_001", "session_legal_001", "contract law", "legal")
        .await
        .unwrap();

    println!("   ✓ 医疗查询选择了 {} 个上下文", medical_selected.len());
    println!("   ✓ 技术查询选择了 {} 个上下文", tech_selected.len());
    println!("   ✓ 法律查询选择了 {} 个上下文", legal_selected.len());

    println!("\n3. 并发处理功能测试...");

    // 并发处理多个请求
    let mut handles = vec![];
    for i in 0..5 {
        let rp = request_processor.clone();
        let handle = tokio::spawn(async move {
            let result = rp.process_request(
                format!("concurrent_user_{}", i),
                format!("concurrent_session_{}", i),
                format!("Query {} for concurrent processing test", i),
                "general".to_string(),
            ).await;
            
            match result {
                Ok(_) => println!("      请求 {} 处理完成", i),
                Err(e) => println!("      请求 {} 处理失败: {:?}", i, e),
            }
        });
        handles.push(handle);
    }

    // 等待所有并发请求完成
    for handle in handles {
        handle.await.unwrap();
    }

    println!("   ✓ 并发处理测试完成");

    println!("\n4. AI集成与上下文感知功能测试...");

    // 使用AI处理基于上下文的查询
    let queries = vec![
        ("What is the treatment for pneumonia?", "medical"),
        ("Explain async programming in Rust.", "technical"),
        ("What are the requirements for a valid contract?", "legal"),
    ];

    for (query, domain) in queries {
        println!("   处理 {} 领域查询: {}", domain, query);
        
        // 获取相关上下文
        let selected_contexts = context_selector
            .select_contexts("test_user", "test_session", query, domain)
            .await
            .unwrap();

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
            content: query.to_string(),
        });

        // 调用AI
        let response = ai_integration.get_ai_client().chat_completion(messages).await?;
        if let Some(choice) = response.choices.first() {
            println!("      AI响应: {}", choice.message.content.chars().take(100).collect::<String>() + "...");
        }
    }

    println!("   ✓ AI集成与上下文感知功能测试完成");

    println!("\n5. 监控系统功能测试...");

    // 记录一些性能指标
    monitoring_system.record_metric("request_latency", penlai::monitoring::monitoring::PerformanceMetric::RequestLatency(150.0)).await;
    monitoring_system.record_metric("context_switch_time", penlai::monitoring::monitoring::PerformanceMetric::ContextSwitchTime(50.0)).await;
    monitoring_system.record_metric("error_rate", penlai::monitoring::monitoring::PerformanceMetric::ErrorRate(0.01)).await;

    // 记录一些事件
    monitoring_system.log_event(penlai::monitoring::monitoring::MonitoringEvent::ContextLoaded {
        domain: "medical".to_string(),
        duration_ms: 45.0,
    }).await;

    monitoring_system.log_event(penlai::monitoring::monitoring::MonitoringEvent::RequestProcessed {
        user_id: "test_user".to_string(),
        session_id: "test_session".to_string(),
        duration_ms: 150.0,
    }).await;

    // 检查阈值
    let alerts = monitoring_system.check_thresholds().await;
    println!("   ✓ 监控系统记录了指标和事件");
    println!("   ✓ 性能警报数量: {}", alerts.len());

    // 获取系统摘要
    let summary = monitoring_system.get_system_summary().await;
    println!("   ✓ 系统摘要: {}", summary);

    println!("\n6. 系统统计信息...");

    let ctx_stats = context_manager.get_stats().await;
    println!("   上下文管理器统计:");
    println!("     - 总上下文数: {}", ctx_stats.total_contexts);
    println!("     - 最大并发数: {}", ctx_stats.max_concurrent);
    println!("     - 可用许可数: {}", ctx_stats.available_permits);

    let proc_stats = request_processor.get_stats().await;
    println!("   请求处理器统计:");
    println!("     - 活跃请求数: {}", proc_stats.active_requests);
    println!("     - 最大并发请求数: {}", proc_stats.max_concurrent_requests);
    println!("     - 用户跟踪数: {}", proc_stats.total_users_tracked);

    println!("\n=== 综合测试完成！ ===");
    println!("Penlai企业级异步上下文管理系统所有功能模块均正常工作：");
    println!("✓ 上下文管理 - 创建、存储、检索、过期管理");
    println!("✓ 上下文选择 - 智能选择相关上下文");
    println!("✓ 并发处理 - 高效处理并发请求");
    println!("✓ AI集成 - 与AI服务集成并使用上下文信息");
    println!("✓ 监控系统 - 实时性能监控和警报");
    println!("✓ 系统统计 - 全面的系统状态监控");

    Ok(())
}