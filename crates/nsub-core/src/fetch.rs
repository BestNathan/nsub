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

/// 拉取订阅原始内容
pub async fn fetch(source: &str) -> Result<String, FetchError> {
    if source == "-" {
        // 标准输入
        return Ok(std::io::read_to_string(std::io::stdin())?);
    }

    if let Ok(path) = Path::new(source).canonicalize() {
        // 本地文件
        return Ok(std::fs::read_to_string(path)?);
    }

    if source.starts_with("http://") || source.starts_with("https://") {
        // 远程 URL
        let client = reqwest::Client::new();
        let resp = client.get(source).send().await?;
        let body = resp.text().await?;
        return Ok(body);
    }

    Err(FetchError::UnsupportedScheme(source.to_string()))
}
