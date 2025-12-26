use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::context::llm_context::{LLMContext, ContextManager};

/// 上下文选择策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextSelectionStrategy {
    PriorityBased,      // 基于优先级
    RecencyBased,       // 基于时间（最近使用）
    RelevanceBased,     // 基于相关性
    Hybrid,             // 混合策略
}

/// 上下文选择器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSelectorConfig {
    pub max_contexts_to_return: usize,  // 最大返回上下文数
    pub min_relevance_score: f64,       // 最小相关性分数
    pub selection_strategy: ContextSelectionStrategy, // 选择策略
    pub enable_cache: bool,             // 是否启用缓存
    pub cache_ttl_seconds: u64,         // 缓存TTL（秒）
}

impl Default for ContextSelectorConfig {
    fn default() -> Self {
        Self {
            max_contexts_to_return: 5,
            min_relevance_score: 0.3,
            selection_strategy: ContextSelectionStrategy::Hybrid,
            enable_cache: true,
            cache_ttl_seconds: 300, // 5分钟
        }
    }
}

/// 上下文选择器 - 企业级大模型上下文选择
pub struct ContextSelector {
    config: Arc<RwLock<ContextSelectorConfig>>,
    context_manager: Arc<ContextManager>,
    /// 查询-上下文ID缓存
    query_context_cache: Arc<RwLock<HashMap<String, (Vec<Uuid>, chrono::DateTime<chrono::Utc>)>>>,
}

