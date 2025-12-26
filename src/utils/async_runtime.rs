use tokio;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};
use crate::context::llm_context::ContextManager;
use crate::domain::domain_classifier::DomainClassifier;
use crate::context::context_loader::ContextLoader;
use crate::selection::context_selector::ContextSelector;

/// 异步运行时配置
pub struct AsyncRuntimeConfig {
    pub max_concurrent_requests: usize,  // 最大并发请求数
    pub request_timeout_ms: u64,         // 请求超时时间（毫秒）
    pub context_load_timeout_ms: u64,    // 上下文加载超时时间
    pub context_selection_timeout_ms: u64, // 上下文选择超时时间
}

impl Default for AsyncRuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
            request_timeout_ms: 5000,
            context_load_timeout_ms: 2000,
            context_selection_timeout_ms: 1000,
        }
    }
}

/// 异步运行时 - 管理并发请求和资源分配
pub struct AsyncRuntime {
    /// 信号量用于限制并发数
    concurrency_limiter: Arc<Semaphore>,
    
    /// 运行时配置
    config: AsyncRuntimeConfig,
    
    /// 上下文管理器
    context_manager: Arc<ContextManager>,
    
    /// 领域分类器
    domain_classifier: Arc<DomainClassifier>,
    
    /// 上下文加载器
    context_loader: Arc<ContextLoader>,
    
    /// 上下文选择器
    context_selector: Arc<ContextSelector>,
}

impl AsyncRuntime {
    /// 创建新的异步运行时
    pub fn new(
        context_manager: Arc<ContextManager>,
        domain_classifier: Arc<DomainClassifier>,
        context_loader: Arc<ContextLoader>,
        context_selector: Arc<ContextSelector>,
    ) -> Self {
        let config = AsyncRuntimeConfig::default();
        let concurrency_limiter = Arc::new(Semaphore::new(config.max_concurrent_requests));

        Self {
            concurrency_limiter,
            config,
            context_manager,
            domain_classifier,
            context_loader,
            context_selector,
        }
    }

    /// 处理单个请求
    pub async fn process_request(&self, query: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // 获取信号量许可以限制并发
        let _permit = self.concurrency_limiter
            .acquire()
            .await
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // 1. 识别领域
        let domain = timeout(
            Duration::from_millis(self.config.request_timeout_ms),
            DomainClassifier::classify_domain_async(&query)
        )
        .await
        .map_err(|_| "领域分类超时")?
        .to_string();

        // 2. 加载上下文
        let contexts = timeout(
            Duration::from_millis(self.config.context_load_timeout_ms),
            self.load_contexts_for_domain(&domain)
        )
        .await
        .map_err(|_| "上下文加载超时")?;

        // 3. 选择合适的上下文
        let selected_contexts = timeout(
            Duration::from_millis(self.config.context_selection_timeout_ms),
            self.select_contexts(&contexts, &query)
        )
        .await
        .map_err(|_| "上下文选择超时")?;

        // 4. 生成响应（简化版）
        let response = self.generate_response(&selected_contexts, &query).await;

        Ok(response)
    }

    /// 为特定领域加载上下文
    async fn load_contexts_for_domain(&self, domain_str: &str) -> Vec<crate::context::llm_context::LLMContext> {
        // 在实际实现中，这里会调用真正的上下文加载逻辑
        // 为演示目的，我们返回一些示例上下文
        use crate::domain::domain_classifier::Domain;
        

        let domain = match domain_str {
            "medical" => Domain::Medical,
            "legal" => Domain::Legal,
            "technical" => Domain::Technical,
            "education" => Domain::Education,
            "finance" => Domain::Finance,
            _ => Domain::General,
        };

        crate::context::context_loader::ContextLoader::load_context_for_domain(&domain).await.unwrap_or_default()
    }

