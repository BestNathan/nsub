- name: {{ node.fragment | default(value=node.host) }}
  type: ss
  server: "{{ node.host }}"
  port: {{ node.port }}
  cipher: {{ node.userinfo[0] }}
  password: {{ node.userinfo[1] }}
