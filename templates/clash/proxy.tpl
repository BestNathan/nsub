{# templates/clash/proxy.tpl — 单条 Clash proxy #}
{%- if node.scheme == "vmess" -%}
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
{%- elif node.scheme == "ss" -%}
- name: {{ node.fragment | default(value=node.host) }}
  type: ss
  server: "{{ node.host }}"
  port: {{ node.port }}
  cipher: {{ node.userinfo[0] }}
  password: {{ node.userinfo[1] }}
{%- elif node.scheme == "trojan" -%}
- name: {{ node.fragment | default(value=node.host) }}
  type: trojan
  server: "{{ node.host }}"
  port: {{ node.port }}
  password: {{ node.userinfo }}
  {%- if node.query.sni and node.query.sni != "localhost" %}
  sni: "{{ node.query.sni }}"
  {%- endif %}
{%- elif node.scheme == "vless" -%}
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
{%- elif node.scheme == "hysteria2" or node.scheme == "hy2" -%}
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
{%- elif node.scheme == "tuic" -%}
- name: {{ node.fragment | default(value=node.host) }}
  type: tuic
  server: "{{ node.host }}"
  port: {{ node.port }}
  uuid: {{ node.userinfo | split(pat=":") | first }}
  password: {{ node.userinfo | split(pat=":") | last }}
{%- endif -%}
