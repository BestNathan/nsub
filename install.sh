#!/usr/bin/env bash
set -eu

APP="nsub"
REPO="BestNathan/nsub"
INSTALL_DIR="${HOME}/.nsub"

# ── Parse args ──────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --global) INSTALL_DIR="/usr/local" ;;
        --dir)    INSTALL_DIR="$2"; shift ;;
        --ver)    VERSION="$2"; shift ;;
        *) echo "Usage: $0 [--global] [--dir PATH] [--ver vX.Y.Z]"; exit 1 ;;
    esac
    shift
done

# ── Detect platform ─────────────────────────────────────────────
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

# ── Get version ─────────────────────────────────────────────────
if [ -z "${VERSION:-}" ]; then
    echo "→ Fetching latest release..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
        | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\(.*\)".*/\1/' || true)
    [ -z "$VERSION" ] && VERSION="v0.1.0"
fi

echo "→ Installing ${APP} ${VERSION} (${TARGET})..."

# ── Download ────────────────────────────────────────────────────
BIN_DIR="${INSTALL_DIR}/bin"
SHARE_DIR="${INSTALL_DIR}/share/${APP}"
mkdir -p "${BIN_DIR}" "${SHARE_DIR}"

ARCHIVE="${APP}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

TMP=$(mktemp -d)
trap 'rm -rf $TMP' EXIT

echo "→ Downloading ${URL}"
curl -fsSL "$URL" -o "${TMP}/${ARCHIVE}" || {
    echo "  Failed. Trying gnu target..."
    TARGET="x86_64-unknown-linux-gnu"
    ARCHIVE="${APP}-${TARGET}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"
    curl -fsSL "$URL" -o "${TMP}/${ARCHIVE}"
}

# ── Extract ─────────────────────────────────────────────────────
tar xzf "${TMP}/${ARCHIVE}" -C "${TMP}"

# Install the real binary (hidden name)
cp "${TMP}/${APP}" "${BIN_DIR}/.${APP}-bin" 2>/dev/null || true
# Windows .exe fallback
cp "${TMP}/${APP}.exe" "${BIN_DIR}/.${APP}-bin" 2>/dev/null || true
chmod +x "${BIN_DIR}/.${APP}-bin" 2>/dev/null || true

# Install assets
for dir in templates protocols rules functions; do
    [ -d "${TMP}/${dir}" ] && cp -r "${TMP}/${dir}" "${SHARE_DIR}/"
done

# ── Create launcher ─────────────────────────────────────────────
cat > "${BIN_DIR}/${APP}" << EOF
#!/usr/bin/env bash
exec "${BIN_DIR}/.${APP}-bin" \\
    --template-dir "${SHARE_DIR}/templates" \\
    --protocol-dir "${SHARE_DIR}/protocols" \\
    --rules-dir "${SHARE_DIR}/rules" \\
    "\$@"
EOF
chmod +x "${BIN_DIR}/${APP}"

# ── Done ────────────────────────────────────────────────────────
echo ""
echo "✅ ${APP} ${VERSION} installed"
echo ""
if ! echo "$PATH" | grep -q "${BIN_DIR}"; then
    echo "   Add to PATH:  export PATH=\"${BIN_DIR}:\$PATH\""
    echo ""
fi
echo "   Try: ${BIN_DIR}/${APP} list templates"
echo "   Try: ${BIN_DIR}/${APP} list rules"
echo "   Try: ${BIN_DIR}/${APP} list protocols"
