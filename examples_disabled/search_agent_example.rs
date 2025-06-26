/// Example of Search Agent integration with Master Claude
/// 
/// This example demonstrates how Master Claude integrates with the Search Agent:
/// 1. Automatic search delegation for tasks containing search keywords
/// 2. Proactive search decisions when agents are stuck or errors occur
/// 3. Research task handling and search result processing

use ccswarm::orchestrator::ProactiveDecision;
use ccswarm::agent::{Task, TaskType, Priority};

fn main() {
    println!("Search Agent Integration Examples\n");
    
    // Example 1: Tasks that trigger automatic search
    println!("1. Tasks that automatically trigger search assistance:");
    let search_triggering_tasks = vec![
        "Research best practices for React hooks implementation",
        "Find information about GraphQL subscription patterns",
        "Look up documentation for AWS Lambda cold starts",
        "Compare PostgreSQL vs MongoDB for real-time analytics",
        "Investigate error: Cannot read property 'map' of undefined",
    ];
    
    for task_desc in &search_triggering_tasks {
        println!("  ✓ {}", task_desc);
    }
    
    // Example 2: Master Claude search request flow
    println!("\n2. Master Claude Search Request Flow:");
    println!("  a) Task Analysis:");
    println!("     - Master Claude receives: 'Research React hooks best practices'");
    println!("     - Detects search keyword: 'research'");
    println!("     - Creates SearchRequest with query and context");
    println!("  b) Search Delegation:");
    println!("     - Sends Coordination message to 'search' agent");
    println!("     - Search agent executes via gemini CLI");
    println!("     - Returns SearchResponse with ranked results");
    println!("  c) Result Processing:");
    println!("     - Master Claude creates Research task for findings review");
    println!("     - Delegates to appropriate agent with search context");
    
    // Example 3: Proactive search decisions
    println!("\n3. Proactive Search Scenarios:");
    println!("  a) Agent Stuck Detection:");
    println!("     - Frontend agent stuck on 'implement OAuth flow' for 15 minutes");
    println!("     - ProactiveMaster suggests: 'Research OAuth implementation examples'");
    println!("  b) Error-Based Search:");
    println!("     - Task fails with 'Module not found: @aws-sdk/client-s3'");
    println!("     - ProactiveMaster suggests: 'Search AWS SDK v3 migration guide'");
    println!("  c) Planning Phase Research:");
    println!("     - Project in Planning phase with tech stack: [React, Node.js, PostgreSQL]");
    println!("     - ProactiveMaster suggests: 'Research best practices for each technology'");
    
    // Example 4: Search result task creation
    println!("\n4. Search Result Processing:");
    println!("  Original Query: 'React hooks best practices'");
    println!("  Search Results:");
    println!("    1. React Hooks Documentation - Official guide (95% relevance)");
    println!("    2. Best Practices for React Hooks - Blog post (88% relevance)");
    println!("    3. Common React Hooks Mistakes - Tutorial (82% relevance)");
    println!("  Generated Task:");
    println!("    - Type: Research");
    println!("    - Title: 'Review and apply search findings: React hooks best practices'");
    println!("    - Details: Includes top 3 results with snippets");
    println!("    - Duration: 15-20 minutes");
    
    // Example 5: Coordination message format
    println!("\n5. Coordination Message Format:");
    println!("  SearchRequest Message:");
    println!("    {{");
    println!("      from_agent: \"master-claude\",");
    println!("      to_agent: \"search\",");
    println!("      message_type: Custom(\"search_request\"),");
    println!("      payload: {{");
    println!("        query: \"React hooks best practices\",");
    println!("        max_results: 10,");
    println!("        context: \"Supporting task: implement-auth-123\"");
    println!("      }}");
    println!("    }}");
    
    println!("\n  SearchResponse Message:");
    println!("    {{");
    println!("      from_agent: \"search-agent-1\",");
    println!("      to_agent: \"master-claude\",");
    println!("      message_type: Custom(\"search_response\"),");
    println!("      payload: {{");
    println!("        results: [...],");
    println!("        total_results: 42,");
    println!("        query_used: \"React hooks best practices\"");
    println!("      }}");
    println!("    }}");
}