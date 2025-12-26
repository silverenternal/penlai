use crate::context::llm_context::LLMContext as Context;
use crate::domain::domain_classifier::Domain;
use std::collections::HashMap;

/// 上下文管理策略枚举
#[derive(Debug)]
pub enum ContextManagementStrategy {
    PriorityBased,      // 基于优先级
    Lru,                // 最近最少使用
    FrequencyBased,     // 基于使用频率
    Hybrid,             // 混合策略
}

/// 领域识别策略枚举
#[derive(Debug)]
pub enum DomainRecognitionStrategy {
    KeywordMatching,    // 关键词匹配
    VectorSimilarity,   // 向量相似度
    MachineLearning,    // 机器学习
    RuleBased,          // 基于规则
}

/// 缓存策略枚举
#[derive(Debug)]
pub enum CacheStrategy {
    Lru,                // 最近最少使用
    Lfu,                // 最少频率使用
    Fifo,               // 先进先出
    Ttl,                // 基于时间的过期
}

/// 上下文选择策略
#[derive(Debug)]
pub struct ContextSelectionStrategy {
    pub similarity_threshold: f64,      // 相似度阈值
    pub max_contexts_to_select: usize,  // 最大选择上下文数
    pub weighting_factor: f64,          // 权重因子
    pub use_domain_matching: bool,      // 是否使用领域匹配
    pub use_content_similarity: bool,   // 是否使用内容相似度
}

impl Default for ContextSelectionStrategy {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.5,
            max_contexts_to_select: 5,
            weighting_factor: 0.7,
            use_domain_matching: true,
            use_content_similarity: true,
        }
    }
}

/// 策略管理器 - 管理各种策略的配置和应用
pub struct StrategyManager {
    context_management_strategy: ContextManagementStrategy,
    domain_recognition_strategy: DomainRecognitionStrategy,
    cache_strategy: CacheStrategy,
    context_selection_strategy: ContextSelectionStrategy,
}

impl StrategyManager {
    /// 创建新的策略管理器
    pub fn new() -> Self {
        Self {
            context_management_strategy: ContextManagementStrategy::Hybrid,
            domain_recognition_strategy: DomainRecognitionStrategy::KeywordMatching,
            cache_strategy: CacheStrategy::Ttl,
            context_selection_strategy: ContextSelectionStrategy::default(),
        }
    }

    /// 选择上下文管理策略
    pub fn set_context_management_strategy(&mut self, strategy: ContextManagementStrategy) {
        self.context_management_strategy = strategy;
    }

    /// 选择领域识别策略
    pub fn set_domain_recognition_strategy(&mut self, strategy: DomainRecognitionStrategy) {
        self.domain_recognition_strategy = strategy;
    }

    /// 选择缓存策略
    pub fn set_cache_strategy(&mut self, strategy: CacheStrategy) {
        self.cache_strategy = strategy;
    }

    /// 选择上下文选择策略
    pub fn set_context_selection_strategy(&mut self, strategy: ContextSelectionStrategy) {
        self.context_selection_strategy = strategy;
    }

    /// 根据策略选择上下文
    pub fn select_contexts_by_strategy(
        &self,
        available_contexts: &[Context],
        query: &str,
        query_domain: &Domain,
    ) -> Vec<Context> {
        match &self.context_management_strategy {
            ContextManagementStrategy::PriorityBased => {
                self.select_by_priority(available_contexts, query)
            }
            ContextManagementStrategy::Lru => {
                self.select_by_lru(available_contexts, query)
            }
            ContextManagementStrategy::FrequencyBased => {
                self.select_by_frequency(available_contexts, query)
            }
            ContextManagementStrategy::Hybrid => {
                self.select_by_hybrid(available_contexts, query, query_domain)
            }
        }
    }

    /// 基于优先级选择上下文
    fn select_by_priority(&self, contexts: &[Context], _query: &str) -> Vec<Context> {
        let mut contexts_with_priority = contexts.to_vec();
        contexts_with_priority.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        contexts_with_priority
            .into_iter()
            .take(self.context_selection_strategy.max_contexts_to_select)
            .collect()
    }

    /// 基于LRU选择上下文（使用时间戳作为最近使用依据）
    fn select_by_lru(&self, contexts: &[Context], _query: &str) -> Vec<Context> {
        let mut contexts_with_time = contexts.to_vec();
        // 按更新时间排序（最近更新的在前）
        contexts_with_time.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        contexts_with_time
            .into_iter()
            .take(self.context_selection_strategy.max_contexts_to_select)
            .collect()
    }

    /// 基于使用频率选择上下文（使用版本号作为频率代理）
    fn select_by_frequency(&self, contexts: &[Context], _query: &str) -> Vec<Context> {
        let mut contexts_with_version = contexts.to_vec();
        // 按版本号排序（更新的版本在前，可视为更常用）
        contexts_with_version.sort_by(|a, b| b.version.cmp(&a.version));
        
        contexts_with_version
            .into_iter()
            .take(self.context_selection_strategy.max_contexts_to_select)
            .collect()
    }

