{# templates/clash/proxy.tpl — 按 scheme 分发到对应协议模板 #}
{# 新增协议：在下方加一个 elif，然后在 proxy/ 目录加 <scheme>.tpl 文件 #}
{%- if node.scheme == "vmess" -%}
{% include "clash/proxy/vmess.tpl" %}
{%- elif node.scheme == "ss" -%}
{% include "clash/proxy/ss.tpl" %}
{%- elif node.scheme == "trojan" -%}
{% include "clash/proxy/trojan.tpl" %}
{%- elif node.scheme == "vless" -%}
{% include "clash/proxy/vless.tpl" %}
{%- elif node.scheme == "hysteria2" or node.scheme == "hy2" -%}
{% include "clash/proxy/hysteria2.tpl" %}
{%- elif node.scheme == "tuic" -%}
{% include "clash/proxy/tuic.tpl" %}
{%- endif -%}
