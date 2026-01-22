//! Scratch environment management

mod lifecycle;
mod status;
mod template;

pub use lifecycle::*;
pub use status::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a scratch environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scratch {
    pub name: String,
    pub branch: String,
    pub template: String,
    pub services: Vec<String>,
    pub databases: HashMap<String, Vec<String>>,
    pub env: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Scratch {
    /// Create a new scratch instance
    pub fn new(name: String, branch: String, template: String) -> Self {
        Self {
            name,
            branch,
            template,
            services: Vec::new(),
            databases: HashMap::new(),
            env: HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Sanitize a branch name to be used as a scratch name
    pub fn sanitize_name(branch: &str) -> String {
        let name: String = branch
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();

        // Remove leading/trailing dashes and collapse multiple dashes
        let mut result = String::new();
        let mut last_was_dash = true; // Start true to skip leading dashes

        for c in name.chars() {
            if c == '-' {
                if !last_was_dash {
                    result.push(c);
                    last_was_dash = true;
                }
            } else {
                result.push(c);
                last_was_dash = false;
            }
        }

        // Remove trailing dash
        if result.ends_with('-') {
            result.pop();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(Scratch::sanitize_name("feature/my-branch"), "feature-my-branch");
        assert_eq!(Scratch::sanitize_name("Feature/MY_Branch"), "feature-my_branch");
        assert_eq!(Scratch::sanitize_name("--test--"), "test");
        assert_eq!(Scratch::sanitize_name("hello world!"), "hello-world");
    }
}
