use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 大模型上下文结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMContext {
    pub id: Uuid,
    pub session_id: String,           // 会话ID
    pub user_id: String,              // 用户ID
    pub domain: String,               // 领域（如：医疗、法律、技术等）
    pub context_data: String,         // 上下文数据
    pub metadata: HashMap<String, String>, // 元数据
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>, // 过期时间
    pub priority: u8,                 // 优先级 (0-10)
    pub version: u32,                 // 版本号
    pub tags: Vec<String>,            // 标签
    pub active: bool,                 // 是否活跃
}

/// 上下文管理器 - 企业级大模型上下文管理
pub struct ContextManager {
    /// 存储所有上下文
    contexts: Arc<RwLock<HashMap<Uuid, LLMContext>>>,
    /// 按会话ID索引的上下文
    session_contexts: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    /// 按用户ID索引的上下文
    user_contexts: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    /// 按领域索引的上下文
    domain_contexts: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    /// 并发控制信号量
    concurrency_limiter: Arc<Semaphore>,
    /// 最大并发数
    max_concurrent: usize,
    /// 上下文过期时间（秒）
    context_ttl: u64,
}

impl ContextManager {
    /// 创建新的上下文管理器
    pub fn new(max_concurrent: usize, context_ttl_seconds: u64) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            session_contexts: Arc::new(RwLock::new(HashMap::new())),
            user_contexts: Arc::new(RwLock::new(HashMap::new())),
            domain_contexts: Arc::new(RwLock::new(HashMap::new())),
            concurrency_limiter: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
            context_ttl: context_ttl_seconds,
        }
    }

    /// 创建新的上下文
    pub async fn create_context(
        &self,
        session_id: String,
        user_id: String,
        domain: String,
        context_data: String,
        priority: u8,
    ) -> Result<LLMContext, Box<dyn std::error::Error + Send + Sync>> {
        let context = LLMContext {
            id: Uuid::new_v4(),
            session_id: session_id.clone(),
            user_id: user_id.clone(),
            domain: domain.clone(),
            context_data,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::seconds(self.context_ttl as i64)),
            priority,
            version: 1,
            tags: Vec::new(),
            active: true,
        };

        // 存储上下文
        {
            let mut contexts = self.contexts.write().await;
            contexts.insert(context.id, context.clone());
        }

        // 更新索引
        self.update_indexes(context.clone()).await;

        Ok(context)
    }

    /// 获取上下文
    pub async fn get_context(&self, context_id: Uuid) -> Option<LLMContext> {
        let contexts = self.contexts.read().await;
        if let Some(context) = contexts.get(&context_id) {
            // 检查是否过期
            if let Some(expires_at) = context.expires_at {
                if Utc::now() > expires_at {
                    // 上下文已过期，返回None
                    return None;
                }
            }
            Some(context.clone())
        } else {
            None
        }
    }

    /// 获取会话的所有上下文
    pub async fn get_session_contexts(&self, session_id: &str) -> Vec<LLMContext> {
        let session_contexts = self.session_contexts.read().await;
        if let Some(context_ids) = session_contexts.get(session_id) {
            let contexts = self.contexts.read().await;
            context_ids
                .iter()
                .filter_map(|id| {
                    contexts.get(id).cloned().and_then(|ctx| {
                        // 检查是否过期
                        if let Some(expires_at) = ctx.expires_at {
                            if Utc::now() > expires_at {
                                return None;
                            }
                        }
                        Some(ctx)
                    })
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取用户的所有上下文
    pub async fn get_user_contexts(&self, user_id: &str) -> Vec<LLMContext> {
        let user_contexts = self.user_contexts.read().await;
        if let Some(context_ids) = user_contexts.get(user_id) {
            let contexts = self.contexts.read().await;
            context_ids
                .iter()
                .filter_map(|id| {
                    contexts.get(id).cloned().and_then(|ctx| {
                        // 检查是否过期
                        if let Some(expires_at) = ctx.expires_at {
                            if Utc::now() > expires_at {
                                return None;
                            }
                        }
                        Some(ctx)
                    })
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取特定领域的上下文
    pub async fn get_domain_contexts(&self, domain: &str) -> Vec<LLMContext> {
        let domain_contexts = self.domain_contexts.read().await;
        if let Some(context_ids) = domain_contexts.get(domain) {
            let contexts = self.contexts.read().await;
            context_ids
                .iter()
                .filter_map(|id| {
                    contexts.get(id).cloned().and_then(|ctx| {
                        // 检查是否过期
                        if let Some(expires_at) = ctx.expires_at {
                            if Utc::now() > expires_at {
                                return None;
                            }
                        }
                        Some(ctx)
                    })
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 更新上下文
    pub async fn update_context(
        &self,
        context_id: Uuid,
        context_data: Option<String>,
        metadata: Option<HashMap<String, String>>,
        priority: Option<u8>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut contexts = self.contexts.write().await;
        if let Some(mut context) = contexts.get_mut(&context_id) {
            if let Some(data) = context_data {
                context.context_data = data;
            }
            if let Some(meta) = metadata {
                context.metadata = meta;
            }
            if let Some(pri) = priority {
                context.priority = pri;
            }
            context.updated_at = Utc::now();
            context.version += 1;

            // 更新索引
            self.update_indexes(context.clone()).await;
            Ok(())
        } else {
            Err("Context not found".into())
        }
    }

    /// 删除上下文
    pub async fn delete_context(&self, context_id: Uuid) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.remove(&context_id) {
            // 从索引中移除
            self.remove_from_indexes(context).await;
            Ok(())
        } else {
            Err("Context not found".into())
        }
    }

    /// 清理过期的上下文
    pub async fn cleanup_expired_contexts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();
        let mut contexts = self.contexts.write().await;
        let expired_ids: Vec<Uuid> = contexts
            .iter()
            .filter_map(|(id, ctx)| {
                if let Some(expires_at) = ctx.expires_at {
                    if now > expires_at {
                        Some(*id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        for id in expired_ids {
            if let Some(context) = contexts.remove(&id) {
                self.remove_from_indexes(context).await;
            }
        }

        Ok(())
    }

    /// 更新索引
    async fn update_indexes(&self, context: LLMContext) {
        // 更新会话索引
        {
            let mut session_contexts = self.session_contexts.write().await;
            session_contexts
                .entry(context.session_id.clone())
                .or_insert_with(Vec::new)
                .push(context.id);
        }

        // 更新用户索引
        {
            let mut user_contexts = self.user_contexts.write().await;
            user_contexts
                .entry(context.user_id.clone())
                .or_insert_with(Vec::new)
                .push(context.id);
        }

        // 更新领域索引
        {
            let mut domain_contexts = self.domain_contexts.write().await;
            domain_contexts
                .entry(context.domain.clone())
                .or_insert_with(Vec::new)
                .push(context.id);
        }
    }

    /// 从索引中移除
    async fn remove_from_indexes(&self, context: LLMContext) {
        // 从会话索引中移除
        {
            let mut session_contexts = self.session_contexts.write().await;
            if let Some(ids) = session_contexts.get_mut(&context.session_id) {
                ids.retain(|id| *id != context.id);
            }
        }

        // 从用户索引中移除
        {
            let mut user_contexts = self.user_contexts.write().await;
            if let Some(ids) = user_contexts.get_mut(&context.user_id) {
                ids.retain(|id| *id != context.id);
            }
        }

        // 从领域索引中移除
        {
            let mut domain_contexts = self.domain_contexts.write().await;
            if let Some(ids) = domain_contexts.get_mut(&context.domain) {
                ids.retain(|id| *id != context.id);
            }
        }
    }

    /// 获取并发许可
    pub async fn acquire_concurrent_permit(&self) -> tokio::sync::SemaphorePermit {
        self.concurrency_limiter.acquire().await.unwrap()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> ContextManagerStats {
        let contexts = self.contexts.read().await;
        let available_permits = self.concurrency_limiter.available_permits();

        ContextManagerStats {
            total_contexts: contexts.len(),
            max_concurrent: self.max_concurrent,
            available_permits,
        }
    }
}

/// 上下文管理器统计信息
#[derive(Debug)]
pub struct ContextManagerStats {
    pub total_contexts: usize,
    pub max_concurrent: usize,
    pub available_permits: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_manager() {
        let manager = ContextManager::new(10, 3600); // 10并发，1小时TTL

        // 创建上下文
        let context = manager
            .create_context(
                "session1".to_string(),
                "user1".to_string(),
                "medical".to_string(),
                "Medical context data".to_string(),
                8,
            )
            .await
            .unwrap();

        assert_eq!(context.domain, "medical");
        assert_eq!(context.user_id, "user1");

        // 获取上下文
        let retrieved = manager.get_context(context.id).await;
        assert!(retrieved.is_some());

        // 获取会话上下文
        let session_contexts = manager.get_session_contexts("session1").await;
        assert_eq!(session_contexts.len(), 1);

        // 获取用户上下文
        let user_contexts = manager.get_user_contexts("user1").await;
        assert_eq!(user_contexts.len(), 1);

        // 获取领域上下文
        let domain_contexts = manager.get_domain_contexts("medical").await;
        assert_eq!(domain_contexts.len(), 1);

        // 更新上下文
        manager
            .update_context(
                context.id,
                Some("Updated context data".to_string()),
                None,
                Some(9),
            )
            .await
            .unwrap();

        let updated = manager.get_context(context.id).await.unwrap();
        assert_eq!(updated.context_data, "Updated context data");
        assert_eq!(updated.priority, 9);

        // 获取统计信息
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_contexts, 1);
        assert_eq!(stats.max_concurrent, 10);
    }
}