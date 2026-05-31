use hudhudscript_runtime::providers::deepseek::DeepSeekProvider;

#[cfg(feature = "integration-tests")]
#[tokio::test]
async fn test_deepseek_call() {
    let api_key =
        std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY environment variable not set");

    let provider = DeepSeekProvider::new(api_key);

    let response = provider
        .call(
            "deepseek-chat",
            "What is the capital of Turkey?",
            Some(0.7),
            Some(100),
        )
        .await
        .expect("Failed to call DeepSeek API");

    println!("Response: {}", response.content);
    println!("Tokens used: {}", response.tokens_used);
    println!("Model: {}", response.model);

    assert!(!response.content.is_empty());
    assert!(response.tokens_used > 0);
}

#[cfg(feature = "integration-tests")]
#[tokio::test]
async fn test_deepseek_code_generation() {
    let api_key =
        std::env::var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY environment variable not set");

    let provider = DeepSeekProvider::new(api_key);

    let response = provider
        .call(
            "deepseek-coder",
            "Write a Rust function to calculate fibonacci numbers",
            Some(0.3),
            Some(500),
        )
        .await
        .expect("Failed to call DeepSeek API");

    println!("Code: {}", response.content);
    println!("Tokens used: {}", response.tokens_used);

    assert!(!response.content.is_empty());
    assert!(response.content.contains("fn") || response.content.contains("fibonacci"));
}
