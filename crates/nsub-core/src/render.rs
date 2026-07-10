//! 模板渲染引擎 — Tera 模板 + node context + rule results
//!
//! 模板里可用的变量:
//! - `{{ nodes }}`             → Vec<RenderNode> (原始所有节点, 含 pre-render proxy)
//! - `{{ dedup["main"] }}`     → NodeGroup
//! - `{{ group["🇭🇰 香港"] }}` → NodeGroup
//! - `{{ exclude["dead"] }}`   → NodeGroup
//! - `{{ pipeline["clean"] }}` → NodeGroup

use crate::types::{NodeContext, RuleResults};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use tera::{Context, Tera};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("template error: {0}")]
    Tera(#[from] tera::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// 模板渲染器
pub struct Renderer {
    tera: Tera,
}

/// 节点的 pre-render 结果：原始字段 + 渲染好的 proxy 字符串
#[derive(Debug, Serialize)]
struct RenderNode {
    scheme: String,
    host: String,
    port: u16,
    query: HashMap<String, String>,
    fragment: String,
    raw: String,
    source: String,
    /// userinfo 作为 serde_json::Value 透传
    userinfo: serde_json::Value,
    /// 预渲染的 proxy 配置文本（如 clash YAML 片段）
    proxy: String,
}

impl From<NodeContext> for RenderNode {
    fn from(n: NodeContext) -> Self {
        Self {
            scheme: n.scheme,
            host: n.host,
            port: n.port,
            query: n.query,
            fragment: n.fragment,
            raw: n.raw,
            source: n.source,
            userinfo: n.userinfo,
            proxy: String::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct NodeGroupContext {
    name: String,
    nodes: Vec<RenderNode>,
}

impl Renderer {
    /// 从 `templates/` 目录加载所有模板
    pub fn load(template_dir: impl AsRef<Path>) -> Result<Self, RenderError> {
        let pattern = template_dir.as_ref().join("**/*.tpl");
        let pattern_str = pattern.to_string_lossy().to_string();

        let tera = Tera::new(&pattern_str)?;
        Ok(Self { tera })
    }

    /// 渲染指定模板
    pub fn render(
        &self,
        template_name: &str,
        nodes: Vec<NodeContext>,
        rules: RuleResults,
    ) -> Result<String, RenderError> {
        let name = if template_name.ends_with(".tpl") {
            template_name.to_string()
        } else {
            format!("{template_name}.tpl")
        };

        // 从 name 提取客户端类型: "clash/xxx" → "clash"
        let client = template_name.split('/').next().unwrap_or("clash");

        // 预渲染每个节点的 proxy 配置
        let render_nodes: Vec<RenderNode> = nodes
            .into_iter()
            .map(|n| {
                let proxy = self.render_proxy(client, &n);
                let mut rn = RenderNode::from(n);
                rn.proxy = proxy.unwrap_or_default();
                rn
            })
            .collect();

        let mut ctx = Context::new();
        ctx.insert("nodes", &render_nodes);

        // 辅助函数：把 NodeContext 列表转为 RenderNode 列表
        let to_render = |nodes: Vec<NodeContext>| -> Vec<RenderNode> {
            nodes
                .into_iter()
                .map(|n| {
                    let proxy = self.render_proxy(client, &n);
                    let mut rn = RenderNode::from(n);
                    rn.proxy = proxy.unwrap_or_default();
                    rn
                })
                .collect()
        };

        let dedup: HashMap<String, NodeGroupContext> = rules
            .dedup
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    NodeGroupContext {
                        name: v.name,
                        nodes: to_render(v.nodes),
                    },
                )
            })
            .collect();
        ctx.insert("dedup", &dedup);

        let exclude: HashMap<String, NodeGroupContext> = rules
            .exclude
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    NodeGroupContext {
                        name: v.name,
                        nodes: to_render(v.nodes),
                    },
                )
            })
            .collect();
        ctx.insert("exclude", &exclude);

        let group: HashMap<String, NodeGroupContext> = rules
            .group
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    NodeGroupContext {
                        name: v.name,
                        nodes: to_render(v.nodes),
                    },
                )
            })
            .collect();
        ctx.insert("group", &group);

        let pipeline: HashMap<String, NodeGroupContext> = rules
            .pipeline
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    NodeGroupContext {
                        name: v.name,
                        nodes: to_render(v.nodes),
                    },
                )
            })
            .collect();
        ctx.insert("pipeline", &pipeline);

        let rendered = self.tera.render(&name, &ctx)?;
        Ok(rendered)
    }

    /// 为单个节点预渲染 proxy 模板
    ///
    /// 自动查找 `{client}/proxy/{scheme}.tpl`，如 `clash/proxy/vmess.tpl`。
    /// 找不到则回退到 `{client}/proxy.tpl`。
    fn render_proxy(&self, client: &str, node: &NodeContext) -> Result<String, RenderError> {
        let mut ctx = Context::new();
        ctx.insert("node", node);

        // 优先按 scheme 找分文件模板
        let scheme_tpl = format!("{client}/proxy/{}.tpl", node.scheme);
        if self.tera.get_template(&scheme_tpl).is_ok() {
            return Ok(self.tera.render(&scheme_tpl, &ctx)?);
        }

        // 回退到旧的单文件模板
        let fallback = format!("{client}/proxy.tpl");
        if self.tera.get_template(&fallback).is_ok() {
            return Ok(self.tera.render(&fallback, &ctx)?);
        }

        // 都没有 → 返回注释
        Ok(format!(
            "# unknown scheme: {} — 查看 nsub skills protocols",
            node.scheme
        ))
    }

    /// 列出所有可用模板
    pub fn list_templates(&self) -> Vec<String> {
        self.tera.get_template_names().map(String::from).collect()
    }
}
