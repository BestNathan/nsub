- name: {{ node.fragment | default(value=node.host) }}
  type: vmess
  server: "{{ node.userinfo.add | default(value=node.host) }}"
  port: {{ node.userinfo.port | default(value=node.port) }}
  uuid: {{ node.userinfo.id }}
  alterId: {{ node.userinfo.aid | default(value=0) }}
  cipher: auto
  {%- if node.userinfo.net and node.userinfo.net != "tcp" %}
  network: {{ node.userinfo.net }}
  {%- endif %}
