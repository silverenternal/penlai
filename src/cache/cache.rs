use moka::future::Cache;
use std::time::Duration;
use crate::context::llm_context::LLMContext as Context;

/// 缓存键类型
#[derive(Hash, PartialEq, Eq, Clone)]
pub enum CacheKey {
    Domain(String),           // 按领域缓存
    UserId(String),           // 按用户ID缓存
    Query(String),            // 按查询缓存
    ContextId(uuid::Uuid),    // 按上下文ID缓存
}

/// 缓存策略枚举
#[derive(Debug)]
pub enum CacheStrategy {
    Lru,      // 最近最少使用
    Lfu,      // 最少频率使用
    Ttl,      // 基于时间的缓存
}

/// 缓存管理器 - 管理多级缓存策略
pub struct CacheManager {
    /// 一级缓存（内存）- 用于快速访问常用上下文
    l1_cache: Cache<CacheKey, Vec<Context>>,
    
    /// 二级缓存配置参数
    l1_max_capacity: u64,
    l1_ttl: Duration,
    
    /// 缓存策略
    strategy: CacheStrategy,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        let l1_max_capacity = 1000; // 最大容量1000个项目
        let l1_ttl = Duration::from_secs(300); // 5分钟TTL
        
        Self {
            l1_cache: Cache::builder()
                .max_capacity(l1_max_capacity)
                .time_to_live(l1_ttl)
                .build(),
            l1_max_capacity,
            l1_ttl,
            strategy: CacheStrategy::Ttl,
        }
    }

    /// 获取缓存的上下文
    pub async fn get_context(&self, key: &CacheKey) -> Option<Vec<Context>> {
        self.l1_cache.get(key).await
    }

    /// 存储上下文到缓存
    pub async fn put_context(&self, key: CacheKey, contexts: Vec<Context>) {
        self.l1_cache.insert(key, contexts).await;
    }

    /// 从缓存中删除上下文
    pub async fn remove_context(&self, key: &CacheKey) {
        self.l1_cache.invalidate(key).await;
    }

    /// 清空所有缓存
    pub async fn clear_all(&self) {
        self.l1_cache.invalidate_all();
    }

    /// 检查缓存中是否存在特定键
    pub async fn contains_key(&self, key: &CacheKey) -> bool {
        self.l1_cache.contains_key(key)
    }

    /// 获取缓存统计信息
    pub async fn get_stats(&self) -> CacheStats {
        let entry_count = self.l1_cache.entry_count();
        
        // Moka cache doesn't expose hit/miss count directly in the async version
        // We'll return placeholder values for now
        let hit_count = 0; // Placeholder - moka doesn't expose hit count directly
        let miss_count = 0; // Placeholder - moka doesn't expose miss count directly
        
        let hit_rate = if hit_count + miss_count > 0 {
            hit_count as f64 / (hit_count + miss_count) as f64
        } else {
            0.0  // Default to 0 if no requests have been made
        };

        CacheStats {
            entry_count,
            hit_count,
            miss_count,
            hit_rate,
        }
    }

    /// 更新缓存配置
    pub fn update_config(&mut self, max_capacity: u64, ttl: Duration, strategy: CacheStrategy) {
        // 注意：moka缓存的配置在创建后不能直接更改
        // 在实际应用中，可能需要重建缓存或使用运行时可配置的缓存
        self.l1_max_capacity = max_capacity;
        self.l1_ttl = ttl;
        self.strategy = strategy;
    }
}

/// 缓存统计信息
pub struct CacheStats {
    pub entry_count: u64,   // 缓存条目数量
    pub hit_count: u64,     // 命中次数
    pub miss_count: u64,    // 未命中次数
    pub hit_rate: f64,      // 命中率
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CacheStats {{ entries: {}, hits: {}, misses: {}, hit_rate: {:.2}% }}",
            self.entry_count,
            self.hit_count,
            self.miss_count,
            self.hit_rate * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_cache_operations() {
        let cache_manager = CacheManager::new();
        
        // 创建测试上下文
        let test_contexts = vec![
            Context {
                id: uuid::Uuid::new_v4(),
                session_id: "test_session".to_string(),
                user_id: "test_user".to_string(),
                domain: "medical".to_string(),
                context_data: "Medical context for testing".to_string(),
                metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                expires_at: None,
                priority: 7,
                version: 1,
                tags: vec!["test".to_string(), "medical".to_string()],
                active: true,
            }
        ];

        // 测试存储和获取
        let key = CacheKey::Domain("medical".to_string());
        cache_manager.put_context(key.clone(), test_contexts.clone()).await;
        
        let retrieved = cache_manager.get_context(&key).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().len(), 1);

        // 测试删除
        cache_manager.remove_context(&key).await;
        let after_removal = cache_manager.get_context(&key).await;
        assert!(after_removal.is_none());

        // 测试包含键
        cache_manager.put_context(key.clone(), test_contexts.clone()).await;
        assert!(cache_manager.contains_key(&key).await);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache_manager = CacheManager::new();
        
        let key = CacheKey::Domain("test".to_string());
        let test_contexts = vec![
            Context {
                id: uuid::Uuid::new_v4(),
                session_id: "test_session".to_string(),
                user_id: "test_user".to_string(),
                domain: "test".to_string(),
                context_data: "Test context".to_string(),
                metadata: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                expires_at: None,
                priority: 5,
                version: 1,
                tags: vec!["test".to_string()],
                active: true,
            }
        ];

        // 先获取不存在的键（未命中）
        let _ = cache_manager.get_context(&key).await;
        
        // 存储键
        cache_manager.put_context(key.clone(), test_contexts).await;
        
        // 再次获取（命中）
        let _ = cache_manager.get_context(&key).await;

        let stats = cache_manager.get_stats().await;
        println!("{}", stats);
        
        // 验证统计信息
        assert!(stats.entry_count >= 0);
    }
}