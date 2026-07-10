- name: {{ node.fragment | default(value=node.host) }}
  type: trojan
  server: "{{ node.host }}"
  port: {{ node.port }}
  password: {{ node.userinfo }}
  {%- if node.query.sni and node.query.sni != "localhost" %}
  sni: "{{ node.query.sni }}"
  {%- endif %}
