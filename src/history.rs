use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MAX_HISTORY: usize = 10;

/// History entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEntry {
    pub platform: String,
    pub category: String,
    pub command: String,
}

/// History manager
pub struct HistoryManager {
    entries: Vec<HistoryEntry>,
    file_path: PathBuf,
}

impl HistoryManager {
    pub fn new() -> Self {
        let file_path = Self::config_path();
        let entries = Self::load_from_file(&file_path);
        Self { entries, file_path }
    }

    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("cmdref");
        std::fs::create_dir_all(&path).ok();
        path.push("history.json");
        path
    }

    fn load_from_file(path: &PathBuf) -> Vec<HistoryEntry> {
        if let Ok(content) = std::fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.entries) {
            let _ = std::fs::write(&self.file_path, json);
        }
    }

    /// Record a command view (deduplicate, most recent first)
    pub fn record(&mut self, platform: &str, category: &str, command: &str) {
        let entry = HistoryEntry {
            platform: platform.to_string(),
            category: category.to_string(),
            command: command.to_string(),
        };

        // Remove existing duplicate
        self.entries.retain(|e| e != &entry);

        // Insert at front
        self.entries.insert(0, entry);

        // Trim to max
        self.entries.truncate(MAX_HISTORY);

        self.save();
    }

    /// Get all history entries
    pub fn all(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Count of history entries
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}
