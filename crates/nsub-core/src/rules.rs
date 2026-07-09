//! 规则引擎 — dedup / exclude / group / pipeline
//!
//! 每条规则独立计算，不修改原始 nodes。
//! 规则按 name 索引，模板里通过 `dedup["main"].nodes` 访问。
//!
//! 语义:
//! - 规则内字段 = AND（都满足才命中）
//! - 多条规则   = OR（独立计算）
//! - group      = 从上到下第一节匹配即归属
//! - dedup      = 命中的节点，按规则的字段值分组，每组保留第一个
//! - pipeline   = 串联多个规则名称

use crate::types::{NodeContext, NodeGroup, RuleResults};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

/// 规则配置文件 `rules.toml` 的顶层结构
#[derive(Debug, Deserialize)]
pub struct RulesConfig {
    #[serde(default)]
    pub dedup: Vec<RuleEntry>,
    #[serde(default)]
    pub exclude: Vec<RuleEntry>,
    #[serde(default)]
    pub group: Vec<GroupEntry>,
    #[serde(default)]
    pub pipeline: Vec<PipelineEntry>,
}

/// dedup / exclude 规则条目
#[derive(Debug, Deserialize)]
pub struct RuleEntry {
    pub name: String,
    #[serde(flatten)]
    pub fields: HashMap<String, String>,
}

/// group 规则条目（结构同 RuleEntry，语义独立）
#[derive(Debug, Deserialize)]
pub struct GroupEntry {
    pub name: String,
    #[serde(flatten)]
    pub fields: HashMap<String, String>,
}

/// pipeline 定义 — 串联已命名的规则
#[derive(Debug, Deserialize)]
pub struct PipelineEntry {
    pub name: String,
    pub steps: Vec<String>, // ["group.HK", "dedup.main", "exclude.dead"]
}

/// 规则引擎
pub struct RuleEngine {
    config: RulesConfig,
}

impl RuleEngine {
    pub fn from_config(config: RulesConfig) -> Self {
        Self { config }
    }

    /// 对所有 nodes 执行全部规则，返回独立索引的 RuleResults
    pub fn run(&self, nodes: &[NodeContext]) -> RuleResults {
        let mut results = RuleResults {
            dedup: HashMap::new(),
            exclude: HashMap::new(),
            group: HashMap::new(),
            pipeline: HashMap::new(),
        };

        // ── dedup ──
        for rule in &self.config.dedup {
            let matched: Vec<&NodeContext> =
                nodes.iter().filter(|n| Self::match_rule(n, &rule.fields)).collect();
            let deduped = Self::apply_dedup(&matched, &rule.fields);
            results.dedup.insert(rule.name.clone(), NodeGroup {
                name: rule.name.clone(),
                nodes: deduped,
            });
        }

        // ── exclude ──
        for rule in &self.config.exclude {
            let matched: Vec<NodeContext> =
                nodes.iter().filter(|n| Self::match_rule(n, &rule.fields)).cloned().collect();
            results.exclude.insert(rule.name.clone(), NodeGroup {
                name: rule.name.clone(),
                nodes: matched,
            });
        }

        // ── group ── (先收集所有 exclude 的节点 key，分组时跳过)
        let excluded_keys: std::collections::HashSet<String> = results
            .exclude
            .values()
            .flat_map(|g| g.nodes.iter().map(|n| Self::node_key(n)))
            .collect();

        let mut assigned: Vec<bool> = vec![false; nodes.len()];
        for g in &self.config.group {
            let is_catch_all = g.fields.is_empty();

            let mut member_indices: Vec<usize> = Vec::new();
            for (i, n) in nodes.iter().enumerate() {
                if assigned[i] {
                    continue;
                }
                // 跳过被 exclude 命中的节点
                if excluded_keys.contains(&Self::node_key(n)) {
                    continue;
                }
                if is_catch_all || Self::match_rule(n, &g.fields) {
                    member_indices.push(i);
                    assigned[i] = true;
                }
            }

            let members: Vec<NodeContext> = member_indices
                .iter()
                .map(|&i| nodes[i].clone())
                .collect();

            results.group.insert(g.name.clone(), NodeGroup {
                name: g.name.clone(),
                nodes: members,
            });
        }

        // ── pipeline ──
        for pipe in &self.config.pipeline {
            let piped = Self::run_pipeline(&results, &pipe.steps);
            results.pipeline.insert(pipe.name.clone(), NodeGroup {
                name: pipe.name.clone(),
                nodes: piped,
            });
        }

        results
    }

    /// 检查一个 node 是否匹配 rule 的所有字段条件 (AND)
    fn match_rule(node: &NodeContext, fields: &HashMap<String, String>) -> bool {
        if fields.is_empty() {
            return true;
        }
        fields.iter().all(|(field, pattern)| {
            let field_value = Self::get_field(node, field);
            Regex::new(pattern)
                .map(|re| re.is_match(&field_value))
                .unwrap_or(false)
        })
    }

    /// 从 node 里取出指定字段的值（用于规则匹配）
    fn get_field(node: &NodeContext, field: &str) -> String {
        match field {
            "scheme"   => node.scheme.clone(),
            "host"     => node.host.clone(),
            "port"     => node.port.to_string(),
            "fragment" => node.fragment.clone(),
            "raw"      => node.raw.clone(),
            "source"   => node.source.clone(),
            _ => {
                // 支持嵌套: "userinfo.net" → node.userinfo["net"]
                if let Some(rest) = field.strip_prefix("userinfo.") {
                    return match &node.userinfo {
                        serde_json::Value::Object(map) => map.get(rest).map_or_else(String::new, |v| v.as_str().unwrap_or("").to_string()),
                        _ => String::new(),
                    };
                }
                // 支持: "query.sni" → node.query["sni"]
                if let Some(rest) = field.strip_prefix("query.") {
                    return node.query.get(rest).cloned().unwrap_or_default();
                }
                String::new()
            }
        }
    }

