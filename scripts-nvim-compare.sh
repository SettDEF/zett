#!/usr/bin/env bash
# Runs the same keys in zett-edit and headless Neovim (no config) and compares
# text and cursor. Cases: TEXT<TAB>KEYS per line on stdin, \n for line breaks.
set -u
cd "$(dirname "$0")"
cargo build -q -j 2 -p zett-edit --example keys || exit 1
Z=${CARGO_TARGET_DIR:-target}/debug/examples/keys
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
while IFS=$'\t' read -r text keys; do
  [ -z "$keys" ] && continue
  z=$("$Z" "$text" "$keys")
  printf '%b' "$text" > "$T/in.txt"; rm -f "$T/out"
  nvim -u NONE -i NONE -n --headless "$T/in.txt" \
      -c "silent! call feedkeys(\"${keys//</\\<}\", 'tx')" \
      -c "call writefile([join(getline(1, '\$'), '\\n') . '|' . line('.') . ':' . col('.')], '$T/out')" \
      -c 'qa!' >/dev/null 2>&1
  n=$(cat "$T/out" 2>/dev/null)
  if [ "$z" = "$n" ]; then pass=$((pass+1)); else fail=$((fail+1)); printf 'DIFF  %-16s %-14s zett=%s  nvim=%s\n' "$text" "$keys" "$z" "$n"; fi
done
echo "nvim-compare: $pass same, $fail different"
