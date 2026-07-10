{{ node.fragment | default(value=node.host) }} = ss, {{ node.host }}, {{ node.port }}, encrypt-method={{ node.userinfo[0] }}, password={{ node.userinfo[1] }}
