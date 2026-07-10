//! nsub CLI — 订阅转换工具
//!
//! 用法:
//!   nsub convert --from sub.txt --to clash/simple --rules simple
//!   nsub list protocols
//!   nsub list templates
//!   nsub list rules
//!
//! 用户自定义: ~/.nsub/protocols/ ~/.nsub/templates/ ~/.nsub/rules/ ...
//! 用户目录中的资源优先级高于安装目录，可覆盖默认资源。

use anyhow::Result;
use clap::{Parser, Subcommand};
use nsub_core::{fetch, protocol::ProtocolRegistry, render::Renderer, rules::RuleEngine};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "nsub",
    about = "订阅转换工具 — URL → Node → 配置",
    after_help = "扩展指南: nsub skills  查看如何添加协议、模板、规则\n用户自定义: ~/.nsub/  放置自定义协议/模板/规则/函数"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 执行订阅转换
    #[command(after_help = "提示: nsub skills  查看如何定义协议、模板、规则")]
    Convert(ConvertArgs),
    /// 列出可用资源
    #[command(subcommand)]
    List(ListArgs),
    /// 查看扩展指南 (protocols, templates, rules, functions)
    #[command(
        after_help = "示例:\n  nsub skills            列出所有扩展指南\n  nsub skills protocols   查看协议定义文档\n  nsub skills templates   查看模板文档\n  nsub skills rules       查看规则文档"
    )]
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
    /// 列出已加载的协议 (扩展: nsub skills protocols)
    Protocols,
    /// 列出可用模板 (扩展: nsub skills templates)
    Templates,
    /// 列出可用规则 (扩展: nsub skills rules)
    Rules,
}

// ── 路径解析 ───────────────────────────────────────────────────────

/// 用户 home 目录
fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

/// 用户自定义目录: `~/.nsub/`
fn user_nsub_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join(".nsub")
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

/// 解析资源的有效目录列表：[install_dir, user_dir]
///
/// 用户目录在后，加载时后加载的覆盖先加载的同名资源。
/// 如果通过 CLI 显式指定了目录，则只使用指定目录（不使用用户目录）。
fn resolve_asset_dirs(name: &str, explicit: Option<&PathBuf>) -> Vec<PathBuf> {
    if let Some(dir) = explicit {
        return vec![dir.clone()];
    }
    let mut dirs = vec![default_asset_dir(name)];
    let user_dir = user_nsub_dir().join(name);
    if user_dir.is_dir() {
        dirs.push(user_dir);
    }
    dirs
}

/// 列出单个目录下的 .toml 文件名（不含扩展名），返回 (路径, 文件名列表)
fn list_toml_dir(dir: &PathBuf) -> Result<Vec<String>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "toml"))
        .filter_map(|e| {
            e.path()
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
        })
        .collect();
    names.sort();
    Ok(names)
}

/// 列出目录下所有 skill 子目录名
fn list_skill_dirs(dir: &PathBuf) -> Result<Vec<String>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").is_file())
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();
    names.sort();
    Ok(names)
}

