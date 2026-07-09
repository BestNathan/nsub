use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// 一个代理节点经过解析后的上下文，直接对应模板里的 `node` 变量。
///
/// URL: scheme://userinfo@host:port?key=val&key=val#fragment
///
/// 模板里访问方式:
/// - `{{ node.scheme }}`      → "vmess"
/// - `{{ node.userinfo.ps }}` → userinfo 经 decode pipe 处理后的值
/// - `{{ node.host }}`        → "1.2.3.4"
/// - `{{ node.port }}`        → 443
/// - `{{ node.query.sni }}`   → query 参数原样
/// - `{{ node.fragment }}`    → "节点名称"
/// - `{{ node.raw }}`         → 原始 URL 字符串
#[derive(Debug, Clone, Serialize)]
pub struct NodeContext {
    pub scheme: String,
    /// 经过 decode pipe 处理后的 userinfo
    pub userinfo: Value,
    pub host: String,
    pub port: u16,
    /// query 参数，key → value（原样，不 decode）
    pub query: HashMap<String, String>,
    /// URL fragment，节点名称
    pub fragment: String,
    /// 原始 URL 字符串
    pub raw: String,
}

/// 规则引擎产出的所有结果，模板里按 name 索引。
///
/// 模板访问方式:
/// - `{{ dedup["main"].nodes }}`
/// - `{{ group["🇭🇰 香港"].nodes }}`
/// - `{{ exclude["dead"].nodes }}`
/// - `{{ pipeline["clean"].nodes }}`
#[derive(Debug, Clone, Serialize)]
pub struct RuleResults {
    pub dedup: HashMap<String, NodeGroup>,
    pub exclude: HashMap<String, NodeGroup>,
    pub group: HashMap<String, NodeGroup>,
    pub pipeline: HashMap<String, NodeGroup>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeGroup {
    pub name: String,
    pub nodes: Vec<NodeContext>,
}
