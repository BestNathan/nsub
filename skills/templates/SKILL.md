# Templates — 模板渲染

## 概述

`templates/` 目录存放 [Tera](https://keats.github.io/tera/) 模板文件（`.tpl` 后缀），控制最终配置文件的输出格式。nsub 不硬编码任何输出格式，所有 Clash/Surge 等客户端的配置结构由模板定义。

## 文件位置

```
templates/
  clash/
    proxy.tpl         ← 单条代理节点（被其他模板 include）
    simple.tpl        ← 全节点 round-robin
    grouped.tpl       ← 按 GEOIP 分组
    by-source.tpl     ← 按订阅来源分组
    config.tpl        ← 域名规则路由
  surge/
    proxy.tpl         ← Surge 单条节点
    config.tpl        ← Surge 完整配置
  node/
    config.tpl        ← 纯节点列表（调试用）
```

## 模板可用变量

### `nodes` — 所有原始节点

```
{% for node in nodes %}
  {{ node.scheme }} {{ node.host }}:{{ node.port }}
{% endfor %}
```

### `pipeline` — 管道处理后的节点组

```yaml
# rules/simple.toml 里定义的 pipeline
[[pipeline]]
name = "all"
steps = ["dedup.main", "exclude.ad"]
```

模板中：

```
{% for pipe_name, p in pipeline %}
  - name: {{ p.name }}          # "all"
    nodes: {{ p.nodes }}        # Vec<NodeContext>
{% endfor %}
```

### `dedup` / `exclude` / `group` — 规则引擎产出

```
{% for gname, g in dedup %} ... {% endfor %}
{% for gname, g in exclude %} ... {% endfor %}
{% for gname, g in group %} ... {% endfor %}
```

每个 `g` 对象有 `name` (String) 和 `nodes` (Vec<NodeContext>)。

## Tera 语法要点

### Include 复用片段

```django
{# templates/clash/grouped.tpl #}
proxies:
{% for pipe_name, p in pipeline %}
{% for node in p.nodes %}
{% include "clash/proxy.tpl" %}    {# 复用单条 proxy 片段 #}
{% endfor %}
{% endfor %}
```

### 条件渲染

```django
{% if node.scheme == "vless" %}
  ...vless 专用字段...
{% elif node.scheme == "hysteria2" or node.scheme == "hy2" %}
  ...hysteria2 专用字段...
{% endif %}
```

### 默认值

```django
server: "{{ node.userinfo.add | default(value=node.host) }}"
```

如果 userinfo 里有 `add` 就用，否则 fallback 到 `host`。

### 设置全局变量

```django
{% set_global fallback = "DIRECT" %}
{% for pipe_name, p in pipeline %}
  {% if p.nodes | length > 0 and fallback == "DIRECT" %}
    {% set_global fallback = pipe_name %}
    {% break %}
  {% endif %}
{% endfor %}
```

用于找到第一个非空管道组作为 fallback 代理。

### 检测字符串包含

```django
{% if "[" in node.host %}
  ...IPv6 地址...
{% endif %}
```

## 单条 Proxy 模板 (proxy.tpl)

这是被所有模板 include 的核心片段。每个协议类型一个 `elif` 分支：

```django
{% if node.scheme == "vmess" %}
- name: {{ node.fragment }}
  type: vmess
  server: "{{ node.userinfo.add | default(value=node.host) }}"
  port: {{ node.userinfo.port | default(value=node.port) }}
  uuid: {{ node.userinfo.id }}
  ...

{% elif node.scheme == "vless" %}
  {% if node.query.security == "reality" %}
    ...reality 专用配置...
  {% elif node.query.security == "tls" %}
    ...tls+ws 专用配置...
  {% endif %}
{% endif %}
```

### YAML 转义注意

IPv6 地址如 `2400:8902::1` 必须加双引号：

```django
{# ✅ 正确 #}
server: "{{ node.host }}"

{# ❌ 错误 — [2400:...] 被 YAML 解析为数组 #}
server: {{ node.host }}
```

## 新增模板

### 新增输出格式

在 `templates/` 下新建目录和 `.tpl` 文件：

```
templates/
  myclient/
    proxy.tpl       ← 单条节点片段
    config.tpl      ← 完整配置（include proxy.tpl）
```

### 新增 proxy 协议支持

编辑 `templates/clash/proxy.tpl`，在末尾加一个新的 `elif`：

```django
{% elif node.scheme == "myproto" %}
- name: {{ node.fragment }}
  type: myproto
  server: "{{ node.host }}"
  port: {{ node.port }}
  password: {{ node.userinfo }}
{% endif %}
```

不需要改任何 Rust 代码。