// ── 主入口 ─────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::List(args) => match args {
            ListArgs::Protocols => {
                let install_dir = default_asset_dir("protocols");
                let user_dir = user_nsub_dir().join("protocols");

                println!("协议定义:");
                print_dir_section("安装", &install_dir, "toml");
                let has_user = user_dir.is_dir();
                if has_user {
                    print_dir_section("用户 (~/.nsub/protocols/)", &user_dir, "toml");
                }
                if !install_dir.is_dir() && !has_user {
                    println!("  (未找到协议目录)");
                }
                println!();
                println!("提示: nsub skills protocols   # 如何定义新协议");
                println!("      用户自定义协议放在 ~/.nsub/protocols/ 可覆盖默认协议");
            }
            ListArgs::Templates => {
                let install_dir = default_asset_dir("templates");
                let user_dir = user_nsub_dir().join("templates");

                // 分别列出两个目录的模板
                println!("可用模板:");
                if install_dir.is_dir() {
                    let renderer = Renderer::load(&install_dir)?;
                    let templates: Vec<String> = renderer.list_templates();
                    println!("  [安装] {}:", install_dir.display());
                    for t in &templates {
                        println!("    {t}");
                    }
                }
                if user_dir.is_dir() {
                    let renderer = Renderer::load(&user_dir)?;
                    let templates: Vec<String> = renderer.list_templates();
                    println!("  [用户] ~/.nsub/templates/:");
                    for t in &templates {
                        println!("    {t} (覆盖安装)");
                    }
                }
                if !install_dir.is_dir() && !user_dir.is_dir() {
                    println!("  (未找到模板目录)");
                }
                println!();
                println!("提示: nsub skills templates   # 如何写模板");
                println!("      用户自定义模板放在 ~/.nsub/templates/ 可覆盖默认模板");
            }
            ListArgs::Rules => {
                let install_dir = default_asset_dir("rules");
                let user_dir = user_nsub_dir().join("rules");

                println!("可用规则:");
                if install_dir.is_dir() {
                    let names = list_toml_dir(&install_dir)?;
                    println!("  [安装] {}:", install_dir.display());
                    for n in &names {
                        println!("    {n}");
                    }
                }
                if user_dir.is_dir() {
                    let names = list_toml_dir(&user_dir)?;
                    if !names.is_empty() {
                        println!("  [用户] ~/.nsub/rules/:");
                        for n in &names {
                            // 检查是否覆盖了安装目录的同名规则
                            let override_mark = if install_dir.join(format!("{n}.toml")).exists() {
                                " (覆盖安装)"
                            } else {
                                ""
                            };
                            println!("    {n}{override_mark}");
                        }
                    }
                }
                if !install_dir.is_dir() && !user_dir.is_dir() {
                    println!("  (未找到规则目录)");
                }
                println!();
                println!("提示: nsub skills rules       # 如何用规则引擎");
                println!("      用户自定义规则放在 ~/.nsub/rules/ 可覆盖默认规则");
            }
        },
        Command::Skills(args) => {
            let install_dir = args
                .skills_dir
                .unwrap_or_else(|| default_asset_dir("skills"));
            let user_dir = user_nsub_dir().join("skills");

            match args.name {
                Some(name) => {
                    // 优先查找用户目录，再查安装目录
                    let user_skill = user_dir.join(&name).join("SKILL.md");
                    let skill_path = if user_skill.is_file() {
                        user_skill
                    } else {
                        install_dir.join(&name).join("SKILL.md")
                    };

                    if skill_path.is_file() {
                        let content = std::fs::read_to_string(&skill_path)?;
                        println!("{}", content);
                    } else {
                        // 列出所有可用 skills
                        let mut install_skills = list_skill_dirs(&install_dir)?;
                        let user_skills = list_skill_dirs(&user_dir)?;
                        // 合并去重，user 优先
                        for s in &user_skills {
                            if !install_skills.contains(s) {
                                install_skills.push(s.clone());
                            }
                        }

                        eprintln!(
                            "未找到 skill '{}' (已查找: ~/.nsub/skills/ 和安装目录)",
                            name
                        );
                        eprintln!();
                        eprintln!("可用 skills:");
                        for n in &install_skills {
                            let source = if user_skills.contains(n) {
                                "~/.nsub/skills/"
                            } else {
                                "安装"
                            };
                            eprintln!("  {n} ({source})");
                        }
                        anyhow::bail!("skill '{}' not found", name);
                    }
                }
                None => {
                    let install_skills = list_skill_dirs(&install_dir)?;
                    let user_skills = list_skill_dirs(&user_dir)?;

                    // 合并 skill 名，user 优先标注
                    let mut all: Vec<(String, &str)> = Vec::new();
                    for n in &install_skills {
                        let source = if user_skills.contains(n) {
                            "用户"
                        } else {
                            "安装"
                        };
                        all.push((n.clone(), source));
                    }
                    for n in &user_skills {
                        if !install_skills.contains(n) {
                            all.push((n.clone(), "用户"));
                        }
                    }

                    println!("可用 skills:");
                    println!();
                    if all.is_empty() {
                        println!("  (未找到 skills)");
                    } else {
                        for (n, source) in &all {
                            // 尝试读取标题
                            let user_skill_md = user_dir.join(n).join("SKILL.md");
                            let skill_md = if user_skill_md.is_file() {
                                user_skill_md
                            } else {
                                install_dir.join(n).join("SKILL.md")
                            };

                            let title = if skill_md.is_file() {
                                std::fs::read_to_string(&skill_md)
                                    .ok()
                                    .and_then(|c| {
                                        c.lines()
                                            .find(|l| l.starts_with("# "))
                                            .map(|l| l.trim_start_matches("# ").to_string())
                                    })
                                    .unwrap_or_else(|| n.clone())
                            } else {
                                n.clone()
                            };
                            println!(
                                "  {:<16} {} {}",
                                format!("{n}:"),
                                title,
                                if *source == "用户" {
                                    "(~/.nsub/)"
                                } else {
                                    ""
                                }
                            );
                        }
                    }
                    println!();
                    println!("示例:");
                    println!("  nsub skills protocols   # 如何定义新协议的解析规则");
                    println!("  nsub skills templates   # 如何写模板控制输出格式");
                    println!("  nsub skills rules       # 如何用规则引擎去重/过滤/分组");
                    println!("  nsub skills functions   # 如何用 Rhai 扩展 pipe 函数");
                    println!();
                    println!("自定义: 在 ~/.nsub/skills/<name>/SKILL.md 放置自定义扩展指南");
                }
            }
        }
        Command::Convert(args) => {
            run_convert(args).await?;
        }
    }

    Ok(())
}

