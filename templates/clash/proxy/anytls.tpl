- name: {{ node.fragment }}
  type: anytls
  server: "{{ node.host }}"
  port: {{ node.port }}
  password: {{ node.userinfo }}
  {%- if node.query.sni and node.query.sni != "localhost" %}
  sni: "{{ node.query.sni }}"
  {%- endif %}
  {%- if node.query.insecure == "1" or node.query.insecure == "true" %}
  skip-cert-verify: true
  {%- endif %}
