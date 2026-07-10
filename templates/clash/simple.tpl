{# templates/clash/simple.tpl — 全节点 round-robin + 非国内IP走代理 #}
{% for gname, g in group %}{% if g.nodes | length > 0 %}{% for node in g.nodes %}
# {{ g.name }}: {{ node.fragment }}
{% endfor %}{% endif %}{% endfor %}
mixed-port: 7890
allow-lan: true
mode: rule
log-level: info

proxies:
{% for node in pipeline["all"].nodes %}
{{ node.proxy }}
{% endfor %}

proxy-groups:
  - name: 🚀 代理
    type: load-balance
    strategy: round-robin
    proxies:
    {%- for node in pipeline["all"].nodes %}
      - {{ node.fragment }}
    {%- endfor %}

rules:
  # 国内IP直连
  - GEOIP,CN,DIRECT
  # 局域网
  - IP-CIDR,127.0.0.0/8,DIRECT
  - IP-CIDR,10.0.0.0/8,DIRECT
  - IP-CIDR,172.16.0.0/12,DIRECT
  - IP-CIDR,192.168.0.0/16,DIRECT
  # 其余走代理
  - MATCH,🚀 代理
