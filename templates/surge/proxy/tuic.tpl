{{ node.fragment | default(value=node.host) }} = tuic, {{ node.host }}, {{ node.port }}, username={{ node.userinfo | split(pat=":") | first }}, password={{ node.userinfo | split(pat=":") | last }}
