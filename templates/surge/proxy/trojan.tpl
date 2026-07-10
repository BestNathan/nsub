{{ node.fragment | default(value=node.host) }} = trojan, {{ node.host }}, {{ node.port }}, password={{ node.userinfo }}{% if node.query.sni %}, sni={{ node.query.sni }}{% endif %}
