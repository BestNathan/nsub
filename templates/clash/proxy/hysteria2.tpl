- name: {{ node.fragment }}
  type: hysteria2
  server: "{{ node.host }}"
  port: {{ node.port }}
  password: {{ node.userinfo }}
  {%- if node.query.sni and node.query.sni != "localhost" %}
  sni: "{{ node.query.sni }}"
  {%- endif %}
  {%- if node.query.insecure == "1" or node.query.insecure == "true" %}
  skip-cert-verify: true
  {%- endif %}
  {%- if node.query.pinSHA256 %}
  fingerprint: {{ node.query.pinSHA256 }}
  {%- endif %}
  {%- if node.query.mport %}
  ports: {{ node.query.mport }}
  {%- endif %}
