use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    #[default]
    Webdav,
    Github,
}

/// 同步凭据与设备标识，存放在 local_dir()（不参与数据同步）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    #[serde(default)]
    pub backend: BackendKind,
    // WebDAV 字段
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    // GitHub 字段
    #[serde(default)]
    pub github_token: String,
    #[serde(default)]
    pub github_repo: String,
    // 通用
    #[serde(default)]
    pub device_id: String,
}

impl SyncConfig {
    pub fn load() -> Option<Self> {
        let path = Self::config_path();
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("序列化失败: {}", e))?;
        std::fs::write(&path, json).map_err(|e| format!("写入失败: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    pub fn remove() {
        let _ = std::fs::remove_file(Self::config_path());
    }

    fn config_path() -> PathBuf {
        crate::paths::local_dir().join("sync.json")
    }

    /// 远端快照文件完整 URL（WebDAV）
    pub fn snapshot_url(&self) -> String {
        format!("{}/cmdref-sync.json", normalize_url(&self.url))
    }

    /// 生成 curl -K 配置内容（WebDAV）
    pub fn curl_config(&self) -> String {
        format!(
            "user = \"{}:{}\"\n",
            curl_config_escape(&self.username),
            curl_config_escape(&self.password)
        )
    }
}

/// URL 规范化：补 https:// 前缀、去尾部斜杠
pub fn normalize_url(raw: &str) -> String {
    let mut url = raw.trim().to_string();
    if !url.contains("://") {
        url = format!("https://{}", url);
    }
    while url.ends_with('/') {
        url.pop();
    }
    url
}

/// curl 配置文件字符串转义。
pub fn curl_config_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\u{0b}' => out.push_str("\\v"),
            _ => out.push(c),
        }
    }
    out
}

/// 生成 8 位 hex 设备标识
pub fn generate_device_id() -> String {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id() as u128;
    let seq = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128;
    let seed = nanos ^ (pid << 64) ^ (nanos >> 32) ^ seq.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut x = seed | 1;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    format!("{:08x}", (x as u32) & 0xffff_ffff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        assert_eq!(normalize_url("dav.jianguoyun.com/dav/cmdref/"), "https://dav.jianguoyun.com/dav/cmdref");
        assert_eq!(normalize_url("https://example.com/dav/"), "https://example.com/dav");
        assert_eq!(normalize_url("http://localhost:8888/"), "http://localhost:8888");
    }

    #[test]
    fn test_curl_config_escape() {
        assert_eq!(curl_config_escape(r#"a\b"c"#), r#"a\\b\"c"#);
        assert_eq!(curl_config_escape("a\nb\tc"), r"a\nb\tc");
        assert_eq!(curl_config_escape("普通密码"), "普通密码");
        assert_eq!(curl_config_escape(r"\n"), r"\\n");
    }

    #[test]
    fn test_curl_config() {
        let cfg = SyncConfig {
            backend: BackendKind::Webdav,
            url: String::new(),
            username: "user@example.com".into(),
            password: "p\"ass\\word".into(),
            github_token: String::new(),
            github_repo: String::new(),
            device_id: String::new(),
        };
        assert_eq!(
            cfg.curl_config(),
            "user = \"user@example.com:p\\\"ass\\\\word\"\n"
        );
    }

    #[test]
    fn test_device_id_format() {
        let id = generate_device_id();
        assert_eq!(id.len(), 8);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(generate_device_id(), generate_device_id());
    }

    #[test]
    fn test_backend_kind_default() {
        let json = r#"{"url":"https://example.com","username":"u","password":"p","device_id":"abc"}"#;
        let cfg: SyncConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.backend, BackendKind::Webdav);
    }
}
