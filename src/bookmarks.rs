use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Bookmark entry: platform > category > command indices
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BookmarkEntry {
    pub platform: String,
    pub category: String,
    pub command: String,
}

/// Bookmark manager
pub struct BookmarkManager {
    bookmarks: Vec<BookmarkEntry>,
    file_path: PathBuf,
}

impl BookmarkManager {
    pub fn new() -> Self {
        let file_path = Self::config_path();
        let bookmarks = Self::load_from_file(&file_path);
        Self { bookmarks, file_path }
    }

    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("cmdref");
        std::fs::create_dir_all(&path).ok();
        path.push("bookmarks.json");
        path
    }

    fn load_from_file(path: &PathBuf) -> Vec<BookmarkEntry> {
        if let Ok(content) = std::fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.bookmarks) {
            let _ = std::fs::write(&self.file_path, json);
        }
    }

    /// Check if a command is bookmarked
    pub fn is_bookmarked(&self, platform: &str, category: &str, command: &str) -> bool {
        self.bookmarks.iter().any(|b| {
            b.platform == platform && b.category == category && b.command == command
        })
    }

    /// Toggle bookmark for a command
    pub fn toggle(&mut self, platform: &str, category: &str, command: &str) -> bool {
        let entry = BookmarkEntry {
            platform: platform.to_string(),
            category: category.to_string(),
            command: command.to_string(),
        };

        if let Some(pos) = self.bookmarks.iter().position(|b| b == &entry) {
            self.bookmarks.remove(pos);
            self.save();
            false // removed
        } else {
            self.bookmarks.push(entry);
            self.save();
            true // added
        }
    }

    /// Get all bookmarks
    pub fn all(&self) -> &[BookmarkEntry] {
        &self.bookmarks
    }

    /// Count of bookmarks
    pub fn count(&self) -> usize {
        self.bookmarks.len()
    }
}
