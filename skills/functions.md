# Functions — 自定义函数

## 状态

⚠️ **计划功能** — Rhai 脚本引擎尚未实现。当前仅支持内置 pipe 函数（base64, json, urldecode, split 等）。

## 概述

`functions/` 目录将存放 [Rhai](https://rhai.rs/) 脚本（`.rhai` 后缀），用于扩展 pipe 引擎。当内置 pipe 函数不够用时，写 Rhai 脚本实现自定义处理逻辑。

## 文件位置

```
functions/
  custom.rhai       ← 自定义 pipe 函数
```

## Rhai 脚本格式

每个 `.rhai` 文件定义一个或多个函数：

```rust
// functions/custom.rhai
fn my_pipe(input) {
    // input 是字符串
    // 返回值会自动转换为 serde_json::Value
    input.to_upper()
}
```

## 模板里怎么用

自定义函数名就是文件名（不含扩展名），在 `protocols/*.toml` 的 `[decode]` 配置中引用：

```toml
[decode]
userinfo = "base64 | my_pipe"
fragment = "urldecode | my_pipe"
```

## 内置函数参考

如果内置函数能满足需求，优先用内置的（性能更好）：

| 函数 | 说明 |
|------|------|
| `base64` | 标准 base64 解码 |
| `json` | JSON 字符串 → 对象 |
| `urldecode` | URL 百分号解码 |
| `lowercase` | 转小写 |
| `uppercase` | 转大写 |
| `trim` | 去掉首尾空白 |
| `split(:)` | 按冒号切分为数组 |
| `split(;)` | 按分号切分 |
| `split(<delim>)` | 泛型分隔符切分 |
| `lines` | 按换行切分为行数组 |

## 示例

### 示例1：提取端口范围

```rust
// functions/port_range.rhai
fn first_port(input) {
    // input: "50000-53000" → output: "50000"
    let parts = input.split('-');
    parts[0]
}
```

```toml
# protocols/myproto.toml
[decode]
userinfo = "base64 | first_port"
```

### 示例2：URL 参数解析

```rust
// functions/parse_params.rhai
fn extract_host(input) {
    // input 是逗号分隔的 key=val 字符串
    // 提取 host=xxx 中的值
    let pairs = input.split(',');
    for pair in pairs {
        let kv = pair.split('=');
        if kv.len() >= 2 && kv[0].trim() == "host" {
            return kv[1].trim();
        }
    }
    ""
}
```

### 示例3：拼接字段

```rust
// functions/build_note.rhai
fn build_note(input) {
    // 拼接备注信息
    `[${input}] 来自 Rhai 自定义函数`
}
```

## 限制

- Rhai 脚本运行在沙箱中，不能访问文件系统或网络
- 函数签名固定：接收一个 `&str` 参数，返回值转为 `serde_json::Value`
- 不支持 `async`
- 性能低于内置函数，大量节点时注意

## 新增函数

加一个 `functions/myfunc.rhai`：

```rust
fn my_transform(input) {
    // 自定义逻辑
    input.replace("old", "new")
}
```

在协议定义中引用：

```toml
[decode]
userinfo = "base64 | my_transform"
```

不需要改任何 Rust 代码。
