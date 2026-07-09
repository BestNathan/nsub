{# templates/node/config.tpl — 中间节点格式（JSON 输出） #}
[
{% for node in nodes %}
  {
    "scheme": {{ node.scheme | json_encode }},
    "userinfo": {{ node.userinfo | json_encode }},
    "host": {{ node.host | json_encode }},
    "port": {{ node.port }},
    "query": {{ node.query | json_encode }},
    "fragment": {{ node.fragment | json_encode }},
    "raw": {{ node.raw | json_encode }}
  }{% if not loop.last %},{% endif %}
{% endfor %}
]
