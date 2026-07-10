#!/usr/bin/env bash
set -eu

APP="nsub"
REPO="BestNathan/nsub"

# ── Default to XDG user dirs, --global → /usr/local ──────────────
BIN_DIR="${HOME}/.local/bin"
SHARE_DIR="${HOME}/.local/share/${APP}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --global) BIN_DIR="/usr/local/bin"; SHARE_DIR="/usr/local/share/${APP}" ;;
        --ver)    VERSION="$2"; shift ;;
        *) echo "Usage: $0 [--global] [--ver vX.Y.Z]"; exit 1 ;;
    esac
    shift
done

# ── Detect platform ───────────────────────────────────────────────
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Linux)  OS_GO="linux" ;;
    Darwin) OS_GO="darwin" ;;
    *) echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64|amd64)   ARCH_TRIPLE="x86_64" ;;
    aarch64|arm64)  ARCH_TRIPLE="aarch64" ;;
    *) echo "Unsupported arch: $ARCH"; exit 1 ;;
esac

case "${OS_GO}-${ARCH_TRIPLE}" in
    linux-x86_64)   TARGET="x86_64-unknown-linux-musl" ;;
    linux-aarch64)  TARGET="aarch64-unknown-linux-gnu" ;;
    darwin-x86_64)  TARGET="x86_64-apple-darwin" ;;
    darwin-aarch64) TARGET="aarch64-apple-darwin" ;;
esac

# ── Get target version + release notes ────────────────────────────
if [ -z "${VERSION:-}" ]; then
    echo "→ Fetching latest release..."
    RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null || true)
    VERSION=$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\(.*\)".*/\1/' || true)
    if [ -z "$VERSION" ]; then
        VERSION=$(git ls-remote --tags --sort=-version:refname \
            "https://github.com/${REPO}.git" 'v*' 2>/dev/null \
            | head -1 | sed 's/.*refs\/tags\/\(v.*\)/\1/' || true)
    fi
    if [ -z "$VERSION" ]; then
        VERSION=$(git ls-remote --tags "https://github.com/${REPO}.git" 'v*' 2>/dev/null \
            | sed 's/.*refs\/tags\/\(v.*\)/\1/' | sort -Vr | head -1 || true)
    fi
    [ -z "$VERSION" ] && { echo "Failed to detect latest version"; exit 1; }
else
    RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/tags/${VERSION}" 2>/dev/null || true)
fi

# ── Show release notes ────────────────────────────────────────────
echo "→ ${APP} ${VERSION} (${TARGET})"
if [ -n "${RELEASE_JSON:-}" ]; then
    BODY=$(echo "$RELEASE_JSON" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    print(data.get('body', ''))
except: pass
" 2>/dev/null || true)
    if [ -n "$BODY" ]; then
        echo ""
        echo "$BODY"
        echo ""
    fi
fi

echo "→ Binary : ${BIN_DIR}"
echo "→ Assets : ${SHARE_DIR}"

# ── Download ──────────────────────────────────────────────────────
ARCHIVE="${APP}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

TMP=$(mktemp -d)
trap 'rm -rf $TMP' EXIT

echo "→ Downloading ${URL}"
HTTP_CODE=$(curl -fsSL -w "%{http_code}" -o "${TMP}/${ARCHIVE}" "$URL" 2>/dev/null || true)
if [ "$HTTP_CODE" != "200" ]; then
    echo "  HTTP ${HTTP_CODE}, retry with gnu target..."
    TARGET="x86_64-unknown-linux-gnu"
    ARCHIVE="${APP}-${TARGET}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"
    curl -fsSL -o "${TMP}/${ARCHIVE}" "$URL" || {
        echo "  Failed to download ${APP} ${VERSION} for ${TARGET}"
        exit 1
    }
fi

# ── Extract & install ─────────────────────────────────────────────
tar xzf "${TMP}/${ARCHIVE}" -C "${TMP}"

cp "${TMP}/${APP}" "${BIN_DIR}/${APP}"
chmod +x "${BIN_DIR}/${APP}"
for dir in templates protocols rules functions skills; do
    [ -d "${TMP}/${dir}" ] && cp -r "${TMP}/${dir}" "${SHARE_DIR}/"
done

# ── Done ──────────────────────────────────────────────────────────
# Create user skeleton directories
USER_NSUB="${HOME}/.nsub"
for dir in templates protocols rules functions skills; do
    mkdir -p "${USER_NSUB}/${dir}"
done

cat <<EOF

✅ ${APP} ${VERSION} installed

   nsub \\
     --template-dir ${SHARE_DIR}/templates \\
     --protocol-dir ${SHARE_DIR}/protocols \\
     --rules-dir     ${SHARE_DIR}/rules \\
     ...

   💡 用户自定义目录: ${USER_NSUB}/
      放置自定义协议/模板/规则/函数/扩展指南，
      优先级高于安装目录，可覆盖默认资源。

      示例:
        ${USER_NSUB}/protocols/my-custom.toml   →  nsub convert 自动加载
        ${USER_NSUB}/templates/clash/proxy/xxx.tpl  →  覆盖安装模板
        ${USER_NSUB}/rules/my-rules.toml        →  覆盖安装规则
EOF
if ! echo "$PATH" | grep -qF "${BIN_DIR}"; then
    cat <<EOF

   Add to PATH:  export PATH="${BIN_DIR}:\$PATH"
EOF
fi
echo
