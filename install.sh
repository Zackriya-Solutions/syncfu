#!/bin/sh
set -eu

REPO="Zackriya-Solutions/syncfu"
APP_NAME="syncfu"
SKIP_CHECKSUM=0

# --- Colors (only if terminal) ---
if [ -t 1 ]; then
  BOLD='\033[1m' GREEN='\033[32m' CYAN='\033[36m' RED='\033[31m' YELLOW='\033[33m' RESET='\033[0m'
else
  BOLD='' GREEN='' CYAN='' RED='' YELLOW='' RESET=''
fi

info()  { printf "${GREEN}info${RESET}  %s\n" "$@"; }
warn()  { printf "${YELLOW}warn${RESET}  %s\n" "$@"; }
error() { printf "${RED}error${RESET} %s\n" "$@" >&2; exit 1; }

# --- Parse arguments ---
VERSION=""
CLI_ONLY=0
for arg in "$@"; do
  case "$arg" in
    --version=*) VERSION="${arg#--version=}" ;;
    --skip-checksum) SKIP_CHECKSUM=1 ;;
    --cli-only) CLI_ONLY=1 ;;
    --help|-h)
      printf "Usage: install.sh [--version=X.Y.Z] [--skip-checksum] [--cli-only]\n"
      printf "\nInstalls the syncfu desktop app and CLI.\n"
      printf "\nOptions:\n"
      printf "  --version=X.Y.Z  Install a specific version (default: latest)\n"
      printf "  --skip-checksum  Skip SHA-256 integrity verification for CLI binary\n"
      printf "  --cli-only       Install only the CLI (no desktop app, headless mode)\n"
      printf "\nEnvironment:\n"
      printf "  SYNCFU_INSTALL_DIR  Override CLI install directory (must be absolute path)\n"
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

# --- Require curl ---
command -v curl >/dev/null 2>&1 || error "curl is required but not installed."

# --- Resolve version ---
if [ -z "$VERSION" ]; then
  info "Resolving latest version..."
  LOCATION_HEADER=$(curl -sI "https://github.com/${REPO}/releases/latest" 2>/dev/null | grep -i '^location:' || true)
  if [ -z "$LOCATION_HEADER" ]; then
    error "Could not determine latest version. Try: --version=0.2.0"
  fi
  VERSION=$(printf '%s' "$LOCATION_HEADER" | sed 's|.*/v||' | tr -d '\r')
  if [ -z "$VERSION" ]; then
    error "Could not parse version from redirect. Try: --version=0.2.0"
  fi
fi

# --- Validate version format ---
if ! expr "$VERSION" : '[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*$' > /dev/null 2>&1; then
  error "Unexpected version format: $VERSION"
fi

