{{ node.fragment | default(value=node.host) }} = vless, {{ node.host }}, {{ node.port }}, username={{ node.userinfo }}
