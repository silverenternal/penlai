use std::sync::Arc;
use tokio;
use penlai::context::llm_context;
use penlai::selection::async_context_selector;
use penlai::processing::concurrent_processor;
use penlai::monitoring::monitoring;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Penlai: Enterprise-Level Asynchronous Context Management Control for Large Language Models");

    // 初始化上下文管理器
    let context_manager = Arc::new(llm_context::ContextManager::new(100, 3600)); // 100并发，1小时TTL

    // 初始化上下文选择器
    let context_selector = Arc::new(async_context_selector::ContextSelector::new(context_manager.clone()));

    // 初始化请求处理器
    let request_processor = Arc::new(concurrent_processor::RequestProcessor::new(
        context_manager.clone(),
        context_selector.clone(),
    ));

    // 创建监控系统
    let monitoring_system = Arc::new(monitoring::MonitoringSystem::new());

    // 启动服务
    start_service(context_manager, context_selector, request_processor, monitoring_system).await?;

    Ok(())
}

async fn start_service(
    context_manager: Arc<penlai::context::llm_context::ContextManager>,
    context_selector: Arc<penlai::selection::async_context_selector::ContextSelector>,
    request_processor: Arc<penlai::processing::concurrent_processor::RequestProcessor>,
    monitoring_system: Arc<penlai::monitoring::monitoring::MonitoringSystem>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting Penlai enterprise service...");

    // 运行演示功能
    demo_functionality(context_manager, context_selector, request_processor, monitoring_system).await;

    Ok(())
}

async fn demo_functionality(
    context_manager: Arc<penlai::context::llm_context::ContextManager>,
    context_selector: Arc<penlai::selection::async_context_selector::ContextSelector>,
    request_processor: Arc<penlai::processing::concurrent_processor::RequestProcessor>,
    monitoring_system: Arc<penlai::monitoring::monitoring::MonitoringSystem>,
) {
    println!("Demonstrating enterprise-level Penlai capabilities...");

    // 开始时间用于计算性能指标
    let start_time = std::time::Instant::now();

    // 创建测试上下文
    let medical_context = context_manager
        .create_context(
            "session_medical_001".to_string(),
            "user_medical_001".to_string(),
            "medical".to_string(),
            "Medical guidelines for treating respiratory infections, including pneumonia, bronchitis, and other lung conditions. Treatment typically involves antibiotics, rest, and supportive care.".to_string(),
            9,
        )
        .await
        .expect("Failed to create medical context");

    let technical_context = context_manager
        .create_context(
            "session_tech_001".to_string(),
            "user_tech_001".to_string(),
            "technical".to_string(),
            "Rust async programming patterns, including async/await, futures, and concurrent processing. Best practices for building high-performance async applications.".to_string(),
            8,
        )
        .await
        .expect("Failed to create technical context");

    println!("Created contexts: {} and {}", medical_context.id, technical_context.id);

    // 记录上下文加载指标
    let context_load_duration = start_time.elapsed().as_millis() as f64;
    monitoring_system.record_metric("context_switch_time", penlai::monitoring::monitoring::PerformanceMetric::ContextSwitchTime(context_load_duration)).await;
    monitoring_system.log_event(penlai::monitoring::monitoring::MonitoringEvent::ContextLoaded {
        domain: "medical".to_string(),
        duration_ms: context_load_duration
    }).await;

    // 测试上下文选择
    let selection_start = std::time::Instant::now();
    let selected_contexts = context_selector
        .select_contexts("user_medical_001", "session_medical_001", "pneumonia treatment", "medical")
        .await
        .expect("Failed to select contexts");
    let selection_duration = selection_start.elapsed().as_millis() as f64;

    println!("Selected {} contexts for medical query in {:.2}ms", selected_contexts.len(), selection_duration);

    // 记录上下文选择指标
    monitoring_system.record_metric("context_selection_time", penlai::monitoring::monitoring::PerformanceMetric::ContextSelectionTime(selection_duration)).await;
    monitoring_system.log_event(penlai::monitoring::monitoring::MonitoringEvent::ContextSelected {
        query_length: "pneumonia treatment".len(),
        selected_count: selected_contexts.len(),
        duration_ms: selection_duration,
    }).await;

    // 测试请求处理
    let request_start = std::time::Instant::now();
    let request_result = request_processor
        .process_request(
            "user_medical_001".to_string(),
            "session_medical_001".to_string(),
            "What is the recommended treatment for pneumonia?".to_string(),
            "medical".to_string(),
        )
        .await;
    let request_duration = request_start.elapsed().as_millis() as f64;

    match request_result {
        Ok(result) => {
            println!("Request processed successfully in {:.2}ms: {}", request_duration, result.request_id);
            println!("Selected {} contexts for the query", result.selected_contexts.len());

            // 记录请求处理事件
            monitoring_system.log_event(penlai::monitoring::monitoring::MonitoringEvent::RequestProcessed {
                user_id: result.user_id,
                session_id: result.session_id,
                duration_ms: request_duration,
            }).await;
        }
        Err(e) => {
            eprintln!("Request processing failed: {:?}", e);
            monitoring_system.record_metric("error_rate", penlai::monitoring::monitoring::PerformanceMetric::ErrorRate(1.0)).await;
        }
    }

    // 测试并发处理能力
    test_concurrent_requests(request_processor.clone(), monitoring_system.clone()).await;

    // 检查性能阈值
    let alerts = monitoring_system.check_thresholds().await;
    if !alerts.is_empty() {
        println!("Performance alerts triggered: {}", alerts.len());
        for alert in &alerts {
            println!("  - {}", alert);
        }
    }

    // 显示统计信息
    let stats = monitoring_system.get_system_summary().await;
    println!("{}", stats);

    // 显示性能趋势
    let trends = monitoring_system.get_performance_trends("request_latency", 1).await;
    println!("Performance trends (last hour): {} data points", trends.len());

    println!("Demo completed successfully!");
}

