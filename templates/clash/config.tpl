{# templates/clash/config.tpl #}
{% set_global fallback = "DIRECT" %}
{% for pipe_name, p in pipeline %}
{% if p.nodes | length > 0 and fallback == "DIRECT" %}
{% set_global fallback = pipe_name %}
{% endif %}
{% endfor %}

mixed-port: 7890
allow-lan: true
mode: rule
log-level: info

proxies:
{% for pipe_name, p in pipeline %}
{% for node in p.nodes %}
{{ node.proxy }}
{% endfor %}
{% endfor %}

proxy-groups:
{% for pipe_name, p in pipeline %}
{% if p.nodes | length > 0 %}
  - name: {{ p.name }}
    type: select
    proxies:
    {%- for node in p.nodes %}
      - {{ node.fragment }}
    {%- endfor %}

  - name: {{ p.name }} - 自动
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
  # Google
  - DOMAIN-SUFFIX,google.com,{{ fallback }} - 自动
  - DOMAIN-SUFFIX,googleapis.com,{{ fallback }} - 自动
  - DOMAIN-SUFFIX,gstatic.com,{{ fallback }} - 自动

  # YouTube
  - DOMAIN-SUFFIX,youtube.com,{{ fallback }} - 自动
  - DOMAIN-SUFFIX,ytimg.com,{{ fallback }} - 自动
  - DOMAIN-SUFFIX,googlevideo.com,{{ fallback }} - 自动

  # GitHub
  - DOMAIN-SUFFIX,github.com,{{ fallback }} - 自动
  - DOMAIN-SUFFIX,githubusercontent.com,{{ fallback }} - 自动

  # Twitter / X
  - DOMAIN-SUFFIX,twitter.com,{{ fallback }} - 自动
  - DOMAIN-SUFFIX,x.com,{{ fallback }} - 自动

  # OpenAI
  - DOMAIN-SUFFIX,openai.com,{{ fallback }} - 自动
  - DOMAIN-SUFFIX,chatgpt.com,{{ fallback }} - 自动

  # Netflix / Disney+
  - DOMAIN-SUFFIX,netflix.com,{{ fallback }} - 自动
  - DOMAIN-SUFFIX,disneyplus.com,{{ fallback }} - 自动

  # 局域网直连
  - DOMAIN-SUFFIX,local,DIRECT
  - IP-CIDR,127.0.0.0/8,DIRECT
  - IP-CIDR,10.0.0.0/8,DIRECT
  - IP-CIDR,172.16.0.0/12,DIRECT
  - IP-CIDR,192.168.0.0/16,DIRECT

  # 兜底走第一个可用代理组
  - MATCH,{{ fallback }} - 自动
