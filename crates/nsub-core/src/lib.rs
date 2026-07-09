//! nsub-core — 订阅转换引擎
//!
//! 不包含任何协议硬编码知识。所有协议解析规则来自 TOML 配置，
//! 所有输出格式来自 Tera 模板。

pub mod fetch;
pub mod pipe;
pub mod protocol;
pub mod render;
pub mod rules;
pub mod types;

pub use types::{NodeContext, RuleResults};
