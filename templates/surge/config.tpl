{# templates/surge/config.tpl — Surge 完整配置 #}
[General]
loglevel = notify

[Proxy]
{% for group_name, g in group %}
{#- ── {{ group_name }} ── #}
{% for node in g.nodes %}
{% include "surge/proxy.tpl" %}
{% endfor %}
{% endfor %}

[Proxy Group]
{% for group_name, g in group %}
{{ group_name }} = select, {% for node in g.nodes %}{{ node.fragment | default(value=node.host) }}{% if not loop.last %}, {% endif %}{% endfor %}
{% endfor %}

[Rule]
FINAL,DIRECT
