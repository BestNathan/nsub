{# templates/clash/grouped.tpl — 按地区分组 + GEOIP 规则 #}
{% set_global fallback = "DIRECT" %}
{% for pipe_name, p in pipeline %}
{% if p.nodes | length > 0 and fallback == "DIRECT" %}
{% set_global fallback = pipe_name %}
{% break %}
{% endif %}
{% endfor %}

mixed-port: 7890
allow-lan: true
mode: rule
log-level: info

proxies:
{% for pipe_name, p in pipeline %}
{% for node in p.nodes %}
{% include "clash/proxy.tpl" %}
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
  # 国内IP直连
  - GEOIP,CN,DIRECT
  # 局域网
  - IP-CIDR,127.0.0.0/8,DIRECT
  - IP-CIDR,10.0.0.0/8,DIRECT
  - IP-CIDR,172.16.0.0/12,DIRECT
  - IP-CIDR,192.168.0.0/16,DIRECT
  # 其余走代理
  - MATCH,{{ fallback }}
