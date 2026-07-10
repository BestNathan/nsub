{# templates/clash/by-source.tpl — 按订阅来源分组输出 #}
{% set_global fallback = "DIRECT" %}
{% for pipe_name, p in pipeline %}
{% if p.nodes | length > 0 and fallback == "DIRECT" %}
{% set_global fallback = pipe_name %}
{% break %}
{% endif %}
{% endfor %}

{% for gname, g in group %}{% if g.nodes | length > 0 %}{% for node in g.nodes %}
# {{ g.name }}: {{ node.fragment }}
{% endfor %}{% endif %}{% endfor %}
mixed-port: 7890
allow-lan: true
mode: rule
log-level: info

proxies:
{% for pipe_name, p in pipeline %}
# ── {{ p.name }} ({{ p.nodes | length }} 个节点) ──
{% for node in p.nodes %}
{{ node.proxy }}
{% endfor %}
{% endfor %}

proxy-groups:
{% for pipe_name, p in pipeline %}
{% if p.nodes | length > 0 %}
  - name: {{ p.name }}
    type: url-test
    url: http://www.gstatic.com/generate_204
    interval: 300
    proxies:
    {%- for node in p.nodes %}
      - {{ node.fragment }}
    {%- endfor %}
{% endif %}
{% endfor %}

rules:
  - GEOIP,CN,DIRECT
  - IP-CIDR,127.0.0.0/8,DIRECT
  - IP-CIDR,10.0.0.0/8,DIRECT
  - IP-CIDR,172.16.0.0/12,DIRECT
  - IP-CIDR,192.168.0.0/16,DIRECT
  - MATCH,{{ fallback }}
