#!/usr/bin/env bash
# export-icons.sh: the file-type SVGs from their VectorCraft sources (filetypes/*.vectorcraft), text as outlines
set -euo pipefail
cd "$(dirname "$0")"
for v in filetypes/*.vectorcraft; do vectorcraft-cli convert "$v" "${v%.vectorcraft}.svg" --outline-text >/dev/null; done
echo "exported $(ls filetypes/*.svg | wc -l) file-type icons (the logo: render-logo.sh)"