async fn test_concurrent_requests(
    request_processor: Arc<penlai::processing::concurrent_processor::RequestProcessor>,
    monitoring_system: Arc<penlai::monitoring::monitoring::MonitoringSystem>,
) {
    println!("Testing concurrent request handling...");

    let mut handles = Vec::new();

    // 发起多个并发请求
    for i in 0..5 {
        let processor_clone = request_processor.clone();
        let monitoring_clone = monitoring_system.clone();
        let handle = tokio::spawn(async move {
            let start_time = std::time::Instant::now();
            let result = processor_clone
                .process_request(
                    format!("concurrent_user_{}", i),
                    format!("session_concurrent_{}", i),
                    format!("Query {} for testing concurrent processing", i),
                    "general".to_string(),
                )
                .await;
            let duration = start_time.elapsed().as_millis() as f64;

            match result {
                Ok(process_result) => {
                    println!("Concurrent request {} processed successfully in {:.2}ms", i, duration);
                    // 记录请求处理事件
                    monitoring_clone.log_event(penlai::monitoring::monitoring::MonitoringEvent::RequestProcessed {
                        user_id: process_result.user_id,
                        session_id: process_result.session_id,
                        duration_ms: duration,
                    }).await;
                },
                Err(e) => {
                    eprintln!("Concurrent request {} failed: {:?}", i, e);
                    monitoring_clone.record_metric("error_rate", penlai::monitoring::monitoring::PerformanceMetric::ErrorRate(1.0)).await;
                }
            }
        });

        handles.push(handle);
    }

    // 记录并发请求数量
    monitoring_system.record_metric("concurrent_requests", penlai::monitoring::monitoring::PerformanceMetric::ConcurrentRequests(handles.len())).await;

    // 等待所有并发请求完成
    for handle in handles {
        let _ = handle.await;
    }

    println!("Concurrent request testing completed");
}