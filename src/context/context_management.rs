use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow;
use crate::utils::web_search::{WebSearchClient, SearchResult};
use crate::utils::intelligent_search::IntelligentSearchClient;

/// 上下文结构体 - 用于存储特定领域的上下文信息
#[derive(Debug, Clone)]
pub struct Context {
    pub id: Uuid,           // 上下文唯一标识符
    pub domain: String,     // 领域类型（如：医疗、法律、技术等）
    pub content: String,    // 上下文内容
    pub tags: Vec<String>,  // 标签，用于分类和检索
    pub created_at: chrono::DateTime<chrono::Utc>,  // 创建时间
    pub updated_at: chrono::DateTime<chrono::Utc>,  // 更新时间
    pub version: u32,       // 版本号
    pub priority: u8,       // 优先级（0-10）
    pub metadata: HashMap<String, String>,  // 元数据
}

/// 上下文管理器 - 管理所有上下文的存储、检索和更新
pub struct ContextManager {
    contexts: Arc<RwLock<HashMap<Uuid, Context>>>,           // 存储所有上下文
    domain_context_map: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,  // 按领域映射上下文ID
    web_search_client: Option<Arc<WebSearchClient>>,         // 可选的网络搜索客户端
    intelligent_search_client: Option<Arc<IntelligentSearchClient>>, // 可选的智能搜索客户端
}

