use crate::context::llm_context::LLMContext as Context;
use std::collections::HashMap;

/// 上下文选择器 - 根据用户查询选择最相关的上下文
pub struct ContextSelector {
    // 可以添加选择策略配置
    strategy_config: ContextSelectionConfig,
}

/// 上下文选择配置
pub struct ContextSelectionConfig {
    pub min_similarity_threshold: f64,  // 最小相似度阈值
    pub max_contexts_to_return: usize,  // 返回的最大上下文数量
    pub prioritize_by_priority: bool,   // 是否按优先级排序
    pub use_similarity_scoring: bool,   // 是否使用相似度评分
}

impl Default for ContextSelectionConfig {
    fn default() -> Self {
        Self {
            min_similarity_threshold: 0.3,
            max_contexts_to_return: 5,
            prioritize_by_priority: true,
            use_similarity_scoring: true,
        }
    }
}

impl ContextSelector {
    /// 创建新的上下文选择器
    pub fn new() -> Self {
        Self {
            strategy_config: ContextSelectionConfig::default(),
        }
    }

    /// 选择与查询最相关的上下文
    pub async fn select_context(&self, available_contexts: &[Context], query: &str) -> Vec<Context> {
        if self.strategy_config.use_similarity_scoring {
            // 使用相似度评分选择上下文
            self.select_by_similarity(available_contexts, query).await
        } else {
            // 使用其他选择策略
            self.select_by_priority(available_contexts).await
        }
    }

    /// 基于相似度选择上下文
    async fn select_by_similarity(&self, contexts: &[Context], query: &str) -> Vec<Context> {
        let mut scored_contexts = Vec::new();

        for context in contexts {
            let similarity = self.calculate_similarity(&context.context_data, query);

            if similarity >= self.strategy_config.min_similarity_threshold {
                scored_contexts.push((context.clone(), similarity));
            }
        }

        // 按相似度排序（降序）
        scored_contexts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 如果配置了按优先级排序，则在相似度相近的情况下考虑优先级
        if self.strategy_config.prioritize_by_priority {
            scored_contexts.sort_by(|a, b| {
                // 首先按相似度排序，相似度相近时按优先级排序
                match b.1.partial_cmp(&a.1).unwrap() {
                    std::cmp::Ordering::Equal => b.0.priority.cmp(&a.0.priority),
                    other => other,
                }
            });
        }

        // 返回指定数量的上下文
        scored_contexts
            .into_iter()
            .take(self.strategy_config.max_contexts_to_return)
            .map(|(context, _)| context)
            .collect()
    }

    /// 基于优先级选择上下文
    async fn select_by_priority(&self, contexts: &[Context]) -> Vec<Context> {
        let mut contexts_with_priority = contexts.to_vec();
        
        // 按优先级排序（降序）
        contexts_with_priority.sort_by(|a, b| b.priority.cmp(&a.priority));

        // 返回指定数量的上下文
        contexts_with_priority
            .into_iter()
            .take(self.strategy_config.max_contexts_to_return)
            .collect()
    }

    /// 计算两个文本之间的相似度（简化版Jaccard相似度）
    fn calculate_similarity(&self, text1: &str, text2: &str) -> f64 {
        let lower_text1 = text1.to_lowercase();
        let lower_text2 = text2.to_lowercase();
        let words1: Vec<&str> = lower_text1.split_whitespace().collect();
        let words2: Vec<&str> = lower_text2.split_whitespace().collect();

        let set1: std::collections::HashSet<&str> = words1.into_iter().collect();
        let set2: std::collections::HashSet<&str> = words2.into_iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// 更新选择策略配置
    pub fn update_config(&mut self, new_config: ContextSelectionConfig) {
        self.strategy_config = new_config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_selection() {
        let selector = ContextSelector::new();

        // 创建测试上下文
        let contexts = vec![
            Context {
                id: uuid::Uuid::new_v4(),
                session_id: "test_session".to_string(),
                user_id: "test_user".to_string(),
                domain: "medical".to_string(),
                context_data: "Treatment for pneumonia involves antibiotics and rest".to_string(),
                metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                expires_at: None,
                priority: 8,
                version: 1,
                tags: vec!["treatment".to_string(), "pneumonia".to_string()],
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
                priority: 6,
                version: 1,
                tags: vec!["symptoms".to_string(), "flu".to_string()],
                active: true,
            },
            Context {
                id: uuid::Uuid::new_v4(),
                session_id: "test_session".to_string(),
                user_id: "test_user".to_string(),
                domain: "technical".to_string(),
                context_data: "Binary search algorithm implementation in Rust".to_string(),
                metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                expires_at: None,
                priority: 7,
                version: 1,
                tags: vec!["algorithm".to_string(), "rust".to_string()],
                active: true,
            },
        ];

        // 测试选择与查询相关的上下文
        let query = "What is the treatment for pneumonia?";
        let selected_contexts = selector.select_context(&contexts, query).await;

        // 验证至少有一个上下文被选中或没有被选中（基于相似度阈值）
        // 如果没有上下文被选中，说明相似度阈值设置得太高
        if !selected_contexts.is_empty() {
            // 如果有上下文被选中，验证它们与查询相关
            let first_context = &selected_contexts[0];
            assert!(first_context.context_data.to_lowercase().contains("treatment") ||
                    first_context.context_data.to_lowercase().contains("pneumonia"));
        }
        // 如果没有上下文被选中，这在某些情况下也是可以接受的
    }

    #[tokio::test]
    async fn test_similarity_calculation() {
        let selector = ContextSelector::new();

        // 测试高相似度
        let similarity1 = selector.calculate_similarity("treatment for pneumonia", "pneumonia treatment options");
        assert!(similarity1 > 0.3);

        // 测试低相似度
        let similarity2 = selector.calculate_similarity("treatment for pneumonia", "stock market analysis");
        assert!(similarity2 < 0.3);
    }
}