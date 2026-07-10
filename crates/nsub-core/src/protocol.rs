//! 协议定义加载 + URL 解析 → NodeContext
//!
//! 协议定义 TOML:
//! ```toml
//! [protocol]
//! schemes = ["vmess", "vmess1"]
//!
//! [decode]
//! userinfo = "base64 | json"
//! fragment  = "urldecode"
//! ```
//!
//! 所有 URL 部件都走 decode pipe，不只是 userinfo。

use crate::pipe::{PipeError, PipeRegistry};
use crate::types::NodeContext;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("unknown scheme: {0} — 查看 nsub skills protocols 了解如何添加协议支持")]
    UnknownScheme(String),
    #[error("load protocol config: {0}")]
    ConfigError(#[from] toml::de::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// 单个协议的定义
#[derive(Debug, Deserialize)]
pub struct ProtocolDef {
    pub protocol: ProtocolMeta,
    #[serde(default)]
    pub decode: DecodeConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProtocolMeta {
    pub schemes: Vec<String>,
}

/// decode 配置: 字段名 → pipe spec
///
/// 支持的字段名:
/// - `userinfo`
/// - `fragment`
/// - `host`
/// - `port`
/// - `query.<param>`  (如 `query.sni`)
#[derive(Debug, Default, Deserialize)]
pub struct DecodeConfig {
    #[serde(default, flatten)]
    pub pipes: HashMap<String, String>,
}

/// 协议注册表 — 从 `protocols/` 目录加载所有 TOML 文件
pub struct ProtocolRegistry {
    /// scheme → ProtocolDef
    definitions: HashMap<String, ProtocolDef>,
    pipe_registry: PipeRegistry,
}

impl ProtocolRegistry {
    /// 从目录加载所有 `*.toml` 协议定义
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, ProtocolError> {
        let mut definitions = HashMap::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "toml") {
                continue;
            }

            let content = std::fs::read_to_string(&path)?;
            let def: ProtocolDef = toml::from_str(&content)?;

            for scheme in &def.protocol.schemes {
                definitions.insert(scheme.clone(), def.clone());
            }
        }

        Ok(Self {
            definitions,
            pipe_registry: PipeRegistry::new(),
        })
    }

    /// 按 scheme 查找协议定义
    pub fn find(&self, scheme: &str) -> Result<&ProtocolDef, ProtocolError> {
        self.definitions
            .get(scheme)
            .ok_or_else(|| ProtocolError::UnknownScheme(scheme.to_string()))
    }

    /// 将一个 URL 解析为 NodeContext
    ///
    /// 步骤:
    /// 1. 解析 URL 各部件
    /// 2. 匹配 scheme → 找到协议定义
    /// 3. 对所有 decode 配置的字段执行 pipe
    pub fn parse_url(&self, raw: &str, source: &str) -> Result<NodeContext, ProtocolError> {
        let parsed = url::Url::parse(raw).map_err(|e| {
            ProtocolError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e.to_string(),
            ))
        })?;

        let scheme = parsed.scheme().to_string();
        let proto = self.find(&scheme)?;

        // url crate 对 IPv6 始终返回带方括号 [::1] 的格式（host_str 和 Host::Display 都是），
        // 但 mihomo 自己也会加括号，导致 [[...]] 双重括号连接失败。手动去掉。
        let host = parsed.host().map(|h| h.to_string()).unwrap_or_default();
        let host = host
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .unwrap_or(&host)
            .to_string();
        let port = parsed.port().unwrap_or(443);
        let fragment = percent_decode(parsed.fragment().unwrap_or(""));
        let userinfo_raw = percent_decode(parsed.username()); // note: url crate 把 userinfo 放 username

        // query 参数
        let mut query: HashMap<String, String> = HashMap::new();
        for (k, v) in parsed.query_pairs() {
            query.insert(k.to_string(), v.to_string());
        }

        // 对每个配置了 decode pipe 的字段执行转换
        let userinfo = self.apply_decode(&userinfo_raw, proto.decode.pipes.get("userinfo"))?;
        let fragment = self.apply_decode(&fragment, proto.decode.pipes.get("fragment"))?;
        let host = self.apply_decode_str(&host, proto.decode.pipes.get("host"))?;

        Ok(NodeContext {
            scheme,
            userinfo,
            host,
            port,
            query,
            fragment: fragment.as_str().map_or_else(String::new, String::from),
            raw: raw.to_string(),
            source: source.to_string(),
        })
    }

    fn apply_decode(&self, raw: &str, pipe_spec: Option<&String>) -> Result<Value, ProtocolError> {
        match pipe_spec {
            Some(spec) => Ok(self.pipe_registry.run(raw, spec)?),
            None => Ok(Value::String(raw.to_string())),
        }
    }

    fn apply_decode_str(
        &self,
        raw: &str,
        pipe_spec: Option<&String>,
    ) -> Result<String, ProtocolError> {
        match pipe_spec {
            Some(spec) => {
                let val = self.pipe_registry.run(raw, spec)?;
                Ok(match val {
                    Value::String(s) => s,
                    other => other.to_string(),
                })
            }
            None => Ok(raw.to_string()),
        }
    }
}

impl From<PipeError> for ProtocolError {
    fn from(e: PipeError) -> Self {
        ProtocolError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            e.to_string(),
        ))
    }
}

impl Clone for ProtocolDef {
    fn clone(&self) -> Self {
        Self {
            protocol: ProtocolMeta {
                schemes: self.protocol.schemes.clone(),
            },
            decode: DecodeConfig {
                pipes: self.decode.pipes.clone(),
            },
        }
    }
}

fn percent_decode(s: &str) -> String {
    url::form_urlencoded::parse(s.as_bytes())
        .map(|(k, _)| k.into_owned())
        .collect::<Vec<_>>()
        .first()
        .cloned()
        .unwrap_or_else(|| s.to_string())
}
