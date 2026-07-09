{# templates/surge/proxy.tpl — 单条 Surge proxy #}
{%- if node.scheme == "ss" -%}
{{ node.fragment | default(value=node.host) }} = ss, {{ node.host }}, {{ node.port }}, encrypt-method={{ node.userinfo[0] }}, password={{ node.userinfo[1] }}
{%- elif node.scheme == "vmess" -%}
{{ node.fragment | default(value=node.userinfo.add) }} = vmess, {{ node.userinfo.add | default(value=node.host) }}, {{ node.userinfo.port | default(value=node.port) }}, username={{ node.userinfo.id }}{% if node.userinfo.net == "ws" %}, ws=true{% endif %}{% if node.userinfo.tls == "tls" %}, tls=true{% endif %}
{%- elif node.scheme == "trojan" -%}
{{ node.fragment | default(value=node.host) }} = trojan, {{ node.host }}, {{ node.port }}, password={{ node.userinfo }}{% if node.query.sni %}, sni={{ node.query.sni }}{% endif %}
{%- elif node.scheme == "vless" -%}
{{ node.fragment | default(value=node.host) }} = vless, {{ node.host }}, {{ node.port }}, username={{ node.userinfo }}
{%- elif node.scheme == "hysteria2" or node.scheme == "hy2" -%}
{{ node.fragment | default(value=node.host) }} = hysteria2, {{ node.host }}, {{ node.port }}, password={{ node.userinfo }}{% if node.query.sni %}, sni={{ node.query.sni }}{% endif %}
{%- elif node.scheme == "tuic" -%}
{{ node.fragment | default(value=node.host) }} = tuic, {{ node.host }}, {{ node.port }}, username={{ node.userinfo | split(pat=":") | first }}, password={{ node.userinfo | split(pat=":") | last }}
{%- endif -%}
