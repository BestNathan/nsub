# nsub — 订阅转换 CLI

把代理订阅 URL 转换成 Clash/Mihomo、Surge 等客户端的配置文件。

```
订阅 URL → 解析协议 → 去重/过滤/分组 → 渲染模板 → 配置文件
```

## 安装

```bash
curl -fsSL https://raw.githubusercontent.com/BestNathan/nsub/main/install.sh | bash
```

或指定全局安装：

```bash
curl -fsSL https://raw.githubusercontent.com/BestNathan/nsub/main/install.sh | bash -s -- --global
```

## 快速开始

```bash
# 单个订阅 → Clash 配置
nsub convert -f "https://your.sub.com/link?token=xxx" -t clash/grouped

# 多个订阅合并
nsub convert -f "sub1.txt,https://sub2.com/link" -t clash/by-source -r by-source -o config.yaml

# 查看可用资源
nsub list templates
nsub list rules
nsub list protocols
```

## 命令

### `nsub convert`

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `-f, --from` | 订阅源（URL 或文件），逗号分隔多个 | — |
| `-t, --to` | 目标模板 | — |
| `-r, --rules` | 规则名称 | `simple` |
| `-o, --output` | 输出文件 | stdout |
| `--template-dir` | 模板目录 | `./templates` |
| `--protocol-dir` | 协议目录 | `./protocols` |
| `--rules-dir` | 规则目录 | `./rules` |

### `nsub list`

```bash
nsub list templates   # 可用模板
nsub list rules       # 可用规则
nsub list protocols   # 已加载协议
```

## 目录结构

```
├── templates/         # Tera 模板 — 控制输出格式
│   └── clash/         #   clash/grouped, clash/config, clash/simple, clash/by-source
│   └── surge/         #   surge/config
├── protocols/         # TOML 协议定义 — 控制 URL 解析
│   ├── ss.toml
│   ├── trojan.toml
│   ├── vmess.toml
│   ├── vless.toml
│   ├── hysteria2.toml
│   └── tuic.toml
├── rules/             # TOML 规则配置 — dedup / exclude / group / pipeline
│   ├── simple.toml
│   └── by-source.toml
├── functions/         # Rhai 自定义函数（可选）
└── install.sh         # 安装脚本
```

## 规则引擎

四类规则，全部在 `rules/*.toml` 中定义：

| 类型 | 说明 | 匹配逻辑 |
|------|------|----------|
| `dedup` | 去重 | 按字段值分组，每组保留第一个 |
| `exclude` | 排除 | 命中即排除 |
| `group` | 分组 | 从上到下，第一节命中即归属 |
| `pipeline` | 管道 | 串联上述规则，取交集 |

规则支持匹配字段：`scheme`, `host`, `port`, `fragment`, `source`, `query.*`, `userinfo.*`

### 示例：按订阅来源分组

```toml
# rules/by-source.toml
[[dedup]]
name = "feiniao"
source = "test_sub\\.txt"
host = ".*"
port = ".*"

[[dedup]]
name = "jiasuqi"
source = "jsjc\\.cfd"
host = ".*"
port = ".*"

[[pipeline]]
name = "🐦 飞鸟云"
steps = ["dedup.feiniao", "exclude.ad"]

[[pipeline]]
name = "🚀 加速器"
steps = ["dedup.jiasuqi", "exclude.ad"]
```

## 协议扩展

在 `protocols/` 下新增 `.toml` 文件即可支持新协议：

```toml
# protocols/myproto.toml
[protocol]
schemes = ["myproto"]

[decode]
userinfo = "base64 | json"
fragment = "urldecode"
```

## Pipe 引擎

`decode` 配置使用管道链处理字段值：

```
"base64 | json | urldecode"
"trim | lowercase"
"split(:)"
"lines"
```

## 构建

```bash
cargo build --release
./target/release/nsub --help
```

## License

MIT
