// Example Rust code for testing RAG code retrieval
// This file demonstrates common Rust patterns used in squid

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Configuration structure example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api_url: String,
    pub api_model: String,
    pub api_key: Option<String>,
    pub context_window: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:1234/v1".to_string(),
            api_model: "qwen2.5-coder-7b-instruct".to_string(),
            api_key: None,
            context_window: 8192,
        }
    }
}

/// Load configuration from file
pub fn load_config(path: &PathBuf) -> Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

/// Async function example
pub async fn fetch_data(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    Ok(text)
}

/// Error handling with Result
pub fn divide(a: i32, b: i32) -> Result<i32> {
    if b == 0 {
        anyhow::bail!("Division by zero");
    }
    Ok(a / b)
}

/// Option handling
pub fn find_user(id: u32, users: &[(u32, String)]) -> Option<String> {
    users.iter()
        .find(|(user_id, _)| *user_id == id)
        .map(|(_, name)| name.clone())
}

/// Iterator example
pub fn filter_even_numbers(numbers: Vec<i32>) -> Vec<i32> {
    numbers.into_iter()
        .filter(|n| n % 2 == 0)
        .collect()
}

/// Trait definition
pub trait Processor {
    fn process(&self, data: &str) -> String;
}

/// Struct implementing trait
pub struct UpperCaseProcessor;

impl Processor for UpperCaseProcessor {
    fn process(&self, data: &str) -> String {
        data.to_uppercase()
    }
}

/// Generic function
pub fn first_element<T: Clone>(items: &[T]) -> Option<T> {
    items.first().cloned()
}

/// Lifetime example
pub fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() {
        s1
    } else {
        s2
    }
}

/// Pattern matching
pub fn describe_number(n: i32) -> String {
    match n {
        0 => "zero".to_string(),
        1..=9 => "single digit".to_string(),
        10..=99 => "two digits".to_string(),
        _ => "large number".to_string(),
    }
}

/// Closure example
pub fn apply_operation<F>(x: i32, y: i32, op: F) -> i32
where
    F: Fn(i32, i32) -> i32,
{
    op(x, y)
}

/// Tests module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divide() {
        assert_eq!(divide(10, 2).unwrap(), 5);
        assert!(divide(10, 0).is_err());
    }

    #[test]
    fn test_filter_even() {
        let numbers = vec![1, 2, 3, 4, 5, 6];
        let even = filter_even_numbers(numbers);
        assert_eq!(even, vec![2, 4, 6]);
    }

    #[test]
    fn test_describe_number() {
        assert_eq!(describe_number(0), "zero");
        assert_eq!(describe_number(5), "single digit");
        assert_eq!(describe_number(42), "two digits");
    }
}

// This file helps test:
// - Code retrieval with RAG
// - Syntax highlighting
// - Function search
// - Example code discovery
