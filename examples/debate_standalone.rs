// Standalone example showing debate system usage
// This demonstrates the API without needing the full server

use lemmy_debate::{
    DebateStyle, DebateStatus,
    CreateDebateRequest, CreateDebateResponse,
};

fn main() {
    println!("AI Debate System - Standalone Example");
    println!("=====================================\n");

    // Example 1: Create a debate request
    let request = CreateDebateRequest {
        post_id: 1.into(),
        ai_models: Some(vec![
            "openai/gpt-4-turbo".to_string(),
            "anthropic/claude-3-opus".to_string(),
        ]),
        debate_style: Some(DebateStyle::Casual),
        max_rounds: Some(3),
        max_tokens_per_response: None,
        custom_system_prompt: None,
        include_human_comments: Some(true),
    };

    println!("1. Create Debate Request:");
    println!("   Post ID: {}", request.post_id);
    println!("   Models: {:?}", request.ai_models);
    println!("   Style: {:?}", request.debate_style);
    println!("   Max Rounds: {:?}", request.max_rounds);

    // Example 2: Debate response
    let response = CreateDebateResponse {
        debate_id: 1,
        status: DebateStatus::Pending,
        message: "Debate created successfully".to_string(),
    };

    println!("\n2. Create Debate Response:");
    println!("   Debate ID: {}", response.debate_id);
    println!("   Status: {:?}", response.status);
    println!("   Message: {}", response.message);

    // Example 3: Debate styles
    println!("\n3. Available Debate Styles:");
    let styles = vec![
        DebateStyle::Formal,
        DebateStyle::Casual,
        DebateStyle::Adversarial,
        DebateStyle::Collaborative,
        DebateStyle::Socratic,
    ];
    for style in styles {
        println!("   - {:?}: {}", style, style.as_str());
    }

    // Example 4: Debate statuses
    println!("\n4. Debate Status Flow:");
    let statuses = vec![
        DebateStatus::Pending,
        DebateStatus::Active,
        DebateStatus::Paused,
        DebateStatus::Completed,
        DebateStatus::Failed,
    ];
    for status in statuses {
        println!("   - {:?}: {}", status, status.as_str());
    }

    println!("\n✅ All debate types work correctly!");
    println!("\nNext steps:");
    println!("1. Set up database with migrations");
    println!("2. Configure OpenRouter API key");
    println!("3. Start the server");
    println!("4. Use the API endpoints to create debates");
}
