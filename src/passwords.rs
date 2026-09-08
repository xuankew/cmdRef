use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Password {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// 加密后的密文，格式: salt_b64:nonce_b64:ciphertext_b64
    pub value: String,
    #[serde(skip)]
    pub source: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PasswordFile {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    #[serde(default)]
    pub passwords: Vec<Password>,
}

pub struct PasswordStore {
    passwords: Vec<Password>,
}

impl PasswordStore {
    pub fn load() -> Self {
        let dir = crate::paths::passwords_dir();
        let mut passwords: Vec<Password> = Vec::new();

        if !dir.is_dir() {
            return Self { passwords };
        }

        let mut entries: Vec<_> = match std::fs::read_dir(&dir) {
            Ok(e) => e.flatten().collect(),
            Err(_) => return Self { passwords },
        };

        entries.sort_by_key(|a| a.file_name());

        for entry in entries {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "yaml" && ext != "yml" {
                continue;
            }

            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if file_stem.starts_with('.')
                || file_stem.to_lowercase().contains("conflict")
                || file_stem.to_lowercase().contains("冲突")
            {
                continue;
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut pw_file: PasswordFile = match serde_yaml::from_str(&content) {
                Ok(f) => f,
                Err(_) => continue,
            };

            for mut pw in pw_file.passwords.drain(..) {
                pw.source = path.clone();
                passwords.push(pw);
            }
        }

        Self { passwords }
    }

    pub fn upsert(&mut self, password: Password, replacing: Option<&str>) -> Result<usize, String> {
        let name = password.name.trim();
        if name.is_empty() {
            return Err("名称不能为空".to_string());
        }
        if password.value.trim().is_empty() {
            return Err("密码值不能为空".to_string());
        }

        if replacing.map(|old| old != name).unwrap_or(true)
            && self.passwords.iter().any(|p| p.name == name)
        {
            return Err(format!("已存在名为 '{}' 的条目", name));
        }

        let target_path = if let Some(old_name) = replacing {
            self.passwords
                .iter()
                .find(|p| p.name == old_name)
                .map(|p| p.source.clone())
                .unwrap_or_else(default_passwords_file)
        } else {
            default_passwords_file()
        };

        let mut pw_file: PasswordFile = if target_path.exists() {
            match std::fs::read_to_string(&target_path) {
                Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
                Err(e) => return Err(format!("读取文件失败: {}", e)),
            }
        } else {
            PasswordFile {
                title: "我的密码".to_string(),
                passwords: Vec::new(),
            }
        };

        if let Some(old_name) = replacing {
            pw_file.passwords.retain(|p| p.name != old_name);
        }

        let mut persisted = password.clone();
        persisted.source = PathBuf::new();
        pw_file.passwords.push(persisted);

        let yaml = match serde_yaml::to_string(&pw_file) {
            Ok(y) => y,
            Err(e) => return Err(format!("序列化失败: {}", e)),
        };

        if let Err(e) = std::fs::create_dir_all(target_path.parent().unwrap_or(&target_path)) {
            return Err(format!("创建目录失败: {}", e));
        }

        if let Err(e) = std::fs::write(&target_path, yaml) {
            return Err(format!("写入文件失败: {}", e));
        }

        *self = Self::load();

        let idx = self
            .passwords
            .iter()
            .position(|p| p.name == name)
            .unwrap_or(0);
        Ok(idx)
    }

    pub fn delete(&mut self, name: &str) -> Result<(), String> {
        let source = match self.passwords.iter().find(|p| p.name == name) {
            Some(p) => p.source.clone(),
            None => return Err(format!("未找到: {}", name)),
        };

        if !source.exists() {
            *self = Self::load();
            return Ok(());
        }

        let mut pw_file: PasswordFile = match std::fs::read_to_string(&source) {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
            Err(e) => return Err(format!("读取文件失败: {}", e)),
        };

        let before = pw_file.passwords.len();
        pw_file.passwords.retain(|p| p.name != name);

        if pw_file.passwords.is_empty() {
            let _ = std::fs::remove_file(&source);
        } else if pw_file.passwords.len() < before {
            let yaml = match serde_yaml::to_string(&pw_file) {
                Ok(y) => y,
                Err(e) => return Err(format!("序列化失败: {}", e)),
            };
            if let Err(e) = std::fs::write(&source, yaml) {
                return Err(format!("写入文件失败: {}", e));
            }
        }

        *self = Self::load();
        Ok(())
    }

    pub fn all(&self) -> &[Password] {
        &self.passwords
    }

    pub fn count(&self) -> usize {
        self.passwords.len()
    }

    pub fn get(&self, index: usize) -> Option<&Password> {
        self.passwords.get(index)
    }
}

fn default_passwords_file() -> PathBuf {
    crate::paths::passwords_dir().join("my_passwords.yaml")
}
