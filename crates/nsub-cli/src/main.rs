//! nsub CLI — 订阅转换工具
//!
//! 用法:
//!   nsub convert --from sub.txt --to clash/simple --rules simple
//!   nsub list protocols
//!   nsub list templates
//!   nsub list rules

use anyhow::Result;
use clap::{Parser, Subcommand};
use nsub_core::{fetch, protocol::ProtocolRegistry, render::Renderer, rules::RuleEngine};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nsub", about = "订阅转换工具 — URL → Node → 配置")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 执行订阅转换
    Convert(ConvertArgs),
    /// 列出可用资源
    #[command(subcommand)]
    List(ListArgs),
}

#[derive(clap::Args)]
struct ConvertArgs {
    /// 订阅源（URL 或文件路径），多个用逗号分隔
    #[arg(short, long, value_delimiter = ',')]
    from: Vec<String>,

    /// 目标模板（如 clash/simple、clash/grouped、surge/config）
    #[arg(short, long)]
    to: String,

    /// 输出文件路径（默认 stdout）
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// 规则名称（对应 rules/<name>.toml，不含扩展名）
    #[arg(short, long, default_value = "simple")]
    rules: String,

    /// 规则目录（默认 ./rules）
    #[arg(long, default_value = "rules")]
    rules_dir: PathBuf,

    /// 协议定义目录（默认 ./protocols）
    #[arg(long, default_value = "protocols")]
    protocol_dir: PathBuf,

    /// 模板目录（默认 ./templates）
    #[arg(long, default_value = "templates")]
    template_dir: PathBuf,
}

#[derive(Subcommand)]
enum ListArgs {
    /// 列出已加载的协议
    Protocols,
    /// 列出可用模板
    Templates,
    /// 列出可用规则
    Rules,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::List(args) => match args {
            ListArgs::Protocols => {
                println!("协议定义 (protocols/):");
                for entry in std::fs::read_dir("protocols")? {
                    let entry = entry?;
                    if entry.path().extension().map_or(false, |e| e == "toml") {
                        println!("  {}", entry.file_name().to_string_lossy());
                    }
                }
            }
            ListArgs::Templates => {
                let renderer = Renderer::load("templates")?;
                println!("可用模板:");
                for t in renderer.list_templates() {
                    println!("  {t}");
                }
            }
            ListArgs::Rules => {
                println!("可用规则 (rules/):");
                for entry in std::fs::read_dir("rules")? {
                    let entry = entry?;
                    if entry.path().extension().map_or(false, |e| e == "toml") {
                        let name = entry.path().file_stem().unwrap().to_string_lossy().to_string();
                        println!("  {name}");
                    }
                }
            }
        },
        Command::Convert(args) => {
            run_convert(args).await?;
        }
    }

    Ok(())
}

async fn run_convert(args: ConvertArgs) -> Result<()> {
    // 1. 加载协议定义
    let registry = ProtocolRegistry::load(&args.protocol_dir)?;
    eprintln!("[nsub] 协议: {} 个", {
        let mut count = 0;
        for _ in std::fs::read_dir(&args.protocol_dir)? { count += 1; }
        count
    });

    // 2. 加载规则
    let rules_path = args.rules_dir.join(format!("{}.toml", args.rules));
    let rules_content = std::fs::read_to_string(&rules_path)?;
    let rules_config: nsub_core::rules::RulesConfig = toml::from_str(&rules_content)?;
    let rule_engine = RuleEngine::from_config(rules_config);

    // 3. 加载模板
    let renderer = Renderer::load(&args.template_dir)?;
    eprintln!("[nsub] 模板: {} 个", renderer.list_templates().len());

    // 4. 拉取订阅 → 解析每个 URI
    let mut all_nodes = Vec::new();
    for source in &args.from {
        let (raw, label) = fetch::fetch(source).await?;
        eprintln!("[nsub] 拉取: {source} ({} bytes) [label: {label}]", raw.len());

        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            match registry.parse_url(line, &label) {
                Ok(node) => all_nodes.push(node),
                Err(e) => eprintln!("[nsub] skip: {e}"),
            }
        }
    }
    eprintln!("[nsub] 解析: {} 个节点", all_nodes.len());

    // 5. 执行规则
    let rule_results = rule_engine.run(&all_nodes);
    eprintln!(
        "[nsub] 规则: dedup={} excl={} group={} pipe={}",
        rule_results.dedup.len(),
        rule_results.exclude.len(),
        rule_results.group.len(),
        rule_results.pipeline.len(),
    );

    // 6. 渲染模板
    let output = renderer.render(&args.to, all_nodes, rule_results)?;

    // 7. 输出
    match args.output {
        Some(path) => {
            std::fs::write(&path, &output)?;
            eprintln!("[nsub] 输出: {}", path.display());
        }
        None => print!("{output}"),
    }

    eprintln!("[nsub] 完成");
    Ok(())
}
