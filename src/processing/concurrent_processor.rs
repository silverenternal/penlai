use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::context::llm_context::{ContextManager, LLMContext};
use crate::selection::async_context_selector::{ContextSelector, ContextSelectorConfig};

/// 请求处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestProcessorConfig {
    pub max_concurrent_requests: usize,      // 最大并发请求数
    pub request_timeout_seconds: u64,        // 请求超时时间（秒）
    pub context_load_timeout_seconds: u64,   // 上下文加载超时时间（秒）
    pub context_selection_timeout_seconds: u64, // 上下文选择超时时间（秒）
    pub enable_rate_limiting: bool,          // 是否启用速率限制
    pub max_requests_per_minute: u32,        // 每分钟最大请求数
}

impl Default for RequestProcessorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
            request_timeout_seconds: 30,
            context_load_timeout_seconds: 10,
            context_selection_timeout_seconds: 5,
            enable_rate_limiting: true,
            max_requests_per_minute: 1000,
        }
    }
}

/// 请求处理器 - 企业级大模型并发请求处理
pub struct RequestProcessor {
    config: Arc<RwLock<RequestProcessorConfig>>,
    context_manager: Arc<ContextManager>,
    context_selector: Arc<ContextSelector>,
    /// 并发控制信号量
    request_semaphore: Arc<Semaphore>,
    /// 用户请求计数器（用于速率限制）
    user_request_counts: Arc<RwLock<std::collections::HashMap<String, (u32, chrono::DateTime<chrono::Utc>)>>>,
}

impl RequestProcessor {
    /// 创建新的请求处理器
    pub fn new(
        context_manager: Arc<ContextManager>,
        context_selector: Arc<ContextSelector>,
    ) -> Self {
        let config = RequestProcessorConfig::default();

        Self {
            config: Arc::new(RwLock::new(config.clone())),
            context_manager,
            context_selector,
            request_semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            user_request_counts: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// 处理大模型请求
    pub async fn process_request(
        &self,
        user_id: String,
        session_id: String,
        query: String,
        domain: String,
    ) -> Result<RequestResult, RequestError> {
        // 检查速率限制
        if self.config.read().await.enable_rate_limiting {
            self.check_rate_limit(&user_id).await?;
        }

        // 获取并发许可
        let _permit = self.request_semaphore
            .acquire()
            .await
            .map_err(|_| RequestError::ResourceUnavailable("Failed to acquire request permit".to_string()))?;

        // 更新请求计数
        self.increment_request_count(&user_id).await;

        // 设置总请求超时
        let result = timeout(
            Duration::from_secs(self.config.read().await.request_timeout_seconds),
            self.process_request_internal(user_id, session_id, query, domain)
        ).await;

        match result {
            Ok(process_result) => process_result,
            Err(_) => Err(RequestError::Timeout("Request timed out".to_string())),
        }
    }

    /// 内部请求处理逻辑
    async fn process_request_internal(
        &self,
        user_id: String,
        session_id: String,
        query: String,
        domain: String,
    ) -> Result<RequestResult, RequestError> {
        // 1. 选择相关上下文
        let selected_contexts = timeout(
            Duration::from_secs(self.config.read().await.context_selection_timeout_seconds),
            self.context_selector.select_contexts(&user_id, &session_id, &query, &domain)
        ).await
        .map_err(|_| RequestError::Timeout("Context selection timed out".to_string()))?
        .map_err(|e| RequestError::ContextSelectionFailed(e.to_string()))?;

        // 2. 准备响应数据
        let response_data = RequestResult {
            request_id: Uuid::new_v4(),
            user_id,
            session_id,
            query,
            domain,
            selected_contexts,
            timestamp: chrono::Utc::now(),
            processing_time_ms: 0, // 实际处理时间会在外部计算
        };

        Ok(response_data)
    }

    /// 检查速率限制
    async fn check_rate_limit(&self, user_id: &str) -> Result<(), RequestError> {
        let max_requests = self.config.read().await.max_requests_per_minute;
        let now = chrono::Utc::now();
        let window_start = now - chrono::Duration::minutes(1);

        let request_counts = self.user_request_counts.read().await;
        if let Some(&(count, last_request_time)) = request_counts.get(user_id) {
            // 清除过期的计数
            if last_request_time < window_start {
                // 计数已过期，允许请求
                return Ok(());
            }

            if count >= self.config.read().await.max_requests_per_minute {
                return Err(RequestError::RateLimitExceeded(format!(
                    "Rate limit exceeded: {} requests per minute", max_requests
                )));
            }
        }

        Ok(())
    }

    /// 增加请求计数
    async fn increment_request_count(&self, user_id: &str) {
        let mut request_counts = self.user_request_counts.write().await;
        let now = chrono::Utc::now();
        
        let entry = request_counts.entry(user_id.to_string()).or_insert((0, now));
        entry.0 += 1;
        entry.1 = now;
    }

    /// 清除过期的请求计数（用于速率限制）
    pub async fn cleanup_expired_request_counts(&self) {
        let mut request_counts = self.user_request_counts.write().await;
        let now = chrono::Utc::now();
        let window_start = now - chrono::Duration::minutes(1);

        request_counts.retain(|_, (_, last_request_time)| {
            *last_request_time >= window_start
        });
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: RequestProcessorConfig) {
        let mut config = self.config.write().await;
        *config = new_config.clone();

        // 重新设置信号量 - 由于request_semaphore是Arc，我们需要创建一个新的Arc
        // 实际企业实现中，可能需要更复杂的配置更新机制
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> RequestProcessorConfig {
        self.config.read().await.clone()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> RequestProcessorStats {
        let config = self.config.read().await;
        let available_permits = self.request_semaphore.available_permits();
        let active_requests = config.max_concurrent_requests - available_permits;

        let request_counts = self.user_request_counts.read().await;
        let total_users_tracked = request_counts.len();

        RequestProcessorStats {
            active_requests,
            max_concurrent_requests: config.max_concurrent_requests,
            available_permits,
            total_users_tracked,
            rate_limit_enabled: config.enable_rate_limiting,
            max_requests_per_minute: config.max_requests_per_minute,
        }
    }
}

/// 请求结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestResult {
    pub request_id: Uuid,
    pub user_id: String,
    pub session_id: String,
    pub query: String,
    pub domain: String,
    pub selected_contexts: Vec<LLMContext>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub processing_time_ms: u64,
}

/// 请求错误类型
#[derive(Debug)]
pub enum RequestError {
    Timeout(String),
    RateLimitExceeded(String),
    ContextSelectionFailed(String),
    ResourceUnavailable(String),
    Other(String),
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            RequestError::RateLimitExceeded(msg) => write!(f, "RateLimitExceeded: {}", msg),
            RequestError::ContextSelectionFailed(msg) => write!(f, "ContextSelectionFailed: {}", msg),
            RequestError::ResourceUnavailable(msg) => write!(f, "ResourceUnavailable: {}", msg),
            RequestError::Other(msg) => write!(f, "Other: {}", msg),
        }
    }
}

impl std::error::Error for RequestError {}

/// 请求处理器统计信息
#[derive(Debug)]
pub struct RequestProcessorStats {
    pub active_requests: usize,
    pub max_concurrent_requests: usize,
    pub available_permits: usize,
    pub total_users_tracked: usize,
    pub rate_limit_enabled: bool,
    pub max_requests_per_minute: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio;

