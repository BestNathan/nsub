//! 模板渲染引擎 — Tera 模板 + node context + rule results
//!
//! 模板里可用的变量:
//! - `{{ nodes }}`             → Vec<NodeContext> (原始所有节点)
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

#[derive(Debug, Serialize)]
struct NodeGroupContext {
    name: String,
    nodes: Vec<NodeContext>,
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
    ///
    /// `template_name` 是相对于 templates/ 的路径。
    /// 例如: `"clash/config"` 或 `"clash/config.tpl"` 均可。
    pub fn render(
        &self,
        template_name: &str,
        nodes: Vec<NodeContext>,
        rules: RuleResults,
    ) -> Result<String, RenderError> {
        // 自动补 .tpl 后缀
        let name = if template_name.ends_with(".tpl") {
            template_name.to_string()
        } else {
            format!("{template_name}.tpl")
        };
        let mut ctx = Context::new();

        // 原始节点
        ctx.insert("nodes", &nodes);

        // dedup
        let dedup: HashMap<String, NodeGroupContext> = rules
            .dedup
            .into_iter()
            .map(|(k, v)| (k, NodeGroupContext { name: v.name, nodes: v.nodes }))
            .collect();
        ctx.insert("dedup", &dedup);

        // exclude
        let exclude: HashMap<String, NodeGroupContext> = rules
            .exclude
            .into_iter()
            .map(|(k, v)| (k, NodeGroupContext { name: v.name, nodes: v.nodes }))
            .collect();
        ctx.insert("exclude", &exclude);

        // group
        let group: HashMap<String, NodeGroupContext> = rules
            .group
            .into_iter()
            .map(|(k, v)| (k, NodeGroupContext { name: v.name, nodes: v.nodes }))
            .collect();
        ctx.insert("group", &group);

        // pipeline
        let pipeline: HashMap<String, NodeGroupContext> = rules
            .pipeline
            .into_iter()
            .map(|(k, v)| (k, NodeGroupContext { name: v.name, nodes: v.nodes }))
            .collect();
        ctx.insert("pipeline", &pipeline);

        let rendered = self.tera.render(&name, &ctx)?;
        Ok(rendered)
    }

    /// 列出所有可用模板
    pub fn list_templates(&self) -> Vec<String> {
        self.tera.get_template_names().map(String::from).collect()
    }
}