    /// 选择与查询相关的上下文
    async fn select_contexts(
        &self,
        contexts: &[crate::context::llm_context::LLMContext],
        query: &str
    ) -> Vec<crate::context::llm_context::LLMContext> {
        // 在实际实现中，这里会调用真正的上下文选择逻辑
        // 为演示目的，我们返回前几个上下文
        contexts.iter()
            .take(3)
            .cloned()
            .collect()
    }

    /// 生成响应
    async fn generate_response(&self, contexts: &[crate::context::llm_context::LLMContext], query: &str) -> String {
        if contexts.is_empty() {
            format!("未能找到与查询 '{}' 相关的上下文", query)
        } else {
            format!(
                "基于以下上下文回答查询 '{}': {}",
                query,
                contexts.first().unwrap().context_data
            )
        }
    }

    /// 更新运行时配置
    pub fn update_config(&mut self, new_config: AsyncRuntimeConfig) {
        self.config = new_config;
        // 重新创建信号量以应用新的并发限制
        self.concurrency_limiter = Arc::new(Semaphore::new(self.config.max_concurrent_requests));
    }

    /// 获取当前运行时统计信息
    pub async fn get_runtime_stats(&self) -> RuntimeStats {
        let available_permits = self.concurrency_limiter.available_permits();
        let max_concurrent = self.config.max_concurrent_requests;
        let active_requests = max_concurrent - available_permits;

        RuntimeStats {
            active_requests,
            max_concurrent_requests: max_concurrent,
            available_permits,
        }
    }
}

/// 运行时统计信息
pub struct RuntimeStats {
    pub active_requests: usize,           // 活跃请求数
    pub max_concurrent_requests: usize,   // 最大并发请求数
    pub available_permits: usize,         // 可用许可数
}

impl std::fmt::Display for RuntimeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RuntimeStats {{ active_requests: {}, max_concurrent: {}, available_permits: {} }}",
            self.active_requests,
            self.max_concurrent_requests,
            self.available_permits
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::context::llm_context::ContextManager;
    use crate::domain::domain_classifier::DomainClassifier;
    use crate::context::context_loader::ContextLoader;
    use crate::selection::context_selector::ContextSelector;

    #[tokio::test]
    async fn test_async_runtime() {
        let context_manager = Arc::new(ContextManager::new(10, 3600));
        let domain_classifier = Arc::new(DomainClassifier::new().unwrap());
        let context_loader = Arc::new(ContextLoader::new(context_manager.clone()));
        let context_selector = Arc::new(ContextSelector::new());

        let runtime = AsyncRuntime::new(
            context_manager,
            domain_classifier,
            context_loader,
            context_selector,
        );

        // 测试处理请求
        let query = "What is the treatment for pneumonia?".to_string();
        let result = runtime.process_request(query).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        println!("Response: {}", response);
        
        // 测试运行时统计
        let stats = runtime.get_runtime_stats().await;
        println!("{}", stats);
        
        assert!(stats.active_requests >= 0);
        assert_eq!(stats.max_concurrent_requests, 100); // 默认值
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let context_manager = Arc::new(ContextManager::new(10, 3600));
        let domain_classifier = Arc::new(DomainClassifier::new().unwrap());
        let context_loader = Arc::new(ContextLoader::new(context_manager.clone()));
        let context_selector = Arc::new(ContextSelector::new());

        let runtime = Arc::new(AsyncRuntime::new(
            context_manager,
            domain_classifier,
            context_loader,
            context_selector,
        ));

        // 同时发起多个请求以测试并发控制
        let mut handles = vec![];
        for i in 0..10 {
            let runtime_clone = runtime.clone();
            let query = format!("Query {}", i);
            let handle = tokio::spawn(async move {
                runtime_clone.process_request(query).await
            });
            handles.push(handle);
        }

        // 等待所有请求完成
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            let inner_result = result.unwrap();
            assert!(inner_result.is_ok());
        }

        // 检查最终统计信息
        let stats = runtime.get_runtime_stats().await;
        println!("Final stats: {}", stats);
    }
}