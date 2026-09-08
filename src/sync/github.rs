use base64::Engine;

use super::config::SyncConfig;

const API_BASE: &str = "https://api.github.com";
const SYNC_FILE: &str = "cmdref-sync.json";

pub struct GithubClient {
    config: SyncConfig,
}

impl GithubClient {
    pub fn new(config: SyncConfig) -> Self {
        Self { config }
    }

    /// 验证 PAT 有效，返回用户名
    pub fn verify_token(&self) -> Result<String, String> {
        let (status, body) = self.request("GET", &format!("{}/user", API_BASE), None)?;
        if status == 200 {
            let json: serde_json::Value = serde_json::from_slice(&body)
                .map_err(|e| format!("响应解析失败: {}", e))?;
            let login = json["login"].as_str()
                .ok_or("响应缺少 login 字段")?;
            Ok(login.to_string())
        } else if status == 401 || status == 403 {
            Err("GitHub token 无效或已过期，请重新生成 Personal Access Token".to_string())
        } else {
            Err(format!("GitHub API 请求失败 (HTTP {})", status))
        }
    }

    /// 获取远端快照内容
    pub fn get_snapshot(&self) -> Result<Option<Vec<u8>>, String> {
        let url = format!(
            "{}/repos/{}/contents/{}",
            API_BASE, self.config.github_repo, SYNC_FILE
        );
        let (status, body) = self.request("GET", &url, None)?;
        match status {
            200 => {
                let json: serde_json::Value = serde_json::from_slice(&body)
                    .map_err(|e| format!("响应解析失败: {}", e))?;
                let content = json["content"].as_str()
                    .ok_or("响应缺少 content 字段")?;
                let cleaned: String = content.chars().filter(|c| !c.is_whitespace()).collect();
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(&cleaned)
                    .map_err(|e| format!("base64 解码失败: {}", e))?;
                Ok(Some(decoded))
            }
            404 => Ok(None),
            401 | 403 => Err("GitHub token 无效，请重新执行: cmdref sync login --backend github --token <PAT>".to_string()),
            _ => Err(format!("GitHub API 请求失败 (HTTP {})", status)),
        }
    }

    /// 获取文件 SHA（更新时需要）
    pub fn get_file_sha(&self) -> Result<Option<String>, String> {
        let url = format!(
            "{}/repos/{}/contents/{}",
            API_BASE, self.config.github_repo, SYNC_FILE
        );
        let (status, body) = self.request("GET", &url, None)?;
        match status {
            200 => {
                let json: serde_json::Value = serde_json::from_slice(&body)
                    .map_err(|e| format!("响应解析失败: {}", e))?;
                Ok(json["sha"].as_str().map(|s| s.to_string()))
            }
            404 => Ok(None),
            401 | 403 => Err("GitHub token 无效".to_string()),
            _ => Err(format!("GitHub API 请求失败 (HTTP {})", status)),
        }
    }

    /// 上传快照到仓库
    pub fn put_snapshot(&self, data: &[u8], sha: Option<&str>) -> Result<(), String> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);

        let mut payload = serde_json::json!({
            "message": format!("cmdref sync v{}", snapshot_version_from_data(data)),
            "content": encoded,
        });
        if let Some(sha) = sha {
            payload["sha"] = serde_json::Value::String(sha.to_string());
        }

        let body_str = serde_json::to_string(&payload)
            .map_err(|e| format!("序列化失败: {}", e))?;

        let url = format!(
            "{}/repos/{}/contents/{}",
            API_BASE, self.config.github_repo, SYNC_FILE
        );

        let tmp = format!("/tmp/cmdref-github-{}.json", std::process::id());
        std::fs::write(&tmp, &body_str).map_err(|e| format!("写临时文件失败: {}", e))?;

        let result = self.request_with_body("PUT", &url, &tmp);
        let _ = std::fs::remove_file(&tmp);

        let (status, _) = result?;
        match status {
            200 | 201 => Ok(()),
            401 | 403 => Err("GitHub token 无效，请重新登录".to_string()),
            409 => Err("文件冲突（SHA 不匹配），请先 pull 再 push".to_string()),
            422 => Err("请求格式错误，请检查仓库是否存在".to_string()),
            _ => Err(format!("上传失败 (HTTP {})", status)),
        }
    }

    /// 验证仓库可访问（存在且有读写权限）
    pub fn verify_repo_access(&self) -> Result<(), String> {
        let url = format!("{}/repos/{}", API_BASE, self.config.github_repo);
        let (status, _) = self.request("GET", &url, None)?;
        match status {
            200 => Ok(()),
            404 => Err(format!(
                "仓库 {} 不存在或无权访问\n请先在 GitHub 上创建该仓库（建议私有仓库），然后重试",
                self.config.github_repo
            )),
            401 | 403 => Err("GitHub token 无效或无权访问该仓库，请检查 PAT scope".to_string()),
            _ => Err(format!("验证仓库失败 (HTTP {})", status)),
        }
    }

    fn request(&self, method: &str, url: &str, _body: Option<&[u8]>) -> Result<(u16, Vec<u8>), String> {
        let args: Vec<String> = vec![
            "-sS".into(),
            "-X".into(), method.into(),
            "-w".into(), "%{http_code}".into(),
            "--header".into(), format!("Authorization: Bearer {}", self.config.github_token),
            "--header".into(), "Accept: application/vnd.github.v3+json".into(),
            "--header".into(), "User-Agent: cmdref".into(),
            "--connect-timeout".into(), "10".into(),
            "--max-time".into(), "60".into(),
            url.into(),
        ];

        let output = std::process::Command::new("curl")
            .args(&args)
            .output()
            .map_err(|e| format!("curl 执行失败: {}", e))?;

        parse_curl_output(&output.stdout)
    }

    fn request_with_body(&self, method: &str, url: &str, body_file: &str) -> Result<(u16, Vec<u8>), String> {
        let args: Vec<String> = vec![
            "-sS".into(),
            "-X".into(), method.into(),
            "-T".into(), body_file.into(),
            "-w".into(), "%{http_code}".into(),
            "--header".into(), format!("Authorization: Bearer {}", self.config.github_token),
            "--header".into(), "Accept: application/vnd.github.v3+json".into(),
            "--header".into(), "Content-Type: application/json".into(),
            "--header".into(), "User-Agent: cmdref".into(),
            "--connect-timeout".into(), "10".into(),
            "--max-time".into(), "60".into(),
            url.into(),
        ];

        let output = std::process::Command::new("curl")
            .args(&args)
            .output()
            .map_err(|e| format!("curl 执行失败: {}", e))?;

        parse_curl_output(&output.stdout)
    }
}

fn parse_curl_output(stdout: &[u8]) -> Result<(u16, Vec<u8>), String> {
    if stdout.len() < 3 {
        return Err(format!("curl 输出过短: {}", String::from_utf8_lossy(stdout)));
    }
    let split = stdout.len() - 3;
    let body = stdout[..split].to_vec();
    let status_str = String::from_utf8_lossy(&stdout[split..]).to_string();
    let status: u16 = status_str.trim().parse()
        .map_err(|_| format!("无法解析状态码: '{}'", status_str))?;
    Ok((status, body))
}

/// 从快照 JSON 中提取版本号用于 commit message
fn snapshot_version_from_data(data: &[u8]) -> String {
    serde_json::from_slice::<serde_json::Value>(data)
        .ok()
        .and_then(|v| v["version"].as_u64())
        .map(|v| v.to_string())
        .unwrap_or_else(|| "?".to_string())
}
