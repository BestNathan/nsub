{{ node.fragment | default(value=node.host) }} = hysteria2, {{ node.host }}, {{ node.port }}, password={{ node.userinfo }}{% if node.query.sni %}, sni={{ node.query.sni }}{% endif %}
