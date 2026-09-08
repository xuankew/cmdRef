use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::bookmarks::BookmarkEntry;
use crate::data::{Command, CommandFile};
use crate::passwords::{Password, PasswordFile};
use crate::prompts::{Prompt, PromptFile};

/// 自定义命令快照条目：冗余文件头信息（platform/category/description），
/// 使逐条目合并不依赖文件组织方式；file 仅用于回写时还原文件分组，不参与合并键
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomCommandEntry {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub file: String,
    pub platform: String,
    pub category: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub command: Command,
}

/// Prompt 快照条目
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptEntry {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub file: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// 密码快照条目
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PasswordEntry {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub file: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub value: String, // encrypted ciphertext, synced as-is
}

/// 可同步的用户数据集合
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SyncData {
    #[serde(default)]
    pub bookmarks: Vec<BookmarkEntry>,
    #[serde(default)]
    pub custom_commands: Vec<CustomCommandEntry>,
    #[serde(default)]
    pub prompts: Vec<PromptEntry>,
    #[serde(default)]
    pub passwords: Vec<PasswordEntry>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub crypto_salt: String,
}

impl SyncData {
    pub fn is_empty(&self) -> bool {
        self.bookmarks.is_empty()
            && self.custom_commands.is_empty()
            && self.prompts.is_empty()
            && self.passwords.is_empty()
    }
}

/// 远端快照文件（cmdref-sync.json）结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncSnapshot {
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub updated_at: u64,
    #[serde(default)]
    pub device_id: String,
    #[serde(default)]
    pub data: SyncData,
}

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 从数据根目录读取全部可同步数据
pub fn build_local_data(root: &Path) -> SyncData {
    let mut data = SyncData {
        bookmarks: load_bookmarks(root),
        custom_commands: load_custom_commands(root),
        prompts: load_prompts(root),
        passwords: load_passwords(root),
        crypto_salt: String::new(),
    };
    if let Some(salt) = crate::crypto::load_salt() {
        let eng = base64::engine::general_purpose::STANDARD;
        data.crypto_salt = eng.encode(&salt);
    }
    data
}

fn load_bookmarks(root: &Path) -> Vec<BookmarkEntry> {
    let path = crate::paths::bookmarks_file_in(root);
    std::fs::read_to_string(path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn load_custom_commands(root: &Path) -> Vec<CustomCommandEntry> {
    let dir = crate::paths::custom_dir_in(root);
    let mut entries = Vec::new();
    for path in sorted_yaml_files(&dir) {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let cmd_file: CommandFile = match serde_yaml::from_str(&content) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        for command in cmd_file.commands {
            entries.push(CustomCommandEntry {
                file: stem.clone(),
                platform: cmd_file.platform.clone(),
                category: cmd_file.category.clone(),
                description: cmd_file.description.clone(),
                command,
            });
        }
    }
    entries
}

fn load_prompts(root: &Path) -> Vec<PromptEntry> {
    let dir = crate::paths::prompts_dir_in(root);
    let mut entries = Vec::new();
    for path in sorted_yaml_files(&dir) {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prompt_file: PromptFile = match serde_yaml::from_str(&content) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        for prompt in prompt_file.prompts {
            entries.push(PromptEntry {
                file: stem.clone(),
                name: prompt.name,
                description: prompt.description,
                content: prompt.content,
                tags: prompt.tags,
            });
        }
    }
    entries
}

fn load_passwords(root: &Path) -> Vec<PasswordEntry> {
    let dir = crate::paths::passwords_dir_in(root);
    let mut entries = Vec::new();
    for path in sorted_yaml_files(&dir) {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let pw_file: PasswordFile = match serde_yaml::from_str(&content) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        for pw in pw_file.passwords {
            entries.push(PasswordEntry {
                file: stem.clone(),
                name: pw.name,
                description: pw.description,
                value: pw.value,
            });
        }
    }
    entries
}

fn sorted_yaml_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                    ext == "yaml" || ext == "yml"
                })
                .filter(|p| {
                    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    let lower = stem.to_lowercase();
                    !stem.starts_with('.')
                        && !lower.contains("conflict")
                        && !lower.contains("冲突")
                })
                .collect()
        })
        .unwrap_or_default();
    paths.sort();
    paths
}

