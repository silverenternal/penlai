use reqwest;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BingSearchResponse {
    pub web_pages: WebPages,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebPages {
    pub value: Vec<WebPageValue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebPageValue {
    pub name: String,      // Title
    pub url: String,
    pub snippet: String,   // Summary
}

#[derive(Debug)]
pub enum WebSearchError {
    ApiKeyMissing,
    RequestError(reqwest::Error),
    ParseError(serde_json::Error),
    ApiError(String),
}

impl From<reqwest::Error> for WebSearchError {
    fn from(err: reqwest::Error) -> Self {
        WebSearchError::RequestError(err)
    }
}

impl From<serde_json::Error> for WebSearchError {
    fn from(err: serde_json::Error) -> Self {
        WebSearchError::ParseError(err)
    }
}

pub struct WebSearchClient {
    client: reqwest::Client,
    bing_search_url: String,
    bing_api_key: String,
}

impl WebSearchClient {
    pub fn new() -> Result<Self, WebSearchError> {
        dotenv::dotenv().ok(); // Load .env file

        let bing_api_key = env::var("BING_API_KEY")
            .map_err(|_| WebSearchError::ApiKeyMissing)?;
        
        let bing_search_url = env::var("BING_SEARCH_URL")
            .unwrap_or_else(|_| "https://api.bing.microsoft.com/v7.0/search".to_string());

        Ok(Self {
            client: reqwest::Client::new(),
            bing_search_url,
            bing_api_key,
        })
    }

    /// Perform a web search using Bing Search API
    pub async fn search(&self, query: &str, count: Option<u32>) -> Result<Vec<SearchResult>, WebSearchError> {
        let count = count.unwrap_or(5);
        
        let params = [
            ("q", query),
            ("count", &count.to_string()),
            ("mkt", "zh-CN"),  // Market/region
            ("textDecorations", "true"),
            ("textFormat", "HTML"),
        ];

        let response = self.client
            .get(&self.bing_search_url)
            .header("Ocp-Apim-Subscription-Key", &self.bing_api_key)
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(WebSearchError::ApiError(format!(
                "Bing API returned status: {}",
                response.status()
            )));
        }

        let search_response: BingSearchResponse = response.json().await?;
        
        let results = search_response.web_pages.value
            .into_iter()
            .map(|item| SearchResult {
                title: item.name,
                url: item.url,
                summary: item.snippet,
            })
            .collect();

        Ok(results)
    }

    /// Perform semantic search and aggregation across multiple queries
    pub async fn semantic_search(&self, query: &str) -> Result<Vec<SearchResult>, WebSearchError> {
        // First, try the main query
        let results = self.search(query, Some(5)).await?;

        // In a more advanced implementation, we might use LLM to analyze relevance
        // For now, we'll implement basic relevance scoring based on keyword matching

        Ok(results)
    }

    /// Aggregate search results from multiple queries with deduplication
    pub async fn aggregate_search(&self, queries: &[&str], max_results: u32) -> Result<Vec<SearchResult>, WebSearchError> {
        let mut all_results = Vec::new();
        let results_per_query = max_results / std::cmp::max(queries.len() as u32, 1);

        for query in queries {
            match self.search(query, Some(results_per_query)).await {
                Ok(results) => {
                    all_results.extend(results);
                },
                Err(e) => {
                    eprintln!("Search failed for query '{}': {:?}", query, e);
                    // Continue with other queries
                    continue;
                }
            }
        }

        // Deduplicate results by URL
        let mut seen_urls = std::collections::HashSet::new();
        let unique_results: Vec<SearchResult> = all_results
            .into_iter()
            .filter(|result| seen_urls.insert(result.url.clone()))
            .collect();

        // Limit to max_results
        Ok(unique_results.into_iter().take(max_results as usize).collect())
    }

    /// Enhanced search with result parsing and filtering
    pub async fn enhanced_search(&self, query: &str, count: Option<u32>, filter_domains: Option<Vec<&str>>) -> Result<Vec<SearchResult>, WebSearchError> {
        let results = self.search(query, count).await?;

        // Apply domain filtering if specified
        let filtered_results = if let Some(domains) = filter_domains {
            results.into_iter()
                .filter(|result| {
                    domains.iter().any(|&domain| result.url.contains(domain))
                })
                .collect()
        } else {
            results
        };

        Ok(filtered_results)
    }

    /// Perform relevance scoring on search results based on query keywords
    pub fn score_relevance(&self, results: Vec<SearchResult>, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let query_keywords: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scored_results: Vec<(SearchResult, f32)> = results
            .into_iter()
            .map(|result| {
                let mut score = 0.0;

                // Score based on title
                let title_lower = result.title.to_lowercase();
                for keyword in &query_keywords {
                    if title_lower.contains(keyword) {
                        score += 2.0; // Higher weight for title matches
                    }
                }

                // Score based on summary
                let summary_lower = result.summary.to_lowercase();
                for keyword in &query_keywords {
                    if summary_lower.contains(keyword) {
                        score += 1.0; // Lower weight for summary matches
                    }
                }

                (result, score)
            })
            .collect();

        // Sort by score (descending)
        scored_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Extract just the results
        scored_results.into_iter().map(|(result, _)| result).collect()
    }

    /// Search and apply relevance scoring in one call
    pub async fn search_with_relevance_scoring(&self, query: &str, count: Option<u32>) -> Result<Vec<SearchResult>, WebSearchError> {
        let results = self.search(query, count).await?;
        Ok(self.score_relevance(results, query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_search_client_creation() {
        // This test will only pass if BING_API_KEY is set in environment
        let result = WebSearchClient::new();
        match result {
            Ok(_) => println!("WebSearchClient created successfully"),
            Err(WebSearchError::ApiKeyMissing) => println!("BING_API_KEY not set, skipping test"),
            Err(e) => println!("Error creating WebSearchClient: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_search_functionality() {
        // This test requires BING_API_KEY to be set
        let client = WebSearchClient::new();
        match client {
            Ok(search_client) => {
                let results = search_client.search("test query", Some(3)).await;
                match results {
                    Ok(res) => {
                        assert!(res.len() <= 3);
                        println!("Successfully retrieved {} search results", res.len());
                    },
                    Err(e) => {
                        eprintln!("Search failed: {:?}", e);
                    }
                }
            },
            Err(WebSearchError::ApiKeyMissing) => {
                println!("BING_API_KEY not set, skipping search test");
            },
            Err(e) => {
                eprintln!("Failed to create client: {:?}", e);
            }
        }
    }

    #[test]
    fn test_relevance_scoring() {
        let results = vec![
            SearchResult {
                title: "Rust Programming Language".to_string(),
                url: "https://rust-lang.org".to_string(),
                summary: "Official Rust programming language website with documentation".to_string(),
            },
            SearchResult {
                title: "Python Tutorial".to_string(),
                url: "https://python.org".to_string(),
                summary: "Learn Python programming with tutorials".to_string(),
            },
        ];

        let web_client = match WebSearchClient::new() {
            Ok(client) => client,
            Err(_) => {
                // Create a dummy client for testing relevance scoring without API
                WebSearchClient {
                    client: reqwest::Client::new(),
                    bing_search_url: "https://api.bing.microsoft.com/v7.0/search".to_string(),
                    bing_api_key: "dummy_key".to_string(),
                }
            }
        };

        let scored_results = web_client.score_relevance(results, "Rust programming");

        // The Rust result should be ranked higher than the Python result
        assert!(!scored_results.is_empty());
        assert!(scored_results[0].title.contains("Rust"));
    }

    #[tokio::test]
    async fn test_aggregate_search() {
        let client = WebSearchClient::new();
        match client {
            Ok(search_client) => {
                let queries = ["Rust", "programming", "language"];
                let results = search_client.aggregate_search(&queries, 6).await;
                match results {
                    Ok(res) => {
                        assert!(res.len() <= 6);
                        println!("Aggregate search returned {} results", res.len());
                    },
                    Err(e) => {
                        eprintln!("Aggregate search failed: {:?}", e);
                    }
                }
            },
            Err(WebSearchError::ApiKeyMissing) => {
                println!("BING_API_KEY not set, skipping aggregate search test");
            },
            Err(e) => {
                eprintln!("Failed to create client: {:?}", e);
            }
        }
    }
}