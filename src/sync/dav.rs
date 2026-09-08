use std::io::Write;
use std::process::{Command, Stdio};

use super::config::SyncConfig;

const USER_AGENT: &str = concat!("cmdref/", env!("CARGO_PKG_VERSION"));

/// WebDAV 请求结果
pub enum DavOutcome {
    /// 2xx 成功，携带响应体
    Success(Vec<u8>),
    /// 404：资源不存在（GET）/ 目录不存在（PROPFIND）
    NotFound,
    /// 405：MKCOL 时目录已存在，视为成功
    MethodNotAllowed,
    /// 409：PUT 时父目录不存在
    Conflict,
    /// 401/403：凭据错误或无权限
    Unauthorized,
    /// 3xx：重定向，MKCOL 时部分服务（坚果云）对已存在目录返回 301
    Redirect,
}

pub struct DavClient {
    config: SyncConfig,
}

impl DavClient {
    pub fn new(config: SyncConfig) -> Self {
        Self { config }
    }

    /// GET 远端文件
    pub fn get(&self, url: &str) -> Result<DavOutcome, String> {
        self.request("GET", url, None)
    }

    /// PUT 上传文件内容
    pub fn put(&self, url: &str, body: &[u8]) -> Result<DavOutcome, String> {
        self.request("PUT", url, Some(body))
    }

    /// PROPFIND 探测目录
    pub fn propfind(&self, url: &str) -> Result<DavOutcome, String> {
        self.request("PROPFIND", url, None)
    }

    /// MKCOL 创建目录。坚果云等服务可能要求逐级创建。
    pub fn mkcol(&self, url: &str) -> Result<DavOutcome, String> {
        self.request("MKCOL", url, None)
    }

    /// 确保远端目录存在：从 URL path 的最深层开始逐级向上尝试 MKCOL，
    /// 405（已存在）与 201（创建成功）都视为成功。
    pub fn ensure_remote_dir(&self, dir_url: &str) -> Result<(), String> {
        let dir = dir_url.trim_end_matches('/');
        let after_scheme = dir.split("://").nth(1).unwrap_or("");
        let path = match after_scheme.find('/') {
            Some(i) => &after_scheme[i..],
            None => return Ok(()),
        };

        let scheme_host = &dir[..dir.len() - path.len()];
        let mut segments: Vec<String> = Vec::new();
        let mut acc = String::new();
        for seg in path.split('/').filter(|s| !s.is_empty()) {
            acc.push('/');
            acc.push_str(seg);
            segments.push(acc.clone());
        }

        for seg in segments {
            let full = format!("{}{}", scheme_host, seg);
            match self.mkcol(&full)? {
                DavOutcome::Success(_) | DavOutcome::MethodNotAllowed | DavOutcome::Redirect => {}
                DavOutcome::NotFound => {
                    return Err(format!("创建远端目录失败: {} (404)", full));
                }
                DavOutcome::Conflict => {
                    return Err(format!("创建远端目录失败: {} (409)", full));
                }
                DavOutcome::Unauthorized => return Err(unauthorized_msg()),
            }
        }
        Ok(())
    }

    /// 底层请求：curl -K - 从 stdin 传凭据（不进 argv/ps），
    /// -w %{http_code} 拿原始状态码（不用 -f），不跟随重定向（不用 -L，防凭据泄漏）
    fn request(&self, method: &str, url: &str, body: Option<&[u8]>) -> Result<DavOutcome, String> {
        let curl_config = self.config.curl_config();

        let mut args: Vec<String> = vec![
            "-sS".to_string(),
            "-K".to_string(),
            "-".to_string(),
            "-X".to_string(),
            method.to_string(),
            "-w".to_string(),
            "%{http_code}".to_string(),
            "--connect-timeout".to_string(),
            "10".to_string(),
            "--max-time".to_string(),
            "60".to_string(),
            "--header".to_string(),
            format!("User-Agent: {}", USER_AGENT),
            url.to_string(),
        ];

        let mut tmp_file: Option<std::path::PathBuf> = None;
        if let Some(data) = body {
            let path = std::env::temp_dir().join(format!("cmdref-sync-{}", std::process::id()));
            std::fs::write(&path, data).map_err(|e| format!("写临时文件失败: {}", e))?;
            args.push("-T".to_string());
            args.push(path.to_string_lossy().to_string());
            tmp_file = Some(path);
        }

        let result = (|| {
            let mut child = Command::new("curl")
                .args(&args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("调用 curl 失败（请确认已安装）: {}", e))?;

            {
                let stdin = child.stdin.as_mut().ok_or("无法写入 curl stdin")?;
                stdin
                    .write_all(curl_config.as_bytes())
                    .map_err(|e| format!("写入 curl 配置失败: {}", e))?;
            }

            let output = child
                .wait_with_output()
                .map_err(|e| format!("等待 curl 结束失败: {}", e))?;

            if !output.stderr.is_empty() {
                let err = String::from_utf8_lossy(&output.stderr);
                if !err.trim().is_empty() {
                    return Err(format!("curl 错误: {}", err.trim()));
                }
            }

            let stdout = output.stdout;
            if stdout.len() < 3 {
                return Err(format!("curl 响应异常: {}", String::from_utf8_lossy(&stdout)));
            }
            let split = stdout.len() - 3;
            let body = stdout[..split].to_vec();
            let status_str = String::from_utf8_lossy(&stdout[split..]).to_string();
            let status: u16 = status_str
                .trim()
                .parse()
                .map_err(|_| format!("无法解析状态码: '{}'", status_str))?;

            Ok(match status {
                200..=226 => DavOutcome::Success(match method {
                    "PROPFIND" | "MKCOL" => Vec::new(),
                    _ => body,
                }),
                404 => DavOutcome::NotFound,
                405 => DavOutcome::MethodNotAllowed,
                409 => DavOutcome::Conflict,
                401 | 403 => DavOutcome::Unauthorized,
                300..=399 => DavOutcome::Redirect,
                429 => return Err("请求过于频繁（429），WebDAV 服务端限流，请稍后重试".to_string()),
                507 => return Err("远端存储空间不足（507）".to_string()),
                _ => return Err(format!("WebDAV 请求失败: HTTP {}", status)),
            })
        })();

        if let Some(path) = tmp_file {
            let _ = std::fs::remove_file(path);
        }

        result
    }
}

pub fn unauthorized_msg() -> String {
    "认证失败（401/403）：请检查 WebDAV 账号与应用密码，重新执行 cmdref sync login".to_string()
}