/// 将合并后的数据写回数据根目录（整体重写三个来源）
pub fn apply_local_data(root: &Path, data: &SyncData) -> Result<(), String> {
    apply_bookmarks(root, &data.bookmarks)?;
    apply_custom_commands(root, &data.custom_commands)?;
    apply_prompts(root, &data.prompts)?;
    apply_passwords(root, &data.passwords)?;
    if !data.crypto_salt.is_empty() {
        crate::crypto::restore_salt_from_sync(&data.crypto_salt)?;
    }
    Ok(())
}

fn apply_bookmarks(root: &Path, bookmarks: &[BookmarkEntry]) -> Result<(), String> {
    let path = crate::paths::bookmarks_file_in(root);
    if bookmarks.is_empty() {
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| format!("删除书签文件失败: {}", e))?;
        }
        return Ok(());
    }
    let json =
        serde_json::to_string_pretty(bookmarks).map_err(|e| format!("序列化书签失败: {}", e))?;
    write_file(&path, &json)
}

fn apply_custom_commands(root: &Path, entries: &[CustomCommandEntry]) -> Result<(), String> {
    let dir = crate::paths::custom_dir_in(root);
    clear_yaml_files(&dir)?;

    if entries.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;

    let mut groups: BTreeMap<String, Vec<&CustomCommandEntry>> = BTreeMap::new();
    for e in entries {
        let stem = sanitize_stem(&e.file).unwrap_or_else(|| "my_commands".to_string());
        groups.entry(stem).or_default().push(e);
    }

    for (stem, group) in groups {
        let first = group[0];
        let cmd_file = CommandFile {
            category: first.category.clone(),
            description: first.description.clone(),
            platform: first.platform.clone(),
            commands: group.iter().map(|e| e.command.clone()).collect(),
        };
        let yaml =
            serde_yaml::to_string(&cmd_file).map_err(|e| format!("序列化命令失败: {}", e))?;
        write_file(&dir.join(format!("{}.yaml", stem)), &yaml)?;
    }
    Ok(())
}

fn apply_prompts(root: &Path, entries: &[PromptEntry]) -> Result<(), String> {
    let dir = crate::paths::prompts_dir_in(root);

    // 记住每个文件原有的 title，回写时保留
    let mut existing_titles: BTreeMap<String, String> = BTreeMap::new();
    for path in sorted_yaml_files(&dir) {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(pf) = serde_yaml::from_str::<PromptFile>(&content) {
                if !pf.title.is_empty() {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        existing_titles.insert(stem.to_string(), pf.title);
                    }
                }
            }
        }
    }

    clear_yaml_files(&dir)?;

    if entries.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;

    let mut groups: BTreeMap<String, Vec<&PromptEntry>> = BTreeMap::new();
    for e in entries {
        let stem = sanitize_stem(&e.file).unwrap_or_else(|| "my_prompts".to_string());
        groups.entry(stem).or_default().push(e);
    }

    for (stem, group) in groups {
        let title = existing_titles
            .get(&stem)
            .cloned()
            .unwrap_or_else(|| "我的 Prompts".to_string());
        let prompt_file = PromptFile {
            title,
            prompts: group
                .iter()
                .map(|e| Prompt {
                    name: e.name.clone(),
                    description: e.description.clone(),
                    content: e.content.clone(),
                    tags: e.tags.clone(),
                    source: PathBuf::new(),
                })
                .collect(),
        };
        let yaml =
            serde_yaml::to_string(&prompt_file).map_err(|e| format!("序列化 Prompt 失败: {}", e))?;
        write_file(&dir.join(format!("{}.yaml", stem)), &yaml)?;
    }
    Ok(())
}

