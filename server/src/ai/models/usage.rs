//! Token usage tracking models

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt: u32,
    pub completion: u32,
    pub total: u32,
}

impl TokenUsage {
    /// Create a new token usage record
    #[must_use]
    pub fn new(prompt: u32, completion: u32) -> Self {
        Self {
            prompt,
            completion,
            total: prompt + completion,
        }
    }
}