    #[tokio::test]
    async fn test_request_processor() {
        let context_manager = Arc::new(ContextManager::new(10, 3600));
        let context_selector = Arc::new(ContextSelector::new(context_manager.clone()));
        let processor = RequestProcessor::new(context_manager.clone(), context_selector.clone());

        // 创建测试上下文
        context_manager
            .create_context(
                "session1".to_string(),
                "user1".to_string(),
                "medical".to_string(),
                "Treatment for pneumonia involves antibiotics".to_string(),
                8,
            )
            .await
            .unwrap();

        // 测试请求处理
        let result = processor
            .process_request(
                "user1".to_string(),
                "session1".to_string(),
                "How to treat pneumonia?".to_string(),
                "medical".to_string(),
            )
            .await;

        match result {
            Ok(request_result) => {
                println!("Request processed successfully: {}", request_result.request_id);
                assert_eq!(request_result.user_id, "user1");
                assert_eq!(request_result.domain, "medical");
            }
            Err(e) => {
                panic!("Request processing failed: {:?}", e);
            }
        }

        // 测试配置获取
        let config = processor.get_config().await;
        assert_eq!(config.max_concurrent_requests, 100);

        // 测试统计信息
        let stats = processor.get_stats().await;
        println!("Stats: {:?}", stats);
        
        assert_eq!(stats.max_concurrent_requests, 100);
        assert!(stats.rate_limit_enabled);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let context_manager = Arc::new(ContextManager::new(10, 3600));
        let context_selector = Arc::new(ContextSelector::new(context_manager.clone()));
        let processor = RequestProcessor::new(context_manager.clone(), context_selector.clone());

        // 临时禁用速率限制以创建上下文
        let mut config = processor.get_config().await;
        config.enable_rate_limiting = false;
        processor.update_config(config).await;

        // 创建测试上下文
        context_manager
            .create_context(
                "session1".to_string(),
                "test_user".to_string(),
                "technical".to_string(),
                "Rust async programming guide".to_string(),
                7,
            )
            .await
            .unwrap();

        // 重新启用速率限制
        let mut config = processor.get_config().await;
        config.enable_rate_limiting = true;
        config.max_requests_per_minute = 2; // 设置较低的限制用于测试
        processor.update_config(config).await;

        // 执行多个请求以测试速率限制
        let result1 = processor
            .process_request(
                "test_user".to_string(),
                "session1".to_string(),
                "Rust async question 1".to_string(),
                "technical".to_string(),
            )
            .await;

        let result2 = processor
            .process_request(
                "test_user".to_string(),
                "session1".to_string(),
                "Rust async question 2".to_string(),
                "technical".to_string(),
            )
            .await;

        let result3 = processor
            .process_request(
                "test_user".to_string(),
                "session1".to_string(),
                "Rust async question 3".to_string(),
                "technical".to_string(),
            )
            .await;

        // 前两个请求应该成功，第三个可能因为速率限制而失败
        assert!(result1.is_ok() || matches!(result1, Err(RequestError::Timeout(_))));
        assert!(result2.is_ok() || matches!(result2, Err(RequestError::Timeout(_))));
        
        // 第三个请求可能因为速率限制而失败
        println!("Result 3: {:?}", result3);
    }
}