fn apply_passwords(root: &Path, entries: &[PasswordEntry]) -> Result<(), String> {
    let dir = crate::paths::passwords_dir_in(root);

    // 记住每个文件原有的 title，回写时保留
    let mut existing_titles: BTreeMap<String, String> = BTreeMap::new();
    for path in sorted_yaml_files(&dir) {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(pf) = serde_yaml::from_str::<PasswordFile>(&content) {
                if !pf.title.is_empty() {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        existing_titles.insert(stem.to_string(), pf.title);
                    }
                }
            }
        }
    }

    clear_yaml_files(&dir)?;

    if entries.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;

    let mut groups: BTreeMap<String, Vec<&PasswordEntry>> = BTreeMap::new();
    for e in entries {
        let stem = sanitize_stem(&e.file).unwrap_or_else(|| "my_passwords".to_string());
        groups.entry(stem).or_default().push(e);
    }

    for (stem, group) in groups {
        let title = existing_titles
            .get(&stem)
            .cloned()
            .unwrap_or_else(|| "我的密码".to_string());
        let pw_file = PasswordFile {
            title,
            passwords: group
                .iter()
                .map(|e| Password {
                    name: e.name.clone(),
                    description: e.description.clone(),
                    value: e.value.clone(),
                    source: PathBuf::new(),
                })
                .collect(),
        };
        let yaml =
            serde_yaml::to_string(&pw_file).map_err(|e| format!("序列化密码失败: {}", e))?;
        write_file(&dir.join(format!("{}.yaml", stem)), &yaml)?;
    }
    Ok(())
}

fn clear_yaml_files(dir: &Path) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }
    for path in sorted_yaml_files(dir) {
        std::fs::remove_file(&path).map_err(|e| format!("清理旧文件失败: {}", e))?;
    }
    Ok(())
}

fn write_file(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    std::fs::write(path, content).map_err(|e| format!("写入文件失败: {}", e))
}

/// 远端来源的 file 字段可能包含路径分隔符等危险字符，净化为安全文件名
fn sanitize_stem(raw: &str) -> Option<String> {
    let mut out = String::new();
    for c in raw.trim().chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    let out = out.trim_matches('_').to_string();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// 合并写回前备份当前数据目录，保留最近 10 份
pub fn backup_local_data() -> Result<PathBuf, String> {
    let root = crate::paths::data_dir();
    let backup_root = crate::paths::local_dir().join("sync-backup");
    let dest = backup_root.join(now_millis().to_string());
    std::fs::create_dir_all(&dest).map_err(|e| format!("创建备份目录失败: {}", e))?;

    let bookmarks = crate::paths::bookmarks_file_in(&root);
    if bookmarks.exists() {
        std::fs::copy(&bookmarks, dest.join("bookmarks.json"))
            .map_err(|e| format!("备份书签失败: {}", e))?;
    }
    copy_dir_recursive(&crate::paths::custom_dir_in(&root), &dest.join("custom"))?;
    copy_dir_recursive(&crate::paths::prompts_dir_in(&root), &dest.join("prompts"))?;
    copy_dir_recursive(&crate::paths::passwords_dir_in(&root), &dest.join("passwords"))?;

    prune_backups(&backup_root, 10);
    Ok(dest)
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    if !src.is_dir() {
        return Ok(());
    }
    std::fs::create_dir_all(dest).map_err(|e| format!("创建备份目录失败: {}", e))?;
    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录失败: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            copy_dir_recursive(&path, &dest.join(entry.file_name()))?;
        } else {
            std::fs::copy(&path, dest.join(entry.file_name()))
                .map_err(|e| format!("备份文件失败: {}", e))?;
        }
    }
    Ok(())
}

