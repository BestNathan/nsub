# Rules — 规则引擎

## 概述

`rules/` 目录存放 TOML 文件，定义 dedup（去重）、exclude（排除）、group（分组）、pipeline（管道）四类规则。每条规则用正则表达式匹配节点的字段，决定节点如何处理和归类。

## 文件位置

```
rules/
  simple.toml       ← 全节点去重 + 过滤广告
  grouped.toml      ← 按地区关键词分组
  by-source.toml    ← 按订阅来源分组
```

## TOML 格式

```toml
[[dedup]]
name = "规则名"
field = "regex"       # 命中此规则的节点按 field 值分组去重

[[exclude]]
name = "规则名"
field = "regex"       # 命中此规则的节点被排除

[[group]]
name = "组名"
field = "regex"       # 第一节命中即归属，不命中流入下一组

[[pipeline]]
name = "管道名"
steps = ["dedup.xxx", "exclude.yyy"]   # 串联规则
```

## 四类规则

### dedup — 去重

按指定的字段值分组，每组只保留第一个出现的节点。

```toml
[[dedup]]
name = "main"
host = ".*"      # 按 host 去重
port = ".*"      # 且按 port 去重（AND 逻辑）
```

等价于：相同 `host:port` 的节点只保留一个。`host = ".*"` 和 `port = ".*"` 之间是 **AND** 关系——两个字段都匹配才命中。

```toml
[[dedup]]
name = "feiniao"
source = "test_sub\\.txt"    # 只对某个订阅源去重
host = ".*"
port = ".*"
```

### exclude — 排除

命中规则的节点被排除，不进入后续处理。

```toml
# 排除广告/提示类节点
[[exclude]]
name = "ad"
fragment = "超过20多个|客户端设置|电报群|防失联|剩余流量|套餐到期"

# 排除占位节点
[[exclude]]
name = "placeholder"
host = "1\\.1\\.1\\.1|2\\.2\\.2\\.2"
```

### group — 分组

从上到下依次匹配，**第一节命中即归属**，不命中进入下一组。空 fields 的组是兜底组。

```toml
[[group]]
name = "🐦 飞鸟云"
source = "459292\\.xyz|459921\\.xyz|344211\\.cc"

[[group]]
name = "🚀 加速器"
source = "jsjc\\.cfd"

[[group]]
name = "🌍 其他"          # 空 fields = 兜底，接收前面未命中的节点
```

### pipeline — 管道

串联已命名的规则，按顺序执行：

```toml
[[pipeline]]
name = "all"
steps = ["dedup.main", "exclude.ad", "exclude.placeholder"]
```

步骤格式：`规则类型.规则名`。
- `dedup.xxx` / `group.xxx` → 取节点集合（第一条初始化，后续取交集）
- `exclude.xxx` → 从当前集合中移除匹配的节点

## 可匹配的字段

| 字段 | 访问方式 | 示例 pattern |
|------|---------|-------------|
| `scheme` | `node.scheme` | `"vmess"` |
| `host` | `node.host` | `"1\\.2\\.3\\.4"` |
| `port` | `node.port` | `"443"` |
| `fragment` | `node.fragment` | `"香港"` |
| `source` | `node.source` | `"jsjc\\.cfd"` |
| `raw` | `node.raw` | 原始 URL |
| `query.<key>` | `node.query.sni` | `"example\\.com"` |
| `userinfo.<key>` | `node.userinfo.add` | `"1\\.2\\.3\\.4"` |

## 常见模式

### 全量去重

```toml
[[dedup]]
name = "main"
host = ".*"
port = ".*"

[[pipeline]]
name = "all"
steps = ["dedup.main"]
```

### 去重 + 过滤 → 输出

```toml
[[dedup]]
name = "main"
host = ".*"
port = ".*"

[[exclude]]
name = "ad"
fragment = "广告|推广|免费"

[[pipeline]]
name = "clean"
steps = ["dedup.main", "exclude.ad"]
```

### 按订阅来源分出独立管道

```toml
[[dedup]]
name = "sub1"
source = "mysub\\.com"
host = ".*"
port = ".*"

[[dedup]]
name = "sub2"
source = "other\\.com"
host = ".*"
port = ".*"

[[exclude]]
name = "ad"
fragment = "广告"

[[pipeline]]
name = "订阅1"
steps = ["dedup.sub1", "exclude.ad"]

[[pipeline]]
name = "订阅2"
steps = ["dedup.sub2", "exclude.ad"]
```

模板里 `{{ pipeline["订阅1"].nodes }}` 和 `{{ pipeline["订阅2"].nodes }}` 就是两个独立代理组。

## 新增规则

加一个 `rules/myrule.toml`：

```toml
[[dedup]]
name = "main"
host = ".*"
port = ".*"

[[pipeline]]
name = "all"
steps = ["dedup.main"]
```

使用：`nsub convert -f sub.txt -t clash/simple -r myrule`

不需要改任何 Rust 代码。
