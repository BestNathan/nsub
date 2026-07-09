//! 订阅拉取模块 — 从 URL 或本地文件获取原始内容
//!
//! 订阅可以是:
//! - 远程 URL（HTTP/HTTPS）
//! - 本地文件路径
//! - 标准输入

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FetchError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported scheme: {0}")]
    UnsupportedScheme(String),
}

/// 拉取订阅原始内容，同时返回来源标签（URL 的 host 或文件名）
pub async fn fetch(source: &str) -> Result<(String, String), FetchError> {
    let label = source_label(source);

    if source == "-" {
        // 标准输入
        let content = std::io::read_to_string(std::io::stdin())?;
        return Ok((content, label));
    }

    if let Ok(path) = Path::new(source).canonicalize() {
        // 本地文件
        let content = std::fs::read_to_string(path)?;
        return Ok((content, label));
    }

    if source.starts_with("http://") || source.starts_with("https://") {
        // 远程 URL
        let client = reqwest::Client::new();
        let resp = client.get(source).send().await?;
        let body = resp.text().await?;
        // 自动检测 base64 编码的订阅内容并解码
        let body = decode_subscription_if_needed(&body);
        return Ok((body, label));
    }

    Err(FetchError::UnsupportedScheme(source.to_string()))
}

/// 从订阅源地址提取来源标签
///
/// URL → host（如 `jsjc.cfd`），文件 → 文件名（如 `test_sub.txt`）
fn source_label(source: &str) -> String {
    if source == "-" {
        return "stdin".to_string();
    }
    // URL: 提取 host
    if source.starts_with("http://") || source.starts_with("https://") {
        if let Ok(parsed) = url::Url::parse(source)
            && let Some(host) = parsed.host_str()
        {
            return host.to_string();
        }
        return source.to_string();
    }
    // 文件路径: 取文件名
    let path = std::path::Path::new(source);
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| source.to_string())
}

/// 自动检测 base64 编码的订阅内容并解码
///
/// 很多订阅服务返回 base64 编码的节点列表。如果内容不含 `://`
///（URL 特征）且看起来像 base64，则尝试解码。
fn decode_subscription_if_needed(body: &str) -> String {
    let trimmed = body.trim();

    // 如果内容已经包含 URL scheme，不需要解码
    if trimmed.contains("://") {
        return body.to_string();
    }

    // 尝试 base64 解码（去掉空白字符）
    let cleaned: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.is_empty() {
        return body.to_string();
    }

    if let Ok(decoded) = base64_decode(&cleaned) {
        // 解码后包含 URL scheme 才采用
        if decoded.contains("://") {
            return decoded;
        }
    }

    body.to_string()
}

fn base64_decode(input: &str) -> Result<String, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|e| e.to_string())
        .and_then(|bytes| String::from_utf8(bytes).map_err(|e| e.to_string()))
}