    /// dedup: 按规则字段的值分组，每组保留第一个
    fn apply_dedup(nodes: &[&NodeContext], fields: &HashMap<String, String>) -> Vec<NodeContext> {
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut result: Vec<NodeContext> = Vec::new();

        for node in nodes {
            // 按 fields 的值构造分组 key
            let key: String = fields
                .keys()
                .map(|f| Self::get_field(node, f))
                .collect::<Vec<_>>()
                .join("|");

            if !seen.contains_key(&key) {
                seen.insert(key.clone(), result.len());
                result.push((*node).clone());
            }
        }

        result
    }

    /// pipeline: 对 nodes 依次应用规则
    ///
    /// step 格式: `"规则类型.规则名"`
    /// - `"group.xxx"`, `"dedup.xxx"` → 取节点（第一条初始化，后续取交集）
    /// - `"exclude.xxx"` → 从当前集合中移除匹配节点
    fn run_pipeline(results: &RuleResults, steps: &[String]) -> Vec<NodeContext> {
        use std::collections::HashSet;

        let mut nodes: Vec<NodeContext> = Vec::new();
        let mut initialized = false;

        for step in steps.iter() {
            let (kind, name) = match step.split_once('.') {
                Some(parts) => parts,
                None => continue,
            };

            match kind {
                "exclude" => {
                    // 从当前集合移除被 exclude 的节点
                    if let Some(g) = results.exclude.get(name) {
                        let exclude_set: HashSet<String> =
                            g.nodes.iter().map(|n| Self::node_key(n)).collect();
                        nodes.retain(|n| !exclude_set.contains(&Self::node_key(n)));
                    }
                }
                _ => {
                    let group_ref = match kind {
                        "dedup" => results.dedup.get(name),
                        "group" => results.group.get(name),
                        _ => None,
                    };

                    if let Some(g) = group_ref {
                        if !initialized {
                            nodes = g.nodes.clone();
                            initialized = true;
                        } else {
                            let step_set: HashSet<String> =
                                g.nodes.iter().map(|n| Self::node_key(n)).collect();
                            nodes.retain(|n| step_set.contains(&Self::node_key(n)));
                        }
                    }
                }
            }
        }

        nodes
    }

    /// 节点的唯一标识（用于 pipeline 交集计算）
    fn node_key(node: &NodeContext) -> String {
        format!("{}|{}|{}", node.scheme, node.host, node.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn make_node(fragment: &str, host: &str, port: u16, scheme: &str) -> NodeContext {
        NodeContext {
            scheme: scheme.to_string(),
            userinfo: Value::String("test".into()),
            host: host.to_string(),
            port,
            query: HashMap::new(),
            fragment: fragment.to_string(),
            raw: format!("{scheme}://user@{host}:{port}#{fragment}"),
            source: "test".to_string(),
        }
    }

    #[test]
    fn test_dedup_by_host() {
        let nodes = vec![
            make_node("A", "1.2.3.4", 443, "trojan"),
            make_node("B", "1.2.3.4", 443, "vmess"),   // same host+port → deduped
            make_node("C", "5.6.7.8", 443, "trojan"),
        ];

        let config = toml::from_str::<RulesConfig>(
            r#"
            [[dedup]]
            name = "main"
            host = ".*"
            port = ".*"
            "#
        ).unwrap();

        let engine = RuleEngine::from_config(config);
        let results = engine.run(&nodes);

        let deduped = &results.dedup["main"].nodes;
        assert_eq!(deduped.len(), 2);
        assert_eq!(deduped[0].fragment, "A");
        assert_eq!(deduped[1].fragment, "C");
    }

    #[test]
    fn test_exclude_and_group() {
        let nodes = vec![
            make_node("香港 01", "1.2.3.4", 443, "vmess"),
            make_node("香港 02", "2.3.4.5", 443, "trojan"),
            make_node("免费节点", "5.6.7.8", 80, "http"),
        ];

        let config = toml::from_str::<RulesConfig>(
            r#"
            [[exclude]]
            name = "dead"
            scheme = "http"

            [[group]]
            name = "🇭🇰 香港"
            fragment = "香港"

            [[group]]
            name = "🌍 其他"
            "#
        ).unwrap();

        let engine = RuleEngine::from_config(config);
        let results = engine.run(&nodes);

        assert_eq!(results.exclude["dead"].nodes.len(), 1);
        assert_eq!(results.exclude["dead"].nodes[0].fragment, "免费节点");

        assert_eq!(results.group["🇭🇰 香港"].nodes.len(), 2);
        assert_eq!(results.group["🌍 其他"].nodes.len(), 0); // 免费节点被 exclude 过滤，兜底组空了
    }

    #[test]
    fn test_local_dedup() {
        let nodes = vec![
            make_node("香港01", "1.2.3.4", 443, "vmess"),
            make_node("HK01", "5.6.7.8", 443, "vmess"),
            make_node("HK01", "1.2.3.4", 443, "trojan"),
        ];

        let config = toml::from_str::<RulesConfig>(
            r#"
            [[dedup]]
            name = "hk_vmess"
            fragment = "香港01|HK01"
            scheme = "vmess"
            "#
        ).unwrap();

        let engine = RuleEngine::from_config(config);
        let results = engine.run(&nodes);

        let deduped = &results.dedup["hk_vmess"].nodes;
        assert_eq!(deduped.len(), 2); // 香港01 + HK01 (vmess only, 同 fragment+scheme deduped)
    }
}
