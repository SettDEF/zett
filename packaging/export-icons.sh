#!/usr/bin/env bash
# export-icons.sh: the SVGs from their VectorCraft sources (zett.vectorcraft, filetypes/*.vectorcraft), text as outlines
set -euo pipefail
cd "$(dirname "$0")"
for v in zett.vectorcraft filetypes/*.vectorcraft; do vectorcraft-cli convert "$v" "${v%.vectorcraft}.svg" --outline-text >/dev/null; done
echo "exported $(ls filetypes/*.svg | wc -l) file-type icons and zett.svg"
