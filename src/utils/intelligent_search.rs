use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use crate::utils::web_search::{SearchResult, WebSearchClient, WebSearchError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitHubSearchResult {
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub description: String,
    pub language: Option<String>,
    pub stars: u32,
    pub forks: u32,
}

#[derive(Debug)]
pub enum IntelligentSearchError {
    WebSearchError(WebSearchError),
    GitHubSearchError(String),
    RequestError(reqwest::Error),
    ParseError(serde_json::Error),
}

impl From<WebSearchError> for IntelligentSearchError {
    fn from(err: WebSearchError) -> Self {
        IntelligentSearchError::WebSearchError(err)
    }
}

impl From<reqwest::Error> for IntelligentSearchError {
    fn from(err: reqwest::Error) -> Self {
        IntelligentSearchError::RequestError(err)
    }
}

impl From<serde_json::Error> for IntelligentSearchError {
    fn from(err: serde_json::Error) -> Self {
        IntelligentSearchError::ParseError(err)
    }
}

pub struct IntelligentSearchClient {
    web_search_client: Option<WebSearchClient>,
    github_search_client: Option<GitHubSearchClient>,
}

impl IntelligentSearchClient {
    pub fn new() -> Result<Self, IntelligentSearchError> {
        let web_search_client = match WebSearchClient::new() {
            Ok(client) => Some(client),
            Err(e) => {
                eprintln!("Failed to initialize WebSearchClient: {:?}", e);
                None
            }
        };

        let github_search_client = match GitHubSearchClient::new() {
            Ok(client) => Some(client),
            Err(e) => {
                eprintln!("Failed to initialize GitHubSearchClient: {:?}", e);
                None
            }
        };

        Ok(Self {
            web_search_client,
            github_search_client,
        })
    }

    /// 智能搜索 - 根据查询内容自动选择合适的搜索引擎
    pub async fn intelligent_search(&self, query: &str, count: Option<u32>) -> Result<Vec<SearchResult>, IntelligentSearchError> {
        let query_type = self.classify_query(query);
        
        match query_type {
            QueryType::Code | QueryType::Technical => {
                if let Some(ref github_client) = self.github_search_client {
                    // 对技术查询使用GitHub搜索
                    let github_results = github_client.search_repositories(query, count.unwrap_or(5)).await?;
                    Ok(self.convert_github_results_to_search_results(github_results))
                } else {
                    // 如果GitHub搜索不可用，回退到普通网络搜索
                    self.fallback_search(query, count).await
                }
            },
            QueryType::General => {
                // 对一般查询使用普通网络搜索
                self.fallback_search(query, count).await
            }
        }
    }

    /// 分类查询类型
    fn classify_query(&self, query: &str) -> QueryType {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let code_indicators = [
            "rust", "python", "javascript", "java", "c++", "go", "c#", "php", "ruby", "swift", "kotlin", "scala", "r", "matlab", "sql",
            "code", "github", "git", "repository", "library", "api",
            "function", "class", "method", "variable", "syntax", "compiler", "interpreter",
            "package", "module", "dependency", "npm", "pip", "cargo", "maven", "gem", "nuget",
            "build", "test", "debug", "error", "bug", "fix", "implementation", "algorithm",
            "framework", "sdk", "ide", "editor", "vscode", "vim", "emacs", "docker", "kubernetes"
        ];

        let tech_indicators = [
            "algorithm", "data-structure", "data structure", "database", "server", "client", "api",
            "network", "security", "performance", "optimization", "framework",
            "library", "sdk", "cli", "command", "terminal", "shell", "script",
            "nosql", "cache", "microservice", "architecture"
        ];

        let mut code_score = 0;
        let mut tech_score = 0;

        // Check for exact word matches first (more precise)
        for indicator in code_indicators {
            if query_lower.contains(indicator) {
                // For single words, check if it's a whole word match to avoid "api" matching "apple"
                if indicator.split_whitespace().count() == 1 {
                    // Check if the indicator appears as a whole word
                    let indicator_lower = indicator.to_lowercase();
                    let mut word_found = false;

                    for word in &query_words {
                        // Remove common punctuation around the word
                        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
                        if clean_word == indicator_lower || clean_word.starts_with(&format!("{}.", indicator_lower)) || clean_word.starts_with(&format!("{}-", indicator_lower)) {
                            word_found = true;
                            break;
                        }
                    }

                    if word_found {
                        code_score += 2; // Higher weight for code indicators
                    }
                } else {
                    // For multi-word phrases, substring match is OK
                    if query_lower.contains(indicator) {
                        code_score += 2;
                    }
                }
            }
        }

        // Check for technical indicators
        for indicator in tech_indicators {
            if query_lower.contains(indicator) {
                // For single words, check if it's a whole word match
                if indicator.split_whitespace().count() == 1 {
                    let indicator_lower = indicator.to_lowercase();
                    let mut word_found = false;

                    for word in &query_words {
                        // Remove common punctuation around the word
                        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
                        if clean_word == indicator_lower {
                            word_found = true;
                            break;
                        }
                    }

                    if word_found {
                        tech_score += 1;
                    }
                } else {
                    // For multi-word phrases, substring match is OK
                    if query_lower.contains(indicator) {
                        tech_score += 1;
                    }
                }
            }
        }

        // Check for technical phrases that might not contain code-specific terms
        let technical_phrases = [
            "best practices", "how to", "tutorial", "guide", "documentation",
            "example", "sample", "implementation", "optimization", "configuration"
        ];

        let mut technical_phrase_score = 0;
        for phrase in technical_phrases {
            if query_lower.contains(phrase) {
                technical_phrase_score += 1;
            }
        }

        // If it's clearly code-related, return Code
        if code_score > 0 {
            QueryType::Code
        // If it has technical context but not code-specific, return Technical
        } else if tech_score + technical_phrase_score > 1 {
            QueryType::Technical
        } else {
            QueryType::General
        }
    }

    /// 回退到普通网络搜索
    async fn fallback_search(&self, query: &str, count: Option<u32>) -> Result<Vec<SearchResult>, IntelligentSearchError> {
        if let Some(ref web_client) = self.web_search_client {
            let results = web_client.search_with_relevance_scoring(query, count).await?;
            Ok(results)
        } else {
            Err(IntelligentSearchError::WebSearchError(WebSearchError::ApiKeyMissing))
        }
    }

    /// 转换GitHub搜索结果为通用搜索结果格式
    fn convert_github_results_to_search_results(&self, github_results: Vec<GitHubSearchResult>) -> Vec<SearchResult> {
        github_results.into_iter()
            .map(|gh_result| SearchResult {
                title: gh_result.name,
                url: gh_result.html_url,
                summary: format!(
                    "{} (Language: {}, Stars: {}, Forks: {})", 
                    gh_result.description,
                    gh_result.language.as_deref().unwrap_or("Unknown"),
                    gh_result.stars,
                    gh_result.forks
                ),
            })
            .collect()
    }
}

#[derive(Debug, PartialEq)]
enum QueryType {
    Code,
    Technical,
    General,
}

pub struct GitHubSearchClient {
    client: reqwest::Client,
    api_key: Option<String>,
}

impl GitHubSearchClient {
    pub fn new() -> Result<Self, WebSearchError> {
        dotenv::dotenv().ok(); // Load .env file

        // GitHub API doesn't require an API key for search, but it's recommended for higher rate limits
        let api_key = env::var("GITHUB_API_KEY").ok(); // Optional token

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
        })
    }

    /// Search GitHub repositories
    pub async fn search_repositories(&self, query: &str, count: u32) -> Result<Vec<GitHubSearchResult>, IntelligentSearchError> {
        let url = format!("https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page={}", 
                         urlencoding::encode(query), 
                         std::cmp::min(count, 30)); // GitHub API limits to 30 per page for search

        let mut request_builder = self.client.get(&url);

        // Add authorization header if token is available
        if let Some(ref token) = self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        // Add user agent as required by GitHub API
        request_builder = request_builder.header("User-Agent", "penlai-search-client");

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            return Err(IntelligentSearchError::GitHubSearchError(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        let search_response: GitHubSearchResponse = response.json().await?;
        
        let results = search_response.items
            .into_iter()
            .take(count as usize)
            .map(|item| GitHubSearchResult {
                name: item.name,
                full_name: item.full_name,
                html_url: item.html_url,
                description: item.description.unwrap_or_default(),
                language: item.language,
                stars: item.stargazers_count,
                forks: item.forks_count,
            })
            .collect();

        Ok(results)
    }
}

#[derive(Debug, Deserialize)]
struct GitHubSearchResponse {
    items: Vec<GitHubRepo>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    name: String,
    full_name: String,
    html_url: String,
    description: Option<String>,
    language: Option<String>,
    stargazers_count: u32,
    forks_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_classification() {
        let client = match IntelligentSearchClient::new() {
            Ok(c) => c,
            Err(_) => {
                // Create a dummy client for testing classification
                IntelligentSearchClient {
                    web_search_client: None,
                    github_search_client: None,
                }
            }
        };

        // Test code-related queries - these should definitely be Code or Technical
        assert!(matches!(client.classify_query("Rust async programming"), QueryType::Code | QueryType::Technical));
        assert!(matches!(client.classify_query("How to fix memory leak in C++"), QueryType::Code | QueryType::Technical));
        assert!(matches!(client.classify_query("Python machine learning tutorial"), QueryType::Code | QueryType::Technical));

        // Test technical queries
        assert!(matches!(client.classify_query("Best practices for database optimization"), QueryType::Technical | QueryType::Code));

        // Test general queries - these should be General now
        assert_eq!(client.classify_query("apple orange banana fruit"), QueryType::General);
        assert_eq!(client.classify_query("What is the weather today"), QueryType::General);
    }
}