{# templates/surge/config.tpl — Surge 完整配置 #}
{% for gname, g in group %}{% if gname == "📊 订阅信息" %}{% for node in g.nodes %}
# {{ gname }}: {{ node.fragment }}
{% endfor %}{% endif %}{% endfor %}
[General]
loglevel = notify

[Proxy]
{% for group_name, g in group %}
{%- if group_name != "📊 订阅信息" %}
{#- ── {{ group_name }} ── #}
{% for node in g.nodes %}
{{ node.proxy }}
{% endfor %}
{% endif %}
{%- endfor %}

[Proxy Group]
{% for group_name, g in group %}
{%- if group_name != "📊 订阅信息" %}
{{ group_name }} = select, {% for node in g.nodes %}{{ node.fragment | default(value=node.host) }}{% if not loop.last %}, {% endif %}{% endfor %}
{% endif %}
{%- endfor %}

[Rule]
FINAL,DIRECT
