# Templates — 模板渲染

## 概述

`templates/` 目录存放 [Tera](https://keats.github.io/tera/) 模板文件（`.tpl` 后缀），控制最终配置文件的输出格式。nsub 不硬编码任何输出格式，所有 Clash/Surge 等客户端的配置结构由模板定义。

## 文件位置

```
templates/
  clash/
    proxy.tpl         ← 分发器：按 scheme include 子模板
    proxy/
      vmess.tpl       ← VMess 单条节点
      vless.tpl       ← VLESS (reality / tls+ws)
      ss.tpl          ← Shadowsocks
      trojan.tpl      ← Trojan
      hysteria2.tpl   ← Hysteria2 / hy2
      tuic.tpl        ← TUIC
      hy2.tpl         ← hysteria2 别名
    simple.tpl        ← 全节点 round-robin
    grouped.tpl       ← 按 GEOIP 分组
    by-source.tpl     ← 按订阅来源分组
    config.tpl        ← 域名规则路由
  surge/
    proxy.tpl         ← 分发器
    proxy/            ← (同上结构)
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

## 单条 Proxy 模板 (proxy/<scheme>.tpl)

每个协议的 proxy 配置独立存放在 `proxy/` 子目录，按 `node.scheme` 自动匹配。
模板里直接用 `{{ node.proxy }}` 输出预渲染好的 proxy 配置，**不需要写分发逻辑**。

### Clash 单条 proxy 模板 (`proxy/vmess.tpl`)

```django
- name: {{ node.fragment | default(value=node.host) }}
  type: vmess
  server: "{{ node.userinfo.add | default(value=node.host) }}"
  port: {{ node.userinfo.port | default(value=node.port) }}
  uuid: {{ node.userinfo.id }}
  alterId: {{ node.userinfo.aid | default(value=0) }}
  cipher: auto
```

### 主模板里怎么用

```django
proxies:
{% for p in pipeline %}
{% for node in p.nodes %}
{{ node.proxy }}          {# 自动选择 proxy/<scheme>.tpl 渲染 #}
{% endfor %}
{% endfor %}
```

`node` 的其他字段（`fragment`, `host`, `source` 等）依然可以直接访问，用于 proxy-groups 等场景。
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

只需加一个文件 `templates/clash/proxy/myproto.tpl`：

```django
- name: {{ node.fragment }}
  type: myproto
  server: "{{ node.host }}"
  port: {{ node.port }}
  password: {{ node.userinfo }}
```

引擎自动按 `node.scheme` 匹配文件，**不需要修改任何其他文件**。

Surge 同理：`templates/surge/proxy/myproto.tpl`。

---

> 需要帮助？[提 Issue](https://github.com/BestNathan/nsub/issues/new?labels=template&template=template.yml)
