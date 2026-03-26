#!/bin/sh
set -eu

REPO="Zackriya-Solutions/syncfu"
BINARY_NAME="syncfu"

# --- Colors (only if terminal) ---
if [ -t 1 ]; then
  BOLD='\033[1m' GREEN='\033[32m' CYAN='\033[36m' RED='\033[31m' RESET='\033[0m'
else
  BOLD='' GREEN='' CYAN='' RED='' RESET=''
fi

info()  { printf "${GREEN}info${RESET}  %s\n" "$@"; }
warn()  { printf "${CYAN}warn${RESET}  %s\n" "$@"; }
error() { printf "${RED}error${RESET} %s\n" "$@" >&2; exit 1; }

# --- Parse arguments ---
VERSION=""
for arg in "$@"; do
  case "$arg" in
    --version=*) VERSION="${arg#--version=}" ;;
    --help|-h)
      printf "Usage: install.sh [--version=X.Y.Z]\n"
      printf "\nInstalls the syncfu CLI from GitHub Releases.\n"
      printf "\nOptions:\n"
      printf "  --version=X.Y.Z  Install a specific version (default: latest)\n"
      printf "\nEnvironment:\n"
      printf "  SYNCFU_INSTALL_DIR  Override install directory\n"
      exit 0
      ;;
  esac
done

# --- Detect platform ---
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
  Darwin) OS_NAME="darwin" ;;
  Linux)  OS_NAME="linux" ;;
  MINGW*|MSYS*|CYGWIN*)
    error "Use PowerShell on Windows: irm https://raw.githubusercontent.com/${REPO}/main/install.ps1 | iex" ;;
  *) error "Unsupported OS: $OS" ;;
esac

case "$ARCH" in
  x86_64|amd64)   ARCH_NAME="x86_64" ;;
  arm64|aarch64)   ARCH_NAME="arm64" ;;
  *) error "Unsupported architecture: $ARCH" ;;
esac

ARTIFACT="${BINARY_NAME}-${OS_NAME}-${ARCH_NAME}"

# --- Require curl ---
command -v curl >/dev/null 2>&1 || error "curl is required but not installed. Install it with your package manager."

# --- Resolve version ---
if [ -z "$VERSION" ]; then
  info "Resolving latest version..."
  VERSION=$(curl -fsSL -o /dev/null -w '%{redirect_url}' \
    "https://github.com/${REPO}/releases/latest" 2>/dev/null | \
    sed 's|.*/v||')
  if [ -z "$VERSION" ]; then
    error "Could not determine latest version. Try: --version=0.2.0"
  fi
fi

# --- Validate version format ---
case "$VERSION" in
  [0-9]*.[0-9]*.[0-9]*) ;;
  *) error "Unexpected version format: $VERSION" ;;
esac

# --- Install directory ---
if [ -n "${SYNCFU_INSTALL_DIR:-}" ]; then
  INSTALL_DIR="$SYNCFU_INSTALL_DIR"
elif [ -w /usr/local/bin ]; then
  INSTALL_DIR="/usr/local/bin"
else
  INSTALL_DIR="$HOME/.syncfu/bin"
fi

# --- Download ---
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ARTIFACT}"
CHECKSUM_URL="https://github.com/${REPO}/releases/download/v${VERSION}/checksums.txt"

WORK_DIR=$(mktemp -d "${TMPDIR:-/tmp}/syncfu-install.XXXXXXXXXX")
trap 'rm -rf "$WORK_DIR"' EXIT

printf "\n"
printf "  ${BOLD}syncfu${RESET} installer\n"
printf "\n"
printf "  ${CYAN}Version:${RESET}  v%s\n" "$VERSION"
printf "  ${CYAN}Platform:${RESET} %s/%s\n" "$OS_NAME" "$ARCH_NAME"
printf "  ${CYAN}Install:${RESET}  %s\n" "$INSTALL_DIR"
printf "\n"

info "Downloading syncfu v${VERSION}..."
HTTP_CODE=$(curl -sL -w '%{http_code}' -o "${WORK_DIR}/${BINARY_NAME}" "$URL" 2>/dev/null || true)
if [ "$HTTP_CODE" != "200" ]; then
  error "Download failed (HTTP ${HTTP_CODE:-???}). Check: https://github.com/${REPO}/releases/tag/v${VERSION}"
fi

# --- Verify checksum ---
info "Verifying checksum..."
if curl -fsSL -o "${WORK_DIR}/checksums.txt" "$CHECKSUM_URL" 2>/dev/null; then
  EXPECTED=$(grep "${ARTIFACT}" "${WORK_DIR}/checksums.txt" | awk '{print $1}')
  if [ -n "$EXPECTED" ]; then
    if command -v sha256sum >/dev/null 2>&1; then
      ACTUAL=$(sha256sum "${WORK_DIR}/${BINARY_NAME}" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
      ACTUAL=$(shasum -a 256 "${WORK_DIR}/${BINARY_NAME}" | awk '{print $1}')
    else
      warn "No sha256sum or shasum found — cannot verify download integrity"
      EXPECTED=""
    fi
    if [ -n "$EXPECTED" ] && [ "$EXPECTED" != "$ACTUAL" ]; then
      error "Checksum mismatch!\n  Expected: ${EXPECTED}\n  Got:      ${ACTUAL}"
    fi
    if [ -n "$EXPECTED" ]; then
      info "Checksum verified"
    fi
  else
    warn "Artifact not found in checksums.txt, skipping verification"
  fi
else
  warn "Could not download checksums.txt, skipping verification"
fi

# --- Install ---
chmod +x "${WORK_DIR}/${BINARY_NAME}"

# Remove macOS quarantine attribute
if [ "$OS_NAME" = "darwin" ]; then
  xattr -d com.apple.quarantine "${WORK_DIR}/${BINARY_NAME}" 2>/dev/null || true
fi

mkdir -p "$INSTALL_DIR"
mv "${WORK_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"

info "Installed to ${INSTALL_DIR}/${BINARY_NAME}"

# --- PATH instructions ---
case ":${PATH:-}:" in
  *":${INSTALL_DIR}:"*)
    # Already in PATH
    ;;
  *)
    printf "\n"
    warn "Add syncfu to your PATH:"
    printf "\n"

    SHELL_NAME=$(basename "${SHELL:-/bin/sh}")
    case "$SHELL_NAME" in
      zsh)
        printf "  echo 'export PATH=\"%s:\$PATH\"' >> ~/.zshrc && source ~/.zshrc\n" "$INSTALL_DIR"
        ;;
      bash)
        printf "  echo 'export PATH=\"%s:\$PATH\"' >> ~/.bashrc && source ~/.bashrc\n" "$INSTALL_DIR"
        ;;
      fish)
        printf "  fish_add_path %s\n" "$INSTALL_DIR"
        ;;
      *)
        printf "  export PATH=\"%s:\$PATH\"\n" "$INSTALL_DIR"
        ;;
    esac
    printf "\n"
    ;;
esac

# --- Done ---
printf "\n"
info "Done! Run ${BOLD}syncfu --help${RESET} to get started."
printf "\n"