    /// 混合策略选择上下文
    fn select_by_hybrid(
        &self,
        contexts: &[Context],
        query: &str,
        query_domain: &Domain,
    ) -> Vec<Context> {
        let mut scored_contexts = Vec::new();

        for context in contexts {
            let mut score = 0.0;

            // 域匹配得分
            if self.context_selection_strategy.use_domain_matching {
                if context.domain == query_domain.to_string() {
                    score += 0.4; // 域匹配权重
                }
            }

            // 内容相似度得分
            if self.context_selection_strategy.use_content_similarity {
                let similarity = self.calculate_content_similarity(&context.context_data, query);
                score += similarity * self.context_selection_strategy.weighting_factor;
            }

            // 优先级得分
            score += (context.priority as f64) / 10.0 * 0.2; // 优先级权重

            if score >= self.context_selection_strategy.similarity_threshold {
                scored_contexts.push((context.clone(), score));
            }
        }

        // 按得分排序
        scored_contexts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 返回得分最高的上下文
        scored_contexts
            .into_iter()
            .take(self.context_selection_strategy.max_contexts_to_select)
            .map(|(context, _)| context)
            .collect()
    }

    /// 计算内容相似度（使用简化的Jaccard相似度）
    fn calculate_content_similarity(&self, content: &str, query: &str) -> f64 {
        let lower_content = content.to_lowercase();
        let lower_query = query.to_lowercase();
        let content_words: std::collections::HashSet<&str> =
            lower_content.split_whitespace().collect();
        let query_words: std::collections::HashSet<&str> =
            lower_query.split_whitespace().collect();

        let intersection = content_words.intersection(&query_words).count();
        let union = content_words.union(&query_words).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// 获取当前策略摘要
    pub fn get_strategy_summary(&self) -> StrategySummary {
        StrategySummary {
            context_management: format!("{:?}", self.context_management_strategy),
            domain_recognition: format!("{:?}", self.domain_recognition_strategy),
            cache: format!("{:?}", self.cache_strategy),
            selection: format!("{:?}", self.context_selection_strategy),
        }
    }
}

/// 策略摘要
pub struct StrategySummary {
    pub context_management: String,
    pub domain_recognition: String,
    pub cache: String,
    pub selection: String,
}

impl std::fmt::Display for StrategySummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StrategySummary {{ context_management: {}, domain_recognition: {}, cache: {}, selection: {} }}",
            self.context_management,
            self.domain_recognition,
            self.cache,
            self.selection
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_strategy_manager() {
        let mut strategy_manager = StrategyManager::new();
        
        // 创建测试上下文
        let test_contexts = vec![
            Context {
                id: uuid::Uuid::new_v4(),
                session_id: "test_session".to_string(),
                user_id: "test_user".to_string(),
                domain: "medical".to_string(),
                context_data: "Treatment for pneumonia involves antibiotics".to_string(),
                metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                expires_at: None,
                priority: 8,
                version: 2,
                tags: vec!["treatment".to_string(), "pneumonia".to_string()],
                active: true,
            },
            Context {
                id: uuid::Uuid::new_v4(),
                session_id: "test_session".to_string(),
                user_id: "test_user".to_string(),
                domain: "legal".to_string(),
                context_data: "Contract law requires offer acceptance and consideration".to_string(),
                metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                expires_at: None,
                priority: 6,
                version: 1,
                tags: vec!["contract".to_string(), "law".to_string()],
                active: true,
            },
            Context {
                id: uuid::Uuid::new_v4(),
                session_id: "test_session".to_string(),
                user_id: "test_user".to_string(),
                domain: "medical".to_string(),
                context_data: "Symptoms of flu include fever and fatigue".to_string(),
                metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                expires_at: None,
                priority: 7,
                version: 3,
                tags: vec!["symptoms".to_string(), "flu".to_string()],
                active: true,
            },
        ];

        // 测试混合策略选择
        let query = "What is the treatment for pneumonia?";
        let domain = Domain::Medical;
        let selected = strategy_manager.select_contexts_by_strategy(&test_contexts, query, &domain);
        
        assert!(!selected.is_empty());
        println!("Selected {} contexts using hybrid strategy", selected.len());

        // 测试优先级策略
        strategy_manager.set_context_management_strategy(ContextManagementStrategy::PriorityBased);
        let selected_priority = strategy_manager.select_contexts_by_strategy(&test_contexts, query, &domain);
        assert!(!selected_priority.is_empty());

        // 测试LRU策略
        strategy_manager.set_context_management_strategy(ContextManagementStrategy::Lru);
        let selected_lru = strategy_manager.select_contexts_by_strategy(&test_contexts, query, &domain);
        assert!(!selected_lru.is_empty());

        // 测试频率策略
        strategy_manager.set_context_management_strategy(ContextManagementStrategy::FrequencyBased);
        let selected_freq = strategy_manager.select_contexts_by_strategy(&test_contexts, query, &domain);
        assert!(!selected_freq.is_empty());

        // 测试策略摘要
        let summary = strategy_manager.get_strategy_summary();
        println!("{}", summary);
    }
}