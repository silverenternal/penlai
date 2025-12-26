use crate::context::llm_context::{LLMContext as Context, ContextManager};
use crate::domain::domain_classifier::Domain;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 上下文加载器 - 负责根据领域动态加载相应的上下文信息
pub struct ContextLoader {
    context_manager: Arc<ContextManager>,
    domain_context_cache: Arc<RwLock<HashMap<String, Vec<Context>>>>,
}

impl ContextLoader {
    /// 创建新的上下文加载器
    pub fn new(context_manager: Arc<ContextManager>) -> Self {
        Self {
            context_manager,
            domain_context_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 为特定领域加载上下文
    pub async fn load_context_for_domain(domain: &Domain) -> Result<Vec<Context>, Box<dyn std::error::Error>> {
        // 在实际实现中，这里会从数据库、文件系统或其他存储中加载上下文
        // 为了演示目的，我们创建一些示例上下文

        let contexts = match domain {
            Domain::Medical => {
                vec![
                    Context {
                        id: uuid::Uuid::new_v4(),
                        session_id: "medical_session".to_string(),
                        user_id: "system".to_string(),
                        domain: domain.to_string(),
                        context_data: "Medical guidelines for common treatments".to_string(),
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        priority: 8,
                        version: 1,
                        tags: vec!["treatment".to_string(), "healthcare".to_string()],
                        active: true,
                    },
                    Context {
                        id: uuid::Uuid::new_v4(),
                        session_id: "medical_session".to_string(),
                        user_id: "system".to_string(),
                        domain: domain.to_string(),
                        context_data: "Symptoms and diagnosis procedures".to_string(),
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        priority: 7,
                        version: 1,
                        tags: vec!["diagnosis".to_string(), "symptoms".to_string()],
                        active: true,
                    },
                ]
            },
            Domain::Legal => {
                vec![
                    Context {
                        id: uuid::Uuid::new_v4(),
                        session_id: "legal_session".to_string(),
                        user_id: "system".to_string(),
                        domain: domain.to_string(),
                        context_data: "Legal precedents and case law".to_string(),
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        priority: 9,
                        version: 1,
                        tags: vec!["precedent".to_string(), "case".to_string()],
                        active: true,
                    },
                    Context {
                        id: uuid::Uuid::new_v4(),
                        session_id: "legal_session".to_string(),
                        user_id: "system".to_string(),
                        domain: domain.to_string(),
                        context_data: "Contract law fundamentals".to_string(),
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        priority: 8,
                        version: 1,
                        tags: vec!["contract".to_string(), "agreement".to_string()],
                        active: true,
                    },
                ]
            },
            Domain::Technical => {
                vec![
                    Context {
                        id: uuid::Uuid::new_v4(),
                        session_id: "tech_session".to_string(),
                        user_id: "system".to_string(),
                        domain: domain.to_string(),
                        context_data: "Best practices for software development".to_string(),
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        priority: 7,
                        version: 1,
                        tags: vec!["development".to_string(), "best-practices".to_string()],
                        active: true,
                    },
                    Context {
                        id: uuid::Uuid::new_v4(),
                        session_id: "tech_session".to_string(),
                        user_id: "system".to_string(),
                        domain: domain.to_string(),
                        context_data: "Algorithm design patterns".to_string(),
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        priority: 8,
                        version: 1,
                        tags: vec!["algorithm".to_string(), "design".to_string()],
                        active: true,
                    },
                ]
            },
            Domain::Education => {
                vec![
                    Context {
                        id: uuid::Uuid::new_v4(),
                        session_id: "edu_session".to_string(),
                        user_id: "system".to_string(),
                        domain: domain.to_string(),
                        context_data: "Pedagogical approaches for different age groups".to_string(),
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        priority: 7,
                        version: 1,
                        tags: vec!["pedagogy".to_string(), "teaching".to_string()],
                        active: true,
                    },
                    Context {
                        id: uuid::Uuid::new_v4(),
                        session_id: "edu_session".to_string(),
                        user_id: "system".to_string(),
                        domain: domain.to_string(),
                        context_data: "Curriculum development strategies".to_string(),
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        priority: 6,
                        version: 1,
                        tags: vec!["curriculum".to_string(), "strategy".to_string()],
                        active: true,
                    },
                ]
            },
            Domain::Finance => {
                vec![
                    Context {
                        id: uuid::Uuid::new_v4(),
                        session_id: "finance_session".to_string(),
                        user_id: "system".to_string(),
                        domain: domain.to_string(),
                        context_data: "Investment analysis techniques".to_string(),
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        priority: 8,
                        version: 1,
                        tags: vec!["investment".to_string(), "analysis".to_string()],
                        active: true,
                    },
                    Context {
                        id: uuid::Uuid::new_v4(),
                        session_id: "finance_session".to_string(),
                        user_id: "system".to_string(),
                        domain: domain.to_string(),
                        context_data: "Risk management principles".to_string(),
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        priority: 9,
                        version: 1,
                        tags: vec!["risk".to_string(), "management".to_string()],
                        active: true,
                    },
                ]
            },
            Domain::General => {
                vec![
                    Context {
                        id: uuid::Uuid::new_v4(),
                        session_id: "general_session".to_string(),
                        user_id: "system".to_string(),
                        domain: domain.to_string(),
                        context_data: "General knowledge and common facts".to_string(),
                        metadata: HashMap::new(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        expires_at: None,
                        priority: 5,
                        version: 1,
                        tags: vec!["general".to_string(), "facts".to_string()],
                        active: true,
                    },
                ]
            },
        };

        Ok(contexts)
    }

    /// 从缓存中获取领域上下文（如果存在）
    pub async fn get_cached_context_for_domain(&self, domain: &str) -> Option<Vec<Context>> {
        let cache = self.domain_context_cache.read().await;
        cache.get(domain).cloned()
    }

    /// 将领域上下文缓存
    pub async fn cache_context_for_domain(&self, domain: String, contexts: Vec<Context>) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = self.domain_context_cache.write().await;
        cache.insert(domain, contexts);
        Ok(())
    }

    /// 清除特定领域的缓存
    pub async fn clear_cache_for_domain(&self, domain: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = self.domain_context_cache.write().await;
        cache.remove(domain);
        Ok(())
    }

    /// 清除所有缓存
    pub async fn clear_all_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = self.domain_context_cache.write().await;
        cache.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::domain_classifier::Domain;

    #[tokio::test]
    async fn test_context_loading() {
        // 测试为医疗领域加载上下文
        let medical_contexts = ContextLoader::load_context_for_domain(&Domain::Medical).await.unwrap();
        assert!(!medical_contexts.is_empty());
        for context in &medical_contexts {
            assert_eq!(context.domain, "medical");
        }

        // 测试为法律领域加载上下文
        let legal_contexts = ContextLoader::load_context_for_domain(&Domain::Legal).await.unwrap();
        assert!(!legal_contexts.is_empty());
        for context in &legal_contexts {
            assert_eq!(context.domain, "legal");
        }

        // 测试为技术领域加载上下文
        let tech_contexts = ContextLoader::load_context_for_domain(&Domain::Technical).await.unwrap();
        assert!(!tech_contexts.is_empty());
        for context in &tech_contexts {
            assert_eq!(context.domain, "technical");
        }
    }
}