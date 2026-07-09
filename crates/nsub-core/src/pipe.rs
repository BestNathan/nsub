//! Pipe 引擎 — 执行 `"base64 | json | to_lowercase"` 链式管道。
//!
//! 分两层:
//! 1. 内置函数 (built-in): base64, json, split, urldecode, lowercase, trim, lines
//! 2. 用户自定义 (Rhai):  从 `functions/**/*.rhai` 加载

use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Pipe 函数注册表: 函数名 → 实现
pub type PipeFn = Box<dyn Fn(&str) -> Result<Value, PipeError> + Send + Sync>;

pub struct PipeRegistry {
    builtins: HashMap<String, PipeFn>,
}

#[derive(Debug, Error)]
pub enum PipeError {
    #[error("pipe function not found: {0}")]
    FunctionNotFound(String),
    #[error("{step}: {detail}")]
    StepFailed { step: String, detail: String },
}

impl PipeRegistry {
    pub fn new() -> Self {
        let mut registry = Self { builtins: HashMap::new() };

        // ── 内置 pipe 函数 ──

        registry.register("base64", |v| {
            let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, v)
                .map_err(|e| PipeError::StepFailed { step: "base64".into(), detail: e.to_string() })?;
            let s = String::from_utf8(bytes).map_err(|e| PipeError::StepFailed {
                step: "base64".into(), detail: e.to_string(),
            })?;
            Ok(Value::String(s))
        });

        registry.register("json", |v| {
            serde_json::from_str::<Value>(v).map_err(|e| PipeError::StepFailed {
                step: "json".into(), detail: e.to_string(),
            })
        });

        registry.register("urldecode", |v| {
            Ok(Value::String(
                url::form_urlencoded::parse(v.as_bytes())
                    .map(|(k, _)| k.into_owned())
                    .collect::<Vec<_>>()
                    .first()
                    .cloned()
                    .unwrap_or_else(|| v.to_string()),
            ))
        });

        registry.register("lowercase", |v| Ok(Value::String(v.to_lowercase())));
        registry.register("uppercase", |v| Ok(Value::String(v.to_uppercase())));
        registry.register("trim", |v| Ok(Value::String(v.trim().to_string())));

        // split(":") — 按分隔符切开为数组
        registry.register("split(:)", |v| {
            let parts: Vec<Value> = v.split(':').map(|s| Value::String(s.to_string())).collect();
            Ok(Value::Array(parts))
        });

        // split(;) — 用于 SS plugin 参数
        registry.register("split(;)", |v| {
            let parts: Vec<Value> = v.split(';').map(|s| Value::String(s.to_string())).collect();
            Ok(Value::Array(parts))
        });

        // lines — 按行切开
        registry.register("lines", |v| {
            let parts: Vec<Value> = v.lines().map(|s| Value::String(s.to_string())).collect();
            Ok(Value::Array(parts))
        });

        registry
    }

    pub fn register(&mut self, name: &str, f: impl Fn(&str) -> Result<Value, PipeError> + Send + Sync + 'static) {
        self.builtins.insert(name.to_string(), Box::new(f));
    }

    /// 执行 `"base64 | json | urldecode"` 这样的管道链
    pub fn run(&self, input: &str, pipe_spec: &str) -> Result<Value, PipeError> {
        let steps: Vec<&str> = pipe_spec.split('|').map(str::trim).collect();
        let mut current = Value::String(input.to_string());

        for step in steps {
            current = self.run_step(&current, step)?;
        }

        Ok(current)
    }

    fn run_step(&self, value: &Value, step: &str) -> Result<Value, PipeError> {
        let s = match value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };

        if let Some(func) = self.builtins.get(step) {
            return func(&s);
        }

        // 支持 split(<delim>) 泛型
        if step.starts_with("split(") && step.ends_with(')') {
            let delim = &step[6..step.len() - 1];
            let parts: Vec<Value> = s.split(delim).map(|p| Value::String(p.to_string())).collect();
            return Ok(Value::Array(parts));
        }

        Err(PipeError::FunctionNotFound(step.to_string()))
    }
}

impl Default for PipeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
