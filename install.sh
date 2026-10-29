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
# the Explorer's file-type icons (drawn in VectorCraft: packaging/filetypes/*.vectorcraft, export-icons.sh)
install -d "$HOME/.local/share/zett/icons/filetypes" && install -m644 packaging/filetypes/*.svg "$HOME/.local/share/zett/icons/filetypes/"
# plugins that ship with zett (plugins/*: Lua, Neovim layout), as start packages
for p in plugins/*/; do n=$(basename "$p"); d="$HOME/.local/share/zett/site/pack/zett/start/$n"; rm -rf "$d"; /usr/bin/install -d "$d"; command cp -r "$p". "$d"/; done
# the store's WASM plugins (wasm/*: their zett-plugin.json and .wasm), copied to a config by :ZettStore
for p in wasm/*/; do n=$(basename "$p"); [ -f "$p/zett-plugin.json" ] && install -d "$HOME/.local/share/zett/store/$n" && install -m644 "$p"/zett-plugin.json "$p"/*.wasm "$HOME/.local/share/zett/store/$n/"; done
if command -v rsvg-convert >/dev/null; then
    for s in 16 24 32 48 64 128 256; do
        install -dm755 "$icons/${s}x${s}/apps"
        rsvg-convert -w $s -h $s packaging/zett.svg -o "$icons/${s}x${s}/apps/zett.png"
    done
fi
sed "s|^Exec=zett|Exec=$HOME/.local/bin/zett|" packaging/zett.desktop > "$HOME/.local/share/applications/zett.desktop"
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
say "installed: 'zett' (the editor, in the terminal), 'zett --gui' (the window), 'zett --help'"
