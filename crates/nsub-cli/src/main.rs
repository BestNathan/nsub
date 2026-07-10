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
    /// 查看扩展指南 (protocols, templates, rules, functions)
    Skills(SkillsArgs),
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

    /// 规则目录
    #[arg(long)]
    rules_dir: Option<PathBuf>,

    /// 协议定义目录
    #[arg(long)]
    protocol_dir: Option<PathBuf>,

    /// 模板目录
    #[arg(long)]
    template_dir: Option<PathBuf>,
}

#[derive(clap::Args)]
struct SkillsArgs {
    /// 查看哪个 skill（不传则列出所有可用 skill）
    name: Option<String>,
    /// skills 目录（默认自动检测）
    #[arg(long)]
    skills_dir: Option<PathBuf>,
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

/// 从二进制位置推导默认 assets 目录
///
/// 安装结构:
///   ~/.local/bin/nsub          ← 二进制
///   ~/.local/share/nsub/       ← assets (templates/protocols/rules)
///
/// 先检查 `../share/nsub/` (相对于二进制), 不存在则回退到 CWD。
fn default_asset_dir(name: &str) -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(bin_dir) = exe.parent()
    {
        let share = bin_dir
            .parent()
            .unwrap_or(bin_dir)
            .join("share")
            .join("nsub")
            .join(name);
        if share.is_dir() {
            return share;
        }
    }
    PathBuf::from(name)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::List(args) => match args {
            ListArgs::Protocols => {
                let dir = default_asset_dir("protocols");
                println!("协议定义 ({}):", dir.display());
                for entry in std::fs::read_dir(&dir)? {
                    let entry = entry?;
                    if entry.path().extension().is_some_and(|e| e == "toml") {
                        println!("  {}", entry.file_name().to_string_lossy());
                    }
                }
            }
            ListArgs::Templates => {
                let dir = default_asset_dir("templates");
                let renderer = Renderer::load(&dir)?;
                println!("可用模板:");
                for t in renderer.list_templates() {
                    println!("  {t}");
                }
            }
            ListArgs::Rules => {
                let dir = default_asset_dir("rules");
                println!("可用规则 ({}):", dir.display());
                for entry in std::fs::read_dir(&dir)? {
                    let entry = entry?;
                    if entry.path().extension().is_some_and(|e| e == "toml") {
                        let name = entry
                            .path()
                            .file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        println!("  {name}");
                    }
                }
            }
        },
        Command::Skills(args) => {
            let base = args
                .skills_dir
                .unwrap_or_else(|| default_asset_dir("skills"));

            match args.name {
                Some(name) => {
                    let skill_path = base.join(&name).join("SKILL.md");
                    if skill_path.is_file() {
                        let content = std::fs::read_to_string(&skill_path)?;
                        println!("{}", content);
                    } else {
                        eprintln!(
                            "未找到 skill '{}' (查找路径: {})",
                            name,
                            skill_path.display()
                        );
                        eprintln!();
                        eprintln!("可用 skills:");
                        for entry in std::fs::read_dir(&base)? {
                            let entry = entry?;
                            if entry.path().is_dir() {
                                let n = entry.file_name().to_string_lossy().to_string();
                                if entry.path().join("SKILL.md").is_file() {
                                    eprintln!("  {n}");
                                }
                            }
                        }
                        anyhow::bail!("skill '{}' not found", name);
                    }
                }
                None => {
                    println!("可用 skills ({}):", base.display());
                    println!();
                    for entry in std::fs::read_dir(&base)? {
                        let entry = entry?;
                        if entry.path().is_dir() {
                            let n = entry.file_name().to_string_lossy().to_string();
                            let skill_path = entry.path().join("SKILL.md");
                            if skill_path.is_file() {
                                // 读取第一行（标题）作为简介
                                let content = std::fs::read_to_string(&skill_path)?;
                                let title = content
                                    .lines()
                                    .find(|l| l.starts_with("# "))
                                    .map(|l| l.trim_start_matches("# ").to_string())
                                    .unwrap_or_else(|| n.clone());
                                println!("  {:<16} {}", format!("{n}:"), title);
                            }
                        }
                    }
                    println!();
                    println!("使用 `nsub skills <name>` 查看详细内容");
                }
            }
        }
        Command::Convert(args) => {
            run_convert(args).await?;
        }
    }

    Ok(())
}

async fn run_convert(args: ConvertArgs) -> Result<()> {
    let protocol_dir = args
        .protocol_dir
        .unwrap_or_else(|| default_asset_dir("protocols"));
    let rules_dir = args.rules_dir.unwrap_or_else(|| default_asset_dir("rules"));
    let template_dir = args
        .template_dir
        .unwrap_or_else(|| default_asset_dir("templates"));

    // 1. 加载协议定义
    let registry = ProtocolRegistry::load(&protocol_dir)?;
    eprintln!("[nsub] 协议: {} 个", {
        let mut count = 0;
        for _ in std::fs::read_dir(&protocol_dir)? {
            count += 1;
        }
        count
    });

    // 2. 加载规则
    let rules_path = rules_dir.join(format!("{}.toml", args.rules));
    let rules_content = std::fs::read_to_string(&rules_path)?;
    let rules_config: nsub_core::rules::RulesConfig = toml::from_str(&rules_content)?;
    let rule_engine = RuleEngine::from_config(rules_config);

    // 3. 加载模板
    let renderer = Renderer::load(&template_dir)?;
    eprintln!("[nsub] 模板: {} 个", renderer.list_templates().len());

    // 4. 拉取订阅 → 解析每个 URI
    let mut all_nodes = Vec::new();
    for source in &args.from {
        let (raw, label) = fetch::fetch(source).await?;
        eprintln!(
            "[nsub] 拉取: {source} ({} bytes) [label: {label}]",
            raw.len()
        );

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
