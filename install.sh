#!/usr/bin/env bash
# Builds zett and installs it for this user: ~/.local/bin/zett, the launcher
# entry and the icon. The workspace is ~/zett (or $ZETT_DIR). Re-run to update.
set -euo pipefail
cd "$(dirname "$0")"
say() { printf '\033[1;34m::\033[0m %s\n' "$*"; }
say "building (release, low priority)"
nice -n 19 cargo build --release --quiet -p zett -j "${JOBS:-4}"
install -Dm755 "${CARGO_TARGET_DIR:-target}/release/zett" "$HOME/.local/bin/zett"
icons="$HOME/.local/share/icons/hicolor"
install -Dm644 packaging/zett.svg "$icons/scalable/apps/zett.svg"
if command -v rsvg-convert >/dev/null; then
    for s in 16 24 32 48 64 128 256; do
        install -dm755 "$icons/${s}x${s}/apps"
        rsvg-convert -w $s -h $s packaging/zett.svg -o "$icons/${s}x${s}/apps/zett.png"
    done
fi
sed "s|^Exec=zett|Exec=$HOME/.local/bin/zett|" packaging/zett.desktop > "$HOME/.local/share/applications/zett.desktop"
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
say "installed: run 'zett' (window) or 'zett --help' (command line)"
