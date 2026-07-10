- name: {{ node.fragment | default(value=node.host) }}
  type: tuic
  server: "{{ node.host }}"
  port: {{ node.port }}
  uuid: {{ node.userinfo | split(pat=":") | first }}
  password: {{ node.userinfo | split(pat=":") | last }}