impl ContextSelector {
    /// 创建新的上下文选择器
    pub fn new(context_manager: Arc<ContextManager>) -> Self {
        Self {
            config: Arc::new(RwLock::new(ContextSelectorConfig::default())),
            context_manager,
            query_context_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 选择与查询最相关的上下文
    pub async fn select_contexts(
        &self,
        user_id: &str,
        session_id: &str,
        query: &str,
        domain: &str,
    ) -> Result<Vec<LLMContext>, Box<dyn std::error::Error + Send + Sync>> {
        // 检查缓存
        if self.config.read().await.enable_cache {
            if let Some(cached_result) = self.get_cached_contexts(query, domain).await {
                return Ok(cached_result);
            }
        }

        // 获取相关上下文
        let mut candidate_contexts = Vec::new();

        // 从会话获取上下文
        candidate_contexts.extend(self.context_manager.get_session_contexts(session_id).await);

        // 从用户获取上下文
        candidate_contexts.extend(self.context_manager.get_user_contexts(user_id).await);

        // 从领域获取上下文
        candidate_contexts.extend(self.context_manager.get_domain_contexts(domain).await);

        // 移除重复项
        candidate_contexts = self.deduplicate_contexts(candidate_contexts).await;

        // 根据策略选择上下文
        let selected_contexts = self.apply_selection_strategy(
            candidate_contexts,
            query,
            &self.config.read().await.selection_strategy,
        ).await;

        // 应用最大数量限制
        let final_contexts: Vec<LLMContext> = selected_contexts
            .into_iter()
            .take(self.config.read().await.max_contexts_to_return)
            .collect();

        // 缓存结果
        if self.config.read().await.enable_cache {
            self.cache_contexts(query, domain, &final_contexts).await;
        }

        Ok(final_contexts)
    }

    /// 应用选择策略
    async fn apply_selection_strategy(
        &self,
        mut contexts: Vec<LLMContext>,
        query: &str,
        strategy: &ContextSelectionStrategy,
    ) -> Vec<LLMContext> {
        match strategy {
            ContextSelectionStrategy::PriorityBased => {
                contexts.sort_by(|a, b| b.priority.cmp(&a.priority));
                contexts
            }
            ContextSelectionStrategy::RecencyBased => {
                contexts.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                contexts
            }
            ContextSelectionStrategy::RelevanceBased => {
                let mut scored_contexts = Vec::new();
                for context in contexts {
                    let score = self.calculate_relevance_score(&context.context_data, query).await;
                    if score >= self.config.read().await.min_relevance_score {
                        scored_contexts.push((context, score));
                    }
                }
                scored_contexts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                scored_contexts.into_iter().map(|(ctx, _)| ctx).collect()
            }
            ContextSelectionStrategy::Hybrid => {
                let mut scored_contexts = Vec::new();
                for context in contexts {
                    let relevance_score = self.calculate_relevance_score(&context.context_data, query).await;
                    if relevance_score >= self.config.read().await.min_relevance_score {
                        // 综合考虑相关性、优先级和时间
                        let hybrid_score = relevance_score * 0.5 + 
                                         (context.priority as f64 / 10.0) * 0.3 + 
                                         self.time_decay_score(&context.updated_at).await * 0.2;
                        scored_contexts.push((context, hybrid_score));
                    }
                }
                scored_contexts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                scored_contexts.into_iter().map(|(ctx, _)| ctx).collect()
            }
        }
    }

    /// 计算上下文相关性分数
    async fn calculate_relevance_score(&self, context_data: &str, query: &str) -> f64 {
        // 简化的相关性计算 - 在实际实现中，这可能使用向量嵌入或更复杂的算法
        let context_lower = context_data.to_lowercase();
        let query_lower = query.to_lowercase();
        
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let context_words: std::collections::HashSet<&str> = context_lower.split_whitespace().collect();

        let mut matches = 0;
        for word in &query_words {
            if context_words.contains(word) {
                matches += 1;
            }
        }

        if query_words.is_empty() {
            0.0
        } else {
            matches as f64 / query_words.len() as f64
        }
    }

    /// 时间衰减分数计算
    async fn time_decay_score(&self, updated_at: &chrono::DateTime<chrono::Utc>) -> f64 {
        let now = chrono::Utc::now();
        let duration = now - *updated_at;
        
        // 基于时间的衰减函数，最近的上下文得分更高
        let hours_since_update = duration.num_seconds() as f64 / 3600.0;
        // 衰减函数：分数随时间指数衰减
        (1.0 / (1.0 + hours_since_update * 0.1)).max(0.0).min(1.0)
    }

    /// 去除重复上下文
    async fn deduplicate_contexts(&self, contexts: Vec<LLMContext>) -> Vec<LLMContext> {
        let mut seen_ids = std::collections::HashSet::new();
        contexts.into_iter().filter(|ctx| seen_ids.insert(ctx.id)).collect()
    }

    /// 获取缓存的上下文
    async fn get_cached_contexts(&self, query: &str, domain: &str) -> Option<Vec<LLMContext>> {
        let cache_key = format!("{}:{}", query, domain);
        let cache = self.query_context_cache.read().await;
        
        if let Some((context_ids, cache_time)) = cache.get(&cache_key) {
            // 检查缓存是否过期
            let now = chrono::Utc::now();
            let ttl = chrono::Duration::seconds(self.config.read().await.cache_ttl_seconds as i64);
            
            if now - *cache_time < ttl {
                // 在实际实现中，这里会从缓存中获取上下文
                // 为演示目的，我们返回空向量
                Some(Vec::new()) // 实际实现会从缓存中获取上下文对象
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 缓存上下文
    async fn cache_contexts(&self, query: &str, domain: &str, contexts: &[LLMContext]) {
        if !self.config.read().await.enable_cache {
            return;
        }
        
        let cache_key = format!("{}:{}", query, domain);
        let context_ids: Vec<Uuid> = contexts.iter().map(|ctx| ctx.id).collect();
        let now = chrono::Utc::now();
        
        let mut cache = self.query_context_cache.write().await;
        cache.insert(cache_key, (context_ids, now));
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: ContextSelectorConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> ContextSelectorConfig {
        self.config.read().await.clone()
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.query_context_cache.write().await;
        cache.clear();
    }

    /// 清除过期缓存
    pub async fn clear_expired_cache(&self) {
        let config = self.config.read().await;
        let ttl = chrono::Duration::seconds(config.cache_ttl_seconds as i64);
        let now = chrono::Utc::now();
        
        let mut cache = self.query_context_cache.write().await;
        cache.retain(|_, (_, cache_time)| now - *cache_time < ttl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio;

    #[tokio::test]
    async fn test_context_selector() {
        let context_manager = Arc::new(ContextManager::new(10, 3600));
        let selector = ContextSelector::new(context_manager.clone());

        // 创建测试上下文
        let ctx1 = context_manager
            .create_context(
                "session1".to_string(),
                "user1".to_string(),
                "medical".to_string(),
                "Treatment for pneumonia involves antibiotics".to_string(),
                8,
            )
            .await
            .unwrap();

        let ctx2 = context_manager
            .create_context(
                "session1".to_string(),
                "user1".to_string(),
                "medical".to_string(),
                "Symptoms of flu include fever and fatigue".to_string(),
                7,
            )
            .await
            .unwrap();

        // 测试上下文选择
        let selected = selector
            .select_contexts("user1", "session1", "pneumonia treatment", "medical")
            .await
            .unwrap();

        assert!(!selected.is_empty());
        println!("Selected {} contexts", selected.len());

        // 测试配置更新
        let new_config = ContextSelectorConfig {
            max_contexts_to_return: 1,
            min_relevance_score: 0.1,
            selection_strategy: ContextSelectionStrategy::RelevanceBased,
            enable_cache: true,
            cache_ttl_seconds: 300,
        };
        
        selector.update_config(new_config).await;
        let config = selector.get_config().await;
        assert_eq!(config.max_contexts_to_return, 1);

        // 测试清除缓存
        selector.clear_cache().await;
    }
}