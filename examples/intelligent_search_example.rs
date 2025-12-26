use penlai::context::context_management::ContextManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize context manager with intelligent search capability
    let context_manager = match ContextManager::new_with_web_search() {
        Ok(manager) => {
            println!("Context manager with intelligent search initialized successfully!");
            manager
        },
        Err(e) => {
            eprintln!("Failed to initialize context manager with intelligent search: {:?}", e);
            println!("Creating basic context manager...");
            ContextManager::new()
        }
    };

    // Example 1: Perform a code-related search (should route to GitHub)
    println!("\n=== Code-Related Search (Rust async) ===");
    match context_manager.intelligent_search("Rust async programming examples").await {
        Ok(results) => {
            println!("Found {} results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", i + 1, result.title);
                println!("   URL: {}", result.url);
                println!("   Summary: {:.100}...", result.summary);
                println!();
            }
        },
        Err(e) => {
            eprintln!("Intelligent search failed: {:?}", e);
        }
    }

    // Example 2: Perform a general search (should route to Bing)
    println!("\n=== General Search (天气) ===");
    match context_manager.intelligent_search("今天北京天气如何").await {
        Ok(results) => {
            println!("Found {} results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", i + 1, result.title);
                println!("   URL: {}", result.url);
                println!("   Summary: {:.100}...", result.summary);
                println!();
            }
        },
        Err(e) => {
            eprintln!("Intelligent search failed: {:?}", e);
        }
    }

    // Example 3: Create context from intelligent search
    println!("\n=== Creating Context from Intelligent Search ===");
    match context_manager.create_context_from_intelligent_search(
        "machine learning frameworks 2025", 
        "technology"
    ).await {
        Ok(context) => {
            println!("Created context with ID: {}", context.id);
            println!("Domain: {}", context.domain);
            println!("Tags: {:?}", context.tags); // Should include "auto-routed" tag
            println!("Content preview: {:.200}...", context.content);
        },
        Err(e) => {
            eprintln!("Failed to create context from intelligent search: {:?}", e);
        }
    }

    // Example 4: Compare different search types
    println!("\n=== Comparing Search Types ===");
    
    // Technical query
    println!("Technical query: 'React hooks best practices'");
    match context_manager.intelligent_search("React hooks best practices").await {
        Ok(results) => {
            println!("  Intelligent search found {} results", results.len());
            if !results.is_empty() {
                println!("  First result: {}", results[0].title);
            }
        },
        Err(e) => eprintln!("  Error: {:?}", e),
    }
    
    // General query
    println!("\nGeneral query: 'healthy breakfast ideas'");
    match context_manager.intelligent_search("healthy breakfast ideas").await {
        Ok(results) => {
            println!("  Intelligent search found {} results", results.len());
            if !results.is_empty() {
                println!("  First result: {}", results[0].title);
            }
        },
        Err(e) => eprintln!("  Error: {:?}", e),
    }

    Ok(())
}