impl ContextManager {
    /// 创建新的上下文管理器
    pub fn new() -> Self {
        // Try to initialize web search client, but don't fail if API key is not available
        let web_search_client = match WebSearchClient::new() {
            Ok(client) => Some(Arc::new(client)),
            Err(e) => {
                eprintln!("Failed to initialize WebSearchClient: {:?}", e);
                None
            }
        };

        // Try to initialize intelligent search client
        let intelligent_search_client = match IntelligentSearchClient::new() {
            Ok(client) => Some(Arc::new(client)),
            Err(e) => {
                eprintln!("Failed to initialize IntelligentSearchClient: {:?}", e);
                None
            }
        };

        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            domain_context_map: Arc::new(RwLock::new(HashMap::new())),
            web_search_client,
            intelligent_search_client,
        }
    }

    /// 创建带网络搜索功能的上下文管理器
    pub fn new_with_web_search() -> Result<Self, Box<dyn std::error::Error>> {
        let web_search_client = Arc::new(WebSearchClient::new().map_err(|e| anyhow::anyhow!("Failed to create WebSearchClient: {:?}", e))?);
        let intelligent_search_client = Arc::new(IntelligentSearchClient::new().map_err(|e| anyhow::anyhow!("Failed to create IntelligentSearchClient: {:?}", e))?);

        Ok(Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            domain_context_map: Arc::new(RwLock::new(HashMap::new())),
            web_search_client: Some(web_search_client),
            intelligent_search_client: Some(intelligent_search_client),
        })
    }

    /// 添加新的上下文
    pub async fn add_context(&self, context: Context) -> Result<(), Box<dyn std::error::Error>> {
        let mut contexts = self.contexts.write().await;
        let mut domain_map = self.domain_context_map.write().await;

        contexts.insert(context.id, context.clone());

        // 更新领域映射
        domain_map
            .entry(context.domain.clone())
            .or_insert_with(Vec::new)
            .push(context.id);

        Ok(())
    }

    /// 根据ID获取上下文
    pub async fn get_context(&self, id: Uuid) -> Option<Context> {
        let contexts = self.contexts.read().await;
        contexts.get(&id).cloned()
    }

    /// 根据领域获取上下文列表
    pub async fn get_contexts_by_domain(&self, domain: &str) -> Vec<Context> {
        let contexts = self.contexts.read().await;
        let domain_map = self.domain_context_map.read().await;

        if let Some(context_ids) = domain_map.get(domain) {
            context_ids
                .iter()
                .filter_map(|id| contexts.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 更新上下文
    pub async fn update_context(&self, context: Context) -> Result<(), Box<dyn std::error::Error>> {
        let mut contexts = self.contexts.write().await;
        contexts.insert(context.id, context);
        Ok(())
    }

    /// 删除上下文
    pub async fn remove_context(&self, id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let mut contexts = self.contexts.write().await;
        let mut domain_map = self.domain_context_map.write().await;

        if let Some(removed_context) = contexts.remove(&id) {
            // 从领域映射中删除
            if let Some(ids) = domain_map.get_mut(&removed_context.domain) {
                ids.retain(|&x| x != id);
            }
        }

        Ok(())
    }

    /// 清理过期的上下文
    pub async fn cleanup_expired_contexts(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 实现清理过期或不常用上下文的逻辑
        // 这可能涉及检查元数据中的过期日期或使用统计信息
        Ok(())
    }

    /// 使用网络搜索获取实时信息并创建上下文
    pub async fn create_context_from_web_search(&self, query: &str, domain: &str) -> Result<Context, Box<dyn std::error::Error>> {
        if let Some(ref search_client) = self.web_search_client {
            let search_results = search_client.search_with_relevance_scoring(query, Some(5)).await
                .map_err(|e| anyhow::anyhow!("Web search failed: {:?}", e))?;

            // Format search results into context content
            let content = self.format_search_results_as_context(search_results);

            let context = Context {
                id: Uuid::new_v4(),
                domain: domain.to_string(),
                content,
                tags: vec!["web-search".to_string(), "real-time".to_string()],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                version: 1,
                priority: 8, // High priority for fresh web data
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("source".to_string(), "web-search".to_string());
                    map.insert("query".to_string(), query.to_string());
                    map
                },
            };

            self.add_context(context.clone()).await?;
            Ok(context)
        } else {
            Err("Web search client not available".into())
        }
    }

    /// 执行网络搜索并返回结果
    pub async fn web_search(&self, query: &str) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        if let Some(ref search_client) = self.web_search_client {
            let results = search_client.search_with_relevance_scoring(query, Some(5)).await
                .map_err(|e| anyhow::anyhow!("Web search failed: {:?}", e))?;
            Ok(results)
        } else {
            Err("Web search client not available".into())
        }
    }

    /// 执行聚合搜索（多个查询）
    pub async fn aggregate_web_search(&self, queries: &[&str]) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        if let Some(ref search_client) = self.web_search_client {
            let results = search_client.aggregate_search(queries, 10).await
                .map_err(|e| anyhow::anyhow!("Aggregate search failed: {:?}", e))?;
            Ok(results)
        } else {
            Err("Web search client not available".into())
        }
    }

    /// 使用智能搜索获取实时信息并创建上下文
    pub async fn create_context_from_intelligent_search(&self, query: &str, domain: &str) -> Result<Context, Box<dyn std::error::Error>> {
        if let Some(ref search_client) = self.intelligent_search_client {
            let search_results = search_client.intelligent_search(query, Some(5)).await
                .map_err(|e| anyhow::anyhow!("Intelligent search failed: {:?}", e))?;

            // Format search results into context content
            let content = self.format_intelligent_search_results_as_context(search_results, query);

            let context = Context {
                id: Uuid::new_v4(),
                domain: domain.to_string(),
                content,
                tags: vec!["intelligent-search".to_string(), "real-time".to_string(), "auto-routed".to_string()],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                version: 1,
                priority: 9, // Higher priority for intelligent search
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("source".to_string(), "intelligent-search".to_string());
                    map.insert("query".to_string(), query.to_string());
                    map
                },
            };

            self.add_context(context.clone()).await?;
            Ok(context)
        } else {
            Err("Intelligent search client not available".into())
        }
    }

    /// 执行智能搜索（自动路由到合适的搜索引擎）
    pub async fn intelligent_search(&self, query: &str) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        if let Some(ref search_client) = self.intelligent_search_client {
            let results = search_client.intelligent_search(query, Some(5)).await
                .map_err(|e| anyhow::anyhow!("Intelligent search failed: {:?}", e))?;
            Ok(results)
        } else {
            Err("Intelligent search client not available".into())
        }
    }

    /// 将搜索结果格式化为上下文内容
    fn format_search_results_as_context(&self, results: Vec<SearchResult>) -> String {
        let mut content = String::new();
        content.push_str("## 网络搜索结果摘要\n\n");

        for (index, result) in results.iter().enumerate() {
            content.push_str(&format!("### 来源 {}: {}\n", index + 1, result.title));
            content.push_str(&format!("**URL**: {}\n", result.url));
            content.push_str(&format!("**摘要**: {}\n\n", result.summary));
        }

        content.push_str("\n*以上信息来源于网络搜索，时效性较强*");
        content
    }

    /// 将智能搜索结果格式化为上下文内容
    fn format_intelligent_search_results_as_context(&self, results: Vec<SearchResult>, query: &str) -> String {
        let mut content = String::new();
        content.push_str(&format!("## 智能搜索结果摘要 (查询: \"{}\")\n\n", query));

        for (index, result) in results.iter().enumerate() {
            content.push_str(&format!("### 来源 {}: {}\n", index + 1, result.title));
            content.push_str(&format!("**URL**: {}\n", result.url));
            content.push_str(&format!("**摘要**: {}\n\n", result.summary));
        }

        content.push_str("\n*以上信息来源于智能搜索（根据查询内容自动选择最合适的搜索引擎），时效性较强*");
        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_context_management() {
        let manager = ContextManager::new();
        let context = Context {
            id: Uuid::new_v4(),
            domain: "medical".to_string(),  // 医疗领域
            content: "Medical context information".to_string(),
            tags: vec!["health".to_string(), "medicine".to_string()],  // 健康、医学标签
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            version: 1,
            priority: 5,
            metadata: HashMap::new(),
        };

        manager.add_context(context.clone()).await.unwrap();
        let retrieved = manager.get_context(context.id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().domain, "medical");
    }

    #[tokio::test]
    async fn test_context_manager_with_web_search() {
        let manager = ContextManager::new();

        // Test that web search client may not be available (due to missing API key)
        if manager.web_search_client.is_some() {
            // If web search is available, test the functionality
            match manager.web_search("test query").await {
                Ok(results) => {
                    println!("Successfully got {} search results", results.len());
                    assert!(results.len() <= 5); // Should respect the default limit
                },
                Err(e) => {
                    eprintln!("Web search not available for testing: {:?}", e);
                    // This is OK - API key might not be set in test environment
                }
            }
        } else {
            println!("Web search client not available (BING_API_KEY not set)");
        }
    }

    #[tokio::test]
    async fn test_format_search_results_as_context() {
        let manager = ContextManager::new();

        let search_results = vec![
            SearchResult {
                title: "Test Result 1".to_string(),
                url: "https://example.com/1".to_string(),
                summary: "This is the first test result".to_string(),
            },
            SearchResult {
                title: "Test Result 2".to_string(),
                url: "https://example.com/2".to_string(),
                summary: "This is the second test result".to_string(),
            }
        ];

        let formatted_content = manager.format_search_results_as_context(search_results);

        assert!(formatted_content.contains("网络搜索结果摘要"));
        assert!(formatted_content.contains("来源 1"));
        assert!(formatted_content.contains("Test Result 1"));
        assert!(formatted_content.contains("https://example.com/1"));
        assert!(formatted_content.contains("时效性较强"));
    }
}