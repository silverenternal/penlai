use penlai::context::context_management::ContextManager;
use penlai::utils::web_search::SearchResult;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize context manager with web search capability
    // Note: This requires BING_API_KEY to be set in your .env file
    let context_manager = match ContextManager::new_with_web_search() {
        Ok(manager) => {
            println!("Context manager with web search initialized successfully!");
            manager
        },
        Err(e) => {
            eprintln!("Failed to initialize context manager with web search: {:?}", e);
            println!("Creating basic context manager without web search...");
            ContextManager::new()
        }
    };

    // Example 1: Perform a direct web search
    println!("\n=== Direct Web Search ===");
    match context_manager.web_search("Rust programming latest features").await {
        Ok(results) => {
            println!("Found {} search results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", i + 1, result.title);
                println!("   URL: {}", result.url);
                println!("   Summary: {:.100}...", result.summary);
                println!();
            }
        },
        Err(e) => {
            eprintln!("Web search failed: {:?}", e);
        }
    }

    // Example 2: Create context from web search
    println!("\n=== Creating Context from Web Search ===");
    match context_manager.create_context_from_web_search(
        "最新的人工智能发展趋势 2025", 
        "technology"
    ).await {
        Ok(context) => {
            println!("Created context with ID: {}", context.id);
            println!("Domain: {}", context.domain);
            println!("Content preview: {:.200}...", context.content);
            println!("Tags: {:?}", context.tags);
        },
        Err(e) => {
            eprintln!("Failed to create context from web search: {:?}", e);
        }
    }

    // Example 3: Perform aggregate search with multiple queries
    println!("\n=== Aggregate Search ===");
    let queries = [
        "Rust web development",
        "Rust async programming", 
        "Rust performance"
    ];
    
    match context_manager.aggregate_web_search(&queries).await {
        Ok(results) => {
            println!("Aggregate search returned {} results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", i + 1, result.title);
                println!("   URL: {}", result.url);
            }
        },
        Err(e) => {
            eprintln!("Aggregate search failed: {:?}", e);
        }
    }

    // Example 4: Show how to integrate search results with AI processing
    println!("\n=== Integrating Search Results with AI ===");
    match context_manager.web_search("Rust language benefits").await {
        Ok(results) => {
            if !results.is_empty() {
                // Format results for AI consumption
                let formatted_content = format_search_results_for_ai(&results);
                println!("Formatted content for AI processing:");
                println!("{}", formatted_content);
            }
        },
        Err(e) => {
            eprintln!("Failed to get search results for AI integration: {:?}", e);
        }
    }

    Ok(())
}

fn format_search_results_for_ai(results: &[SearchResult]) -> String {
    let mut content = String::from("Web Search Results:\n\n");
    
    for (i, result) in results.iter().enumerate() {
        content.push_str(&format!(
            "Result {}:\nTitle: {}\nURL: {}\nSummary: {}\n\n",
            i + 1,
            result.title,
            result.url,
            result.summary
        ));
    }
    
    content.push_str("Please analyze these search results and provide a comprehensive summary.");
    content
}