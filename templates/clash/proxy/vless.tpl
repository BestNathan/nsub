- name: {{ node.fragment | default(value=node.host) }}
  type: vless
  server: "{{ node.host }}"
  port: {{ node.port }}
  uuid: {{ node.userinfo }}
  {%- if node.query.type and node.query.type != "tcp" %}
  network: {{ node.query.type }}
  {%- endif %}
  {%- if node.query.security == "reality" %}
  tls: true
  servername: "{{ node.query.sni }}"
  {%- if node.query.flow %}
  flow: {{ node.query.flow }}
  {%- endif %}
  {%- if node.query.fp %}
  fingerprint: {{ node.query.fp }}
  {%- endif %}
  reality-opts:
    public-key: {{ node.query.pbk }}
    short-id: {{ node.query.sid }}
  {%- elif node.query.security == "tls" %}
  tls: true
  {%- if node.query.sni %}
  servername: "{{ node.query.sni }}"
  {%- endif %}
  {%- if node.query.fp %}
  fingerprint: {{ node.query.fp }}
  {%- endif %}
  {%- if node.query.type == "ws" %}
  ws-opts:
    {%- if node.query.path %}
    path: "{{ node.query.path }}"
    {%- endif %}
    {%- if node.query.host %}
    headers:
      Host: "{{ node.query.host }}"
    {%- endif %}
  {%- endif %}
  {%- endif %}
