//! Scratch status tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of a scratch environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchStatus {
    pub name: String,
    pub branch: String,
    pub status: String,
    pub services: HashMap<String, String>,
    pub databases: Vec<String>,
    pub url: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ScratchStatus {
    /// Create a new scratch status
    pub fn new(name: String, branch: String) -> Self {
        Self {
            name,
            branch,
            status: "unknown".to_string(),
            services: HashMap::new(),
            databases: Vec::new(),
            url: None,
            created_at: None,
        }
    }

    /// Determine overall status from service statuses
    pub fn calculate_status(&mut self) {
        if self.services.is_empty() {
            self.status = "stopped".to_string();
            return;
        }

        let all_running = self.services.values().all(|s| s == "running");
        let any_running = self.services.values().any(|s| s == "running");

        self.status = if all_running {
            "running".to_string()
        } else if any_running {
            "partial".to_string()
        } else {
            "stopped".to_string()
        };
    }
}