# --- CLI install directory ---
if [ -n "${SYNCFU_INSTALL_DIR:-}" ]; then
  case "$SYNCFU_INSTALL_DIR" in
    /*) ;;
    *) error "SYNCFU_INSTALL_DIR must be an absolute path" ;;
  esac
  CLI_DIR="$SYNCFU_INSTALL_DIR"
elif [ -w /usr/local/bin ]; then
  CLI_DIR="/usr/local/bin"
else
  CLI_DIR="$HOME/.syncfu/bin"
fi

# --- Work directory ---
WORK_DIR=$(mktemp -d "${TMPDIR:-/tmp}/syncfu-install.XXXXXXXXXX")
trap 'rm -rf "$WORK_DIR"' EXIT

# --- Artifact names ---
CLI_ARTIFACT="${APP_NAME}-${OS_NAME}-${ARCH_NAME}"

if [ "$OS_NAME" = "darwin" ]; then
  # Tauri produces: syncfu_0.2.0_aarch64.dmg / syncfu_0.2.0_x86_64.dmg
  if [ "$ARCH_NAME" = "arm64" ]; then
    TAURI_ARCH="aarch64"
  else
    TAURI_ARCH="x86_64"
  fi
  DESKTOP_ARTIFACT="${APP_NAME}_${VERSION}_${TAURI_ARCH}.dmg"
elif [ "$OS_NAME" = "linux" ]; then
  DESKTOP_ARTIFACT="${APP_NAME}_${VERSION}_amd64.AppImage"
fi

BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"

printf "\n"
printf "  ${BOLD}syncfu${RESET} installer\n"
printf "\n"
printf "  ${CYAN}Version:${RESET}   v%s\n" "$VERSION"
printf "  ${CYAN}Platform:${RESET}  %s/%s\n" "$OS_NAME" "$ARCH_NAME"
if [ "$CLI_ONLY" = "0" ]; then
  printf "  ${CYAN}Desktop:${RESET}   yes (tray + overlay notifications)\n"
fi
printf "  ${CYAN}CLI:${RESET}       %s/syncfu\n" "$CLI_DIR"
printf "\n"

# =============================================
# 1. Install desktop app (unless --cli-only)
# =============================================
if [ "$CLI_ONLY" = "0" ]; then
  if [ "$OS_NAME" = "darwin" ]; then
    install_macos_app() {
      info "Downloading syncfu desktop app..."
      DMG_PATH="${WORK_DIR}/${DESKTOP_ARTIFACT}"
      HTTP_CODE=$(curl -sL -w '%{http_code}' -o "$DMG_PATH" "${BASE_URL}/${DESKTOP_ARTIFACT}" 2>/dev/null || true)
      if [ "$HTTP_CODE" != "200" ]; then
        warn "Desktop app download failed (HTTP ${HTTP_CODE:-???}). Falling back to CLI-only."
        return 1
      fi

      info "Installing syncfu.app to /Applications..."
      MOUNT_DIR=$(mktemp -d "${TMPDIR:-/tmp}/syncfu-dmg.XXXXXXXXXX")

      hdiutil attach "$DMG_PATH" -nobrowse -mountpoint "$MOUNT_DIR" -quiet 2>/dev/null
      if [ -d "$MOUNT_DIR/syncfu.app" ]; then
        # Remove old version if present
        if [ -d "/Applications/syncfu.app" ]; then
          rm -rf "/Applications/syncfu.app"
        fi
        cp -R "$MOUNT_DIR/syncfu.app" "/Applications/syncfu.app"
        # Remove quarantine
        xattr -rd com.apple.quarantine "/Applications/syncfu.app" 2>/dev/null || true
        info "Installed syncfu.app to /Applications"
      else
        hdiutil detach "$MOUNT_DIR" -quiet 2>/dev/null || true
        rm -rf "$MOUNT_DIR"
        warn "Could not find syncfu.app in DMG. Falling back to CLI-only."
        return 1
      fi

      hdiutil detach "$MOUNT_DIR" -quiet 2>/dev/null || true
      rm -rf "$MOUNT_DIR"
      return 0
    }

    install_macos_app || CLI_ONLY=1

  elif [ "$OS_NAME" = "linux" ]; then
    install_linux_app() {
      info "Downloading syncfu AppImage..."
      APPIMAGE_PATH="${WORK_DIR}/${DESKTOP_ARTIFACT}"
      HTTP_CODE=$(curl -sL -w '%{http_code}' -o "$APPIMAGE_PATH" "${BASE_URL}/${DESKTOP_ARTIFACT}" 2>/dev/null || true)
      if [ "$HTTP_CODE" != "200" ]; then
        warn "Desktop app download failed (HTTP ${HTTP_CODE:-???}). Falling back to CLI-only."
        return 1
      fi

      APPIMAGE_DIR="$HOME/.local/share/syncfu"
      mkdir -p "$APPIMAGE_DIR"
      chmod +x "$APPIMAGE_PATH"
      mv "$APPIMAGE_PATH" "$APPIMAGE_DIR/syncfu.AppImage"
      info "Installed syncfu AppImage to $APPIMAGE_DIR"

      # Create desktop entry
      DESKTOP_DIR="$HOME/.local/share/applications"
      mkdir -p "$DESKTOP_DIR"
      cat > "$DESKTOP_DIR/syncfu.desktop" << 'DESKTOP_EOF'
[Desktop Entry]
Name=syncfu
Comment=Universal notification overlay
Exec=$HOME/.local/share/syncfu/syncfu.AppImage
Icon=syncfu
Type=Application
Categories=Utility;
StartupNotify=false
DESKTOP_EOF
      # Fix the Exec path (can't use variable inside heredoc with single quotes)
      sed -i "s|\$HOME|$HOME|g" "$DESKTOP_DIR/syncfu.desktop"
      info "Created desktop entry"
      return 0
    }

    install_linux_app || CLI_ONLY=1
  fi
fi

# =============================================
# 2. Install CLI binary
# =============================================
info "Downloading syncfu CLI..."
CLI_URL="${BASE_URL}/${CLI_ARTIFACT}"
CHECKSUM_URL="${BASE_URL}/checksums.txt"

HTTP_CODE=$(curl -sL -w '%{http_code}' -o "${WORK_DIR}/syncfu" "$CLI_URL" 2>/dev/null || true)
if [ "$HTTP_CODE" != "200" ]; then
  error "CLI download failed (HTTP ${HTTP_CODE:-???}). Check: https://github.com/${REPO}/releases/tag/v${VERSION}"
fi

# --- Verify checksum ---
if [ "$SKIP_CHECKSUM" = "1" ]; then
  warn "Checksum verification skipped (--skip-checksum)"
else
  info "Verifying checksum..."
  if ! curl -fsSL -o "${WORK_DIR}/checksums.txt" "$CHECKSUM_URL" 2>/dev/null; then
    error "Could not download checksums.txt — cannot verify integrity. Use --skip-checksum to bypass."
  fi
  EXPECTED=$(grep -F "${CLI_ARTIFACT}" "${WORK_DIR}/checksums.txt" | awk '{print $1}')
  if [ -z "$EXPECTED" ]; then
    error "Artifact '${CLI_ARTIFACT}' not found in checksums.txt. Use --skip-checksum to bypass."
  fi
  if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL=$(sha256sum "${WORK_DIR}/syncfu" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    ACTUAL=$(shasum -a 256 "${WORK_DIR}/syncfu" | awk '{print $1}')
  else
    error "No sha256sum or shasum found — cannot verify integrity. Use --skip-checksum to bypass."
  fi
  if [ "$EXPECTED" != "$ACTUAL" ]; then
    error "Checksum mismatch!\n  Expected: ${EXPECTED}\n  Got:      ${ACTUAL}"
  fi
  info "Checksum verified"
fi

# --- Install CLI ---
chmod +x "${WORK_DIR}/syncfu"
if [ "$OS_NAME" = "darwin" ]; then
  xattr -d com.apple.quarantine "${WORK_DIR}/syncfu" 2>/dev/null || true
fi
mkdir -p "$CLI_DIR"
mv "${WORK_DIR}/syncfu" "${CLI_DIR}/syncfu"
info "CLI installed to ${CLI_DIR}/syncfu"

# --- PATH instructions ---
case ":${PATH:-}:" in
  *":${CLI_DIR}:"*)
    ;;
  *)
    printf "\n"
    warn "Add syncfu to your PATH:"
    printf "\n"
    SHELL_NAME=$(basename "${SHELL:-/bin/sh}")
    case "$SHELL_NAME" in
      zsh)  printf "  echo 'export PATH=\"%s:\$PATH\"' >> ~/.zshrc && source ~/.zshrc\n" "$CLI_DIR" ;;
      bash) printf "  echo 'export PATH=\"%s:\$PATH\"' >> ~/.bashrc && source ~/.bashrc\n" "$CLI_DIR" ;;
      fish) printf "  fish_add_path %s\n" "$CLI_DIR" ;;
      *)    printf "  export PATH=\"%s:\$PATH\"\n" "$CLI_DIR" ;;
    esac
    printf "\n"
    ;;
esac

# =============================================
# 3. Start desktop app (tray only, no window)
# =============================================
if [ "$CLI_ONLY" = "0" ]; then
  if [ "$OS_NAME" = "darwin" ] && [ -d "/Applications/syncfu.app" ]; then
    info "Starting syncfu (tray + overlay)..."
    open -a "/Applications/syncfu.app"
    # Wait briefly for the server to come up
    for i in 1 2 3 4 5 6 7 8; do
      sleep 0.5
      if curl -s "http://127.0.0.1:9868/health" >/dev/null 2>&1; then
        break
      fi
    done

    if curl -s "http://127.0.0.1:9868/health" >/dev/null 2>&1; then
      info "syncfu is running — server listening on port 9868"
    else
      warn "syncfu app started but server not yet responding. It may take a moment."
    fi

  elif [ "$OS_NAME" = "linux" ]; then
    APPIMAGE="$HOME/.local/share/syncfu/syncfu.AppImage"
    if [ -x "$APPIMAGE" ]; then
      info "Starting syncfu (tray + overlay)..."
      nohup "$APPIMAGE" >/dev/null 2>&1 &
      sleep 2
      if curl -s "http://127.0.0.1:9868/health" >/dev/null 2>&1; then
        info "syncfu is running — server listening on port 9868"
      else
        warn "syncfu started but server not yet responding. It may take a moment."
      fi
    fi
  fi
fi

# --- Verify CLI ---
SYNCFU_BIN="${CLI_DIR}/syncfu"
if [ -x "$SYNCFU_BIN" ]; then
  INSTALLED_VERSION=$("$SYNCFU_BIN" --version 2>/dev/null || true)
  if [ -n "$INSTALLED_VERSION" ]; then
    info "$INSTALLED_VERSION"
  fi
fi

# --- Done ---
printf "\n"
if [ "$CLI_ONLY" = "0" ]; then
  printf "  ${BOLD}syncfu is installed and running!${RESET}\n"
  printf "\n"
  printf "  ${CYAN}Quick test:${RESET}\n"
  printf "    syncfu send \"Hello from syncfu!\"\n"
  printf "\n"
  printf "  The desktop app runs in the system tray. Notifications appear\n"
  printf "  as overlay panels — no focus stealing, works across all Spaces.\n"
else
  printf "  ${BOLD}syncfu CLI installed (headless mode).${RESET}\n"
  printf "\n"
  printf "  ${CYAN}Start the server:${RESET}\n"
  printf "    syncfu serve\n"
  printf "\n"
  printf "  ${CYAN}Send a notification:${RESET}\n"
  printf "    syncfu send \"Hello from syncfu!\"\n"
  printf "\n"
  printf "  For overlay notifications, install the desktop app:\n"
  printf "    curl -fsSL https://syncfu.dev/install.sh | sh\n"
fi
printf "\n"
info "Done! Run ${BOLD}syncfu --help${RESET} to get started."
printf "\n"
