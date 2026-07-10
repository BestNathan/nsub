{# templates/surge/proxy.tpl — 按 scheme 分发到对应协议模板 #}
{# 新增协议：加 elif + proxy/<scheme>.tpl #}
{%- if node.scheme == "ss" -%}
{% include "surge/proxy/ss.tpl" %}
{%- elif node.scheme == "vmess" -%}
{% include "surge/proxy/vmess.tpl" %}
{%- elif node.scheme == "trojan" -%}
{% include "surge/proxy/trojan.tpl" %}
{%- elif node.scheme == "vless" -%}
{% include "surge/proxy/vless.tpl" %}
{%- elif node.scheme == "hysteria2" or node.scheme == "hy2" -%}
{% include "surge/proxy/hysteria2.tpl" %}
{%- elif node.scheme == "tuic" -%}
{% include "surge/proxy/tuic.tpl" %}
{%- endif -%}
