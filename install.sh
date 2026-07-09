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

# ── Get version ───────────────────────────────────────────────────
if [ -z "${VERSION:-}" ]; then
    echo "→ Fetching latest release..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
        | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\(.*\)".*/\1/' || true)
    [ -z "$VERSION" ] && VERSION="v0.1.0"
fi

echo "→ Installing ${APP} ${VERSION} (${TARGET})..."
echo "→ Binary : ${BIN_DIR}"
echo "→ Assets : ${SHARE_DIR}"

# ── Download ──────────────────────────────────────────────────────
ARCHIVE="${APP}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"

TMP=$(mktemp -d)
trap 'rm -rf $TMP' EXIT

echo "→ Downloading ${URL}"
curl -fsSL "$URL" -o "${TMP}/${ARCHIVE}" || {
    echo "  Retry with gnu target..."
    TARGET="x86_64-unknown-linux-gnu"
    ARCHIVE="${APP}-${TARGET}.tar.gz"
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"
    curl -fsSL "$URL" -o "${TMP}/${ARCHIVE}"
}

# ── Extract & install ─────────────────────────────────────────────
tar xzf "${TMP}/${ARCHIVE}" -C "${TMP}"

install -m 755 "${TMP}/${APP}" "${BIN_DIR}/${APP}"
cp -r "${TMP}/templates" "${SHARE_DIR}/"
cp -r "${TMP}/protocols" "${SHARE_DIR}/"
cp -r "${TMP}/rules" "${SHARE_DIR}/"
cp -r "${TMP}/functions" "${SHARE_DIR}/" 2>/dev/null || true

# ── Done ──────────────────────────────────────────────────────────
echo ""
echo "✅ ${APP} ${VERSION} installed"
echo ""
echo "   nsub \\
echo "     --template-dir ${SHARE_DIR}/templates \\"
echo "     --protocol-dir ${SHARE_DIR}/protocols \\"
echo "     --rules-dir     ${SHARE_DIR}/rules \\"
echo "     ..."
echo ""
if ! echo "$PATH" | grep -qF "${BIN_DIR}"; then
    echo "   Add to PATH:  export PATH=\"${BIN_DIR}:\$PATH\""
    echo ""
fi
