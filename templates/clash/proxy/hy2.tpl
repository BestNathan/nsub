- name: {{ node.fragment }}
  type: hysteria2
  server: "{{ node.host }}"
  port: {{ node.port }}
  password: {{ node.userinfo }}
  {%- if node.query.sni and node.query.sni != "localhost" %}
  sni: "{{ node.query.sni }}"
  {%- endif %}
  {%- if node.query.insecure == "1" %}
  skip-cert-verify: true
  {%- endif %}
