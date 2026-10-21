#!/bin/sh
# gen-digraphs.sh — writes crates/zett-edit/src/digraph_table.rs from the Neovim installed here
# (digraph_getlist(true): RFC 1345 and Vim's own). NU is a real NUL in zett (Vim keeps NUL as NL).
# Usage: scripts/gen-digraphs.sh        (needs nvim and jq)
set -eu
cd "$(dirname "$0")/.."
command -v nvim >/dev/null && command -v jq >/dev/null || { echo "gen-digraphs: needs nvim and jq" >&2; exit 1; }
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT
nvim --headless --clean +"lua local f=io.open('$tmp','w'); f:write(vim.json.encode(vim.fn.digraph_getlist(true))); f:close()" +qa
out=crates/zett-edit/src/digraph_table.rs
ver=$(nvim --version | head -1 | sed 's/^NVIM v//; s/\.[0-9]*$//')
{
    echo "//! Vim's digraphs (RFC 1345 and Vim's own), taken from Neovim $ver's digraph_getlist(true): scripts/gen-digraphs.sh."
    echo
    echo "pub(crate) const TABLE: &[(&str, char)] = &["
    jq -r '.[] | "\(.[0]|tojson)\t\(.[1]|explode[0])"' "$tmp" | awk -F'\t' '{ if ($1 == "\"NU\"") $2 = 0; printf "    (%s, \x27\\u{%x}\x27),\n", $1, $2 }'
    echo "];"
} > "$out"
echo "gen-digraphs: $(grep -c "^    (" "$out") digraphs in $out"