/// 辅助: 列出目录下的文件
fn print_dir_section(label: &str, dir: &PathBuf, ext: &str) {
    if !dir.is_dir() {
        return;
    }
    println!("  [{label}] {}:", dir.display());
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == ext))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    files.sort();
    for f in &files {
        println!("    {f}");
    }
}

async fn run_convert(args: ConvertArgs) -> Result<()> {
    // 解析目录：如果显式指定则只用指定的，否则 [install, user]
    let protocol_dirs = resolve_asset_dirs("protocols", args.protocol_dir.as_ref());
    let template_dirs = resolve_asset_dirs("templates", args.template_dir.as_ref());
    let rules_dirs = resolve_asset_dirs("rules", args.rules_dir.as_ref());

    // 1. 加载协议定义（多目录，user 覆盖 install）
    let registry = ProtocolRegistry::load_from_dirs(&protocol_dirs)?;
    let scheme_count = registry.list_schemes().len();
    eprintln!("[nsub] 协议: {scheme_count} 个 (来源: {})", {
        protocol_dirs
            .iter()
            .map(|d| d.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    });

    // 2. 加载规则 — 优先使用用户目录中的同名规则
    let rules_path = find_rule_file(&args.rules, &rules_dirs)?;
    let rules_content = std::fs::read_to_string(&rules_path)?;
    let rules_config: nsub_core::rules::RulesConfig = toml::from_str(&rules_content)?;
    let rule_engine = RuleEngine::from_config(rules_config);
    eprintln!(
        "[nsub] 规则: {} (来自 {})",
        args.rules,
        rules_path.display()
    );

    // 3. 加载模板（多目录，user 覆盖 install）
    let renderer = Renderer::load_from_dirs(&template_dirs)?;
    eprintln!(
        "[nsub] 模板: {} 个 (来源: {})",
        renderer.list_templates().len(),
        {
            template_dirs
                .iter()
                .map(|d| d.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        }
    );

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

/// 在多个目录中查找规则文件，返回第一个匹配的路径
///
/// 按 dirs 顺序查找：先找安装目录，再找用户目录。
/// 用户目录中的同名规则覆盖安装目录。
fn find_rule_file(name: &str, dirs: &[PathBuf]) -> Result<PathBuf> {
    let filename = format!("{name}.toml");
    // 反向查找：后面的目录优先（用户覆盖）
    for dir in dirs.iter().rev() {
        let path = dir.join(&filename);
        if path.is_file() {
            return Ok(path);
        }
    }
    // 正向查找给出有意义的错误
    for dir in dirs {
        let path = dir.join(&filename);
        if path.is_file() {
            return Ok(path);
        }
    }
    anyhow::bail!(
        "规则 '{}' 未找到 (已查找: {})",
        name,
        dirs.iter()
            .map(|d| d.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}
