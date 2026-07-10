# Protocols — 协议定义

## 概述

`protocols/` 目录存放 TOML 文件，每个文件定义一个代理协议的解析规则。nsub **不硬编码任何协议知识**，所有 URL 解析行为由此目录配置。

## 文件位置

```
protocols/
  hysteria2.toml    ← Hysteria2 / hy2
  ss.toml           ← Shadowsocks
  trojan.toml       ← Trojan
  tuic.toml         ← TUIC
  vless.toml        ← VLESS (reality / tls+ws)
  vmess.toml        ← VMess
```

## TOML 格式

```toml
[protocol]
schemes = ["scheme1", "alias1"]   # 匹配的 URL scheme

[decode]
field = "pipe1 | pipe2 | pipe3"   # 管道链
```

### `[protocol]` — 必填

| 字段 | 类型 | 说明 |
|------|------|------|
| `schemes` | `string[]` | URL scheme 列表，第一个是规范名，其余是别名 |

```toml
[protocol]
schemes = ["hysteria2", "hy2"]   # hysteria2://... 和 hy2://... 都命中
```

### `[decode]` — 可选

对 URL 的各个部分执行 pipe 管道转换。每个字段的值是一串用 `|` 分隔的 pipe 函数名。

**支持的字段：**

| 字段 | 对应 URL 部分 | 示例 |
|------|-------------|------|
| `userinfo` | `scheme://**这部分**@host` | `"base64 | json"` |
| `fragment` | `...#**这部分**` | `"urldecode"` |
| `host` | `scheme://user@**host**:port` | `"lowercase"` |

## 内置 Pipe 函数

| Pipe | 输入 → 输出 | 说明 |
|------|-----------|------|
| `base64` | base64 字符串 → 原始文本 | 标准 base64 解码 |
| `json` | JSON 字符串 → JSON 对象 | 解析为 serde_json::Value |
| `urldecode` | 百分号编码 → 原始字符串 | `%E4%B8%AD` → `中` |
| `split(:)` | 字符串 → 数组 | `a:b` → `["a","b"]` |
| `split(;)` | 字符串 → 数组 | 用于 SS plugin 参数 |
| `lowercase` | 字符串 → 小写 | 大小写归一化 |
| `uppercase` | 字符串 → 大写 | |
| `trim` | 字符串 → 去首尾空白 | |
| `lines` | 字符串 → 行数组 | 按换行切分 |
| `split(<delim>)` | 字符串 → 数组 | 泛型，如 `split(,)` |

## 模板里怎么取

协议解析结果变成 `node` 变量，模板里访问：

```
{{ node.scheme }}     → "hysteria2"
{{ node.host }}       → "1.2.3.4"
{{ node.port }}       → 443
{{ node.fragment }}   → "日本01"
{{ node.raw }}        → 原始 URL
{{ node.source }}     → 订阅来源 (host 或文件名)

{{ node.userinfo }}              → decode 处理后的 userinfo
{{ node.userinfo.id }}           → userinfo 是 JSON 时取字段
{{ node.query.sni }}             → URL 里的 ?sni=xxx
{{ node.query.insecure }}        → URL 里的 ?insecure=1
```

## 实战示例

### Hysteria2（简单）

URL: `hysteria2://password@1.2.3.4:443/?insecure=1&sni=example.com#节点名`

```toml
[protocol]
schemes = ["hysteria2", "hy2"]

[decode]
fragment = "urldecode"
```

fragment 需要 urldecode 因为节点名可能含中文。

### VMess（base64 + JSON）

URL: `vmess://base64(json)#name`

```toml
[protocol]
schemes = ["vmess", "vmess1"]

[decode]
userinfo = "base64 | json"    # 先 base64 解码，再 JSON 解析
fragment = "urldecode"
```

userinfo 经过 base64 解码后是 JSON，再 json 解析后模板里可以用 `{{ node.userinfo.add }}` 取 address。

### Shadowsocks（base64 + split）

URL: `ss://base64(method:password)@host:port#name`

```toml
[protocol]
schemes = ["ss"]

[decode]
userinfo = "base64 | split(:)"   # base64 解码后按 : 切开
fragment = "urldecode"
```

模板里 `{{ node.userinfo[0] }}` 是 cipher，`{{ node.userinfo[1] }}` 是 password。

## 新增协议

加一个 `protocols/myproto.toml`：

```toml
[protocol]
schemes = ["myproto"]

[decode]
userinfo = "base64"
fragment = "urldecode"
```

然后加对应的模板片段（见 **templates** skill）。不需要改任何 Rust 代码。