fn prune_backups(backup_root: &Path, keep: usize) {
    let mut names: Vec<u128> = std::fs::read_dir(backup_root)
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| e.file_name().to_str()?.parse::<u128>().ok())
                .collect()
        })
        .unwrap_or_default();
    names.sort_unstable();
    names.reverse();
    for name in names.iter().skip(keep) {
        let _ = std::fs::remove_dir_all(backup_root.join(name.to_string()));
    }
}

/// 合并基线：上次同步后的远端快照（不参与用户数据同步，存本地状态目录）
pub fn load_base() -> Option<SyncSnapshot> {
    let path = crate::paths::local_dir().join("sync-base.json");
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_base(snapshot: &SyncSnapshot) -> Result<(), String> {
    let path = crate::paths::local_dir().join("sync-base.json");
    let json = serde_json::to_string_pretty(snapshot).map_err(|e| format!("序列化失败: {}", e))?;
    write_file(&path, &json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Example;

    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "cmdref-snap-test-{}-{}",
            std::process::id(),
            tag
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_data() -> SyncData {
        SyncData {
            bookmarks: vec![BookmarkEntry {
                platform: "mac".into(),
                category: "Homebrew".into(),
                command: "brew install".into(),
            }],
            custom_commands: vec![CustomCommandEntry {
                file: "my_commands".into(),
                platform: "custom".into(),
                category: "我的命令".into(),
                description: "自定义的常用命令".into(),
                command: Command {
                    name: "my-cmd".into(),
                    summary: "test".into(),
                    examples: vec![Example {
                        description: "运行".into(),
                        code: "echo hi".into(),
                        frequency: String::new(),
                        danger: String::new(),
                    }],
                    tips: vec![],
                    related: vec![],
                    tags: vec!["test".into()],
                },
            }],
            prompts: vec![PromptEntry {
                file: "my_prompts".into(),
                name: "翻译助手".into(),
                description: "翻译".into(),
                content: "翻译成英文".into(),
                tags: vec![],
            }],
            passwords: vec![],
            crypto_salt: String::new(),
        }
    }

    #[test]
    fn test_roundtrip_apply_build() {
        let root = temp_root("roundtrip");
        let data = sample_data();
        apply_local_data(&root, &data).unwrap();
        let mut loaded = build_local_data(&root);
        // crypto_salt is loaded from real local dir, not temp root — normalize for comparison
        loaded.crypto_salt = data.crypto_salt.clone();
        assert_eq!(loaded, data);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_apply_empty_removes_files() {
        let root = temp_root("empty");
        apply_local_data(&root, &sample_data()).unwrap();
        assert!(crate::paths::bookmarks_file_in(&root).exists());

        apply_local_data(&root, &SyncData::default()).unwrap();
        assert!(!crate::paths::bookmarks_file_in(&root).exists());
        assert!(crate::paths::custom_dir_in(&root).read_dir().unwrap().next().is_none());
        assert!(crate::paths::prompts_dir_in(&root).read_dir().unwrap().next().is_none());
        let pw_dir = crate::paths::passwords_dir_in(&root);
        assert!(!pw_dir.exists() || pw_dir.read_dir().unwrap().next().is_none());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_build_from_empty_dir() {
        let root = temp_root("build-empty");
        let data = build_local_data(&root);
        assert!(data.is_empty());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_sanitize_stem() {
        assert_eq!(sanitize_stem("my_commands"), Some("my_commands".into()));
        assert_eq!(sanitize_stem("a/b\\c"), Some("a_b_c".into()));
        assert_eq!(sanitize_stem(".hidden"), Some("hidden".into()));
        assert_eq!(sanitize_stem("///"), None);
        assert_eq!(sanitize_stem(""), None);
        assert_eq!(sanitize_stem("..cmdref"), Some("cmdref".into()));
    }

    #[test]
    fn test_snapshot_json_roundtrip() {
        let snap = SyncSnapshot {
            version: 3,
            updated_at: 1725768000000,
            device_id: "ab12cd34".into(),
            data: sample_data(),
        };
        let json = serde_json::to_string(&snap).unwrap();
        let parsed: SyncSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, snap);
    }
}
