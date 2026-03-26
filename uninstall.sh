#!/bin/sh
set -eu

APP_NAME="syncfu"

# --- Colors (only if terminal) ---
if [ -t 1 ]; then
  BOLD='\033[1m' GREEN='\033[32m' RED='\033[31m' YELLOW='\033[33m' RESET='\033[0m'
else
  BOLD='' GREEN='' RED='' YELLOW='' RESET=''
fi

info()  { printf "${GREEN}info${RESET}  %s\n" "$@"; }
warn()  { printf "${YELLOW}warn${RESET}  %s\n" "$@"; }

printf "\n"
printf "  ${BOLD}syncfu uninstaller${RESET}\n"
printf "\n"

REMOVED=0

# --- Stop running processes ---
if pgrep -f "syncfu" >/dev/null 2>&1; then
  info "Stopping syncfu processes..."
  pkill -f "syncfu.app" 2>/dev/null || true
  pkill -f "syncfu.AppImage" 2>/dev/null || true
  sleep 1
fi

# --- Remove desktop app ---
OS=$(uname -s)
case "$OS" in
  Darwin)
    if [ -d "/Applications/syncfu.app" ]; then
      rm -rf "/Applications/syncfu.app"
      info "Removed /Applications/syncfu.app"
      REMOVED=$((REMOVED + 1))
    fi
    ;;
  Linux)
    APPIMAGE="$HOME/.local/share/syncfu/syncfu.AppImage"
    if [ -f "$APPIMAGE" ]; then
      rm -f "$APPIMAGE"
      rmdir "$HOME/.local/share/syncfu" 2>/dev/null || true
      info "Removed AppImage"
      REMOVED=$((REMOVED + 1))
    fi
    DESKTOP_FILE="$HOME/.local/share/applications/syncfu.desktop"
    if [ -f "$DESKTOP_FILE" ]; then
      rm -f "$DESKTOP_FILE"
      info "Removed desktop entry"
    fi
    ;;
esac

# --- Remove CLI binary ---
for DIR in /usr/local/bin "$HOME/.syncfu/bin" "$HOME/.cargo/bin"; do
  if [ -f "${DIR}/syncfu" ]; then
    rm -f "${DIR}/syncfu"
    info "Removed ${DIR}/syncfu"
    REMOVED=$((REMOVED + 1))
  fi
done

# --- Clean up ~/.syncfu directory ---
if [ -d "$HOME/.syncfu" ]; then
  rm -rf "$HOME/.syncfu"
  info "Removed ~/.syncfu"
fi

# --- Remove PATH entries from shell configs ---
for RC_FILE in "$HOME/.zshrc" "$HOME/.bashrc"; do
  if [ -f "$RC_FILE" ] && grep -q '.syncfu/bin' "$RC_FILE" 2>/dev/null; then
    # Create backup then remove the line
    cp "$RC_FILE" "${RC_FILE}.bak"
    grep -v '.syncfu/bin' "$RC_FILE" > "${RC_FILE}.tmp" && mv "${RC_FILE}.tmp" "$RC_FILE"
    info "Removed PATH entry from $(basename "$RC_FILE")"
  fi
done

# --- Done ---
printf "\n"
if [ "$REMOVED" -gt 0 ]; then
  info "syncfu has been uninstalled."
else
  warn "No syncfu installation found."
fi
printf "\n"
