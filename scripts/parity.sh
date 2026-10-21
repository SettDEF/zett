#!/usr/bin/env bash
# parity.sh: the complete lists of what Neovim and Emacs offer, from the installed programs
# (clean, no user config), with what zett already has marked: docs/parity/*.md.
#   scripts/parity.sh            regenerate (needs nvim, emacs, and zett on PATH or ~/.local/bin)
# Re-run as zett grows: the marks come from zett itself (its commands, :ex names, settings).
set -euo pipefail
cd "$(dirname "$0")/.."
out=docs/parity
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$out"
ZETT=${ZETT:-$(command -v zett || echo "$HOME/.local/bin/zett")}
EMACS=${EMACS:-/usr/bin/emacs}
RT=$(nvim --clean --headless -c 'lua io.write(vim.env.VIMRUNTIME)' -c q 2>&1)

# ── what Neovim and Emacs have ──
nvim --clean --headless -c "lua local o=vim.api.nvim_get_all_options_info(); local t={}; for n,i in pairs(o) do t[#t+1]=(string.format('%s\t%s\t%s\t%s\t%s', n, (i.shortname ~= '' and i.shortname) or '-', i.type, i.scope, (tostring(i.default) ~= '' and tostring(i.default)) or '""'):gsub('[\n\t]+$',''):gsub('\n',' ')) end; table.sort(t); vim.fn.writefile(t, '$tmp/opts.tsv')" -c q
nvim --clean --headless -c "lua local t={} for k in pairs(vim.api) do t[#t+1]=k end table.sort(t) vim.fn.writefile(t,'$tmp/api.txt')" -c q
nvim --clean --headless -c "lua local t={} for _,n in ipairs(vim.fn.getcompletion('', 'function')) do if not n:find('#') then t[#t+1]=(n:gsub('%(%)?$','')) end end table.sort(t) vim.fn.writefile(t,'$tmp/fn.txt')" -c q
awk '/\*ex-cmd-index\*/{f=1} f && /^\|:/' "$RT/doc/index.txt" | sed 's/^|\([^|]*\)|\t*\([^\t]*\)\t*\(.*\)/\1\t\2\t\3/' > "$tmp/ex.tsv"
awk '/\*normal-index\*/{f=1} /\*visual-index\*/{f=0} f && /^\|[^|]*\|/' "$RT/doc/index.txt" | sed 's/^|\([^|]*\)|\t*\([^\t]*\)\t*\(.*\)/\1\t\2\t\3/' | awk -F'\t' '{d=$3; for (i=4; i<=NF; i++) d=d" "$i; print $1"\t"$2"\t"d}' > "$tmp/normal.tsv"
awk "/\*option-list\*/{f=1} f && /^'[a-z]/" "$RT/doc/quickref.txt" | sed "s/^'\([a-z]*\)'[ \t]*\('[a-z]*'\)\{0,1\}[ \t]*\(.*\)/\1\t\3/" > "$tmp/optdesc.tsv"
"$EMACS" -Q --batch --eval "(let (out) (mapatoms (lambda (s) (when (commandp s) (push (format \"%s\t%s\" s (let ((d (ignore-errors (documentation s t)))) (if d (car (split-string d \"\n\")) \"\"))) out)))) (with-temp-file \"$tmp/emacs-cmds.tsv\" (insert (mapconcat (function identity) (sort out (function string<)) \"\n\"))))"
"$EMACS" -Q --batch --eval "(let (out) (mapatoms (lambda (s) (when (commandp s) (let ((k (where-is-internal s nil t))) (when k (push (format \"%s\t%s\" (key-description k) s) out)))))) (with-temp-file \"$tmp/emacs-keys.tsv\" (insert (mapconcat (function identity) (sort out (function string<)) \"\n\"))))"

# ── what zett has (from zett itself and its sources) ──
"$ZETT" doc commands 2>/dev/null | grep -o '^- `[^`]*`' | sed 's/^- `//; s/`$//' | sort -u > "$tmp/zett-cmds.txt"
grep -oh '^\s*"[a-z!&<>=]*"\( | "[a-z!&<>=]*"\)* =>' crates/zett-tui/src/ex.rs crates/zett-tui/src/lib.rs crates/zett-tui/src/commands.rs | grep -o '"[^"]*"' | tr -d '"' | sort -u > "$tmp/zett-ex.txt"
# the quickfix and location commands are a table (qfcmd.rs NAMES): full names and shortest forms
grep -o '("[A-Za-z]*", "[A-Za-z]*")' crates/zett-tui/src/qfcmd.rs | grep -o '"[A-Za-z]*"' | tr -d '"' >> "$tmp/zett-ex.txt"
"$ZETT" options --working > "$tmp/zett-opts.txt"
grep -o 'name: "[a-z]*"' crates/zett-tui/src/nvim_options.rs | cut -d'"' -f2 > "$tmp/zett-kept.txt"
has() { grep -qxF -- "$1" "$2"; }
stamp="Generated $(date +%Y-%m-%d) by scripts/parity.sh from $(nvim --version | head -1), $("$EMACS" --version | head -1) and zett $("$ZETT" --version 2>/dev/null | awk '{print $2}')."

# ── Neovim's options ──
{
    n=$(wc -l < "$tmp/opts.tsv"); have=0; kept=0
    while IFS=$'\t' read -r name short _ _ _; do
        if { has "$name" "$tmp/zett-opts.txt" || has "$short" "$tmp/zett-opts.txt"; }; then have=$((have + 1)); elif has "$name" "$tmp/zett-kept.txt"; then kept=$((kept + 1)); fi
    done < "$tmp/opts.tsv"
    echo "# Neovim's options in zett"; echo; echo "$stamp"; echo
    echo "$have ✓ do something in zett, $kept ◐ are kept by :set but do nothing yet, of $n (\`:set\`, \`zett.opt\`, \`zett.ft.<type>\`; \`:help settings\`)."; echo
    echo "| | option | short | type | scope | default | what it does |"; echo "|---|---|---|---|---|---|---|"
    while IFS=$'\t' read -r name short type scope def; do
        mark="·"; if { has "$name" "$tmp/zett-opts.txt" || has "$short" "$tmp/zett-opts.txt"; }; then mark="✓"; elif has "$name" "$tmp/zett-kept.txt"; then mark="◐"; fi
        desc=$(awk -F'\t' -v n="$name" '$1==n{print $2; exit}' "$tmp/optdesc.tsv")
        [ "$short" = - ] && short=""
        printf '| %s | `%s` | %s | %s | %s | `%s` | %s |\n' "$mark" "$name" "$short" "$type" "$scope" "${def//|/\\|}" "${desc//|/\\|}"
    done < "$tmp/opts.tsv"
} > "$out/nvim-options.md"

# ── Neovim's Ex commands ──
{
    echo "# Neovim's Ex commands in zett"; echo; echo "$stamp"; echo
    echo "✓ zett's \`:\` line has it (by its full or short name), · not yet."; echo
    echo "| | command | written | what it does |"; echo "|---|---|---|---|"
    while IFS=$'\t' read -r tag written desc; do
        full=${written#:}; full=${full//[\[\]]/}; short=${written#:}; short=${short%%\[*}
        mark="·"; { has "$full" "$tmp/zett-ex.txt" || has "$short" "$tmp/zett-ex.txt"; } && mark="✓"
        printf '| %s | `%s` | `%s` | %s |\n' "$mark" "${tag//|/\\|}" "${written//|/\\|}" "${desc//|/\\|}"
    done < "$tmp/ex.tsv"
} > "$out/nvim-ex-commands.md"

# ── Neovim's Normal-mode keys: each one fed into zett's Vim profile (zett probe-keys), or bound in its key map ──
# its own data folder: zg / zw in the probe must not touch the real word lists
cut -f1,2 "$tmp/normal.tsv" | XDG_DATA_HOME="$tmp" "$ZETT" probe-keys > "$tmp/probe.tsv"
"$ZETT" keys n 2>/dev/null | awk '{print $1}' > "$tmp/zett-nkeys.txt"
{
    echo "# Neovim's Normal-mode keys in zett"; echo; echo "$stamp"; echo
    echo "Each key is written out (\`d{motion}\` as \`dw\`) and fed into zett's Vim profile with text, a yank, a change and"
    echo "a search to act on: ✓ something happened (or zett's key map binds it), · nothing yet."; echo
    echo "| | key | fed as | what it does |"; echo "|---|---|---|---|"
    paste "$tmp/normal.tsv" "$tmp/probe.tsv" | tr '\t' '\037' | while IFS=$'\037' read -r tag written desc ok keys; do
        mark="·"; { [ "$ok" = 1 ] || has "$keys" "$tmp/zett-nkeys.txt"; } && mark="✓"
        printf '| %s | `%s` | `%s` | %s |\n' "$mark" "${written//|/\\|}" "${keys//|/\\|}" "$(echo "${desc//|/\\|}" | sed 's/^[0-9]  *//')"
    done
} > "$out/nvim-normal-keys.md"

# the same keys as a help page (:help vim-keys, :help gJ): the key, what it does, whether zett has it
{
    echo "# Every normal-mode key"; echo
    echo "Neovim's normal-mode index, key by key, with what each does. \`:help KEY\` (\`:help gJ\`,"
    echo "\`:help CTRL-W_x\`) opens this page at it; \`:help vim\` explains them by area. Generated by"
    echo "scripts/parity.sh; a key zett does not have yet says so."; echo
    awk -F' \\| ' '/^\| [✓·] \| `/ { k=$2; gsub(/^`|`$/, "", k); gsub(/\\\|/, "|", k); d=$4; sub(/ \|$/, "", d); gsub(/\\\|/, "|", d); gsub(/^ +| +$/, "", d); printf "- `%s`  %s%s\n", k, d, ($1 ~ /·/ ? " (not in zett yet)" : "") }' "$out/nvim-normal-keys.md"
} > crates/zett-script/docs/vim-keys.md

# ── Neovim's Lua API and functions ──
{
    echo "# Neovim's API (vim.api) and functions (vim.fn)"; echo; echo "$stamp"; echo
    echo "What a plugin can call. zett's own plugin API is \`zett.*\` (\`:help config\`, \`:help plugins\`); this is what"
    echo "it is measured against."; echo
    echo "✓ zett's \`vim\` compatibility layer has it (Neovim plugins call it unchanged), · not yet."; echo
    "$ZETT" lua 'for k in pairs((vim or {}).api or {}) do print(k) end; for k in pairs((vim or {}).fn or {}) do print("fn." .. k) end' 2>/dev/null | sort -u > "$tmp/zett-vim.txt" || true
    echo "## vim.api ($(wc -l < "$tmp/api.txt"))"; echo
    while read -r n; do printf -- '- %s `%s`\n' "$(has "$n" "$tmp/zett-vim.txt" && echo ✓ || echo ·)" "$n"; done < "$tmp/api.txt"; echo
    echo "## vim.fn ($(wc -l < "$tmp/fn.txt"))"; echo
    while read -r n; do printf -- '- %s `%s`\n' "$(has "fn.$n" "$tmp/zett-vim.txt" && echo ✓ || echo ·)" "$n"; done < "$tmp/fn.txt"
} > "$out/nvim-api.md"

# ── Emacs's keys and commands ──
{
    n=$(wc -l < "$tmp/emacs-keys.tsv"); have=0
    while IFS=$'\t' read -r k cmd; do has "$cmd" "$tmp/zett-cmds.txt" && have=$((have + 1)); done < "$tmp/emacs-keys.tsv"
    echo "# Emacs's default keys in zett"; echo; echo "$stamp"; echo
    echo "$have of $n bound commands. ✓ zett has the command under the same name (\`M-x\`), · not yet."; echo
    echo "| | key | command |"; echo "|---|---|---|"
    while IFS=$'\t' read -r k cmd; do mark="·"; has "$cmd" "$tmp/zett-cmds.txt" && mark="✓"; printf '| %s | `%s` | `%s` |\n' "$mark" "${k//|/\\|}" "$cmd"; done < "$tmp/emacs-keys.tsv"
} > "$out/emacs-keys.md"
{
    n=$(wc -l < "$tmp/emacs-cmds.tsv")
    echo "# Emacs's commands"; echo; echo "$stamp"; echo
    echo "All $n interactive commands of a plain Emacs (\`emacs -Q\`), by their package's prefix. ✓ zett has it under"
    echo "the same name. Most are packages (mail, games, modes for other languages): zett's answer to many is a"
    echo "plugin, not the core."; echo
    awk -F'\t' '{p=$1; sub(/-.*/, "", p); print p "\t" $0}' "$tmp/emacs-cmds.tsv" | sort -t$'\t' -k1,1 -k2,2 | awk -F'\t' -v z="$tmp/zett-cmds.txt" '
        BEGIN { while ((getline l < z) > 0) have[l] = 1 }
        $1 != last { printf "\n## %s\n\n", $1; last = $1 }
        { d = $3; gsub(/\|/, "\\|", d); printf "- %s `%s` %s\n", ($2 in have) ? "✓" : "·", $2, d }'
} > "$out/emacs-commands.md"

# ── the index ──
{
    echo "# Parity: Neovim and Emacs, and what zett has"; echo; echo "$stamp"; echo
    for f in nvim-options nvim-ex-commands nvim-normal-keys nvim-api emacs-keys emacs-commands; do
        t=$(head -1 "$out/$f.md" | sed 's/^# //'); c=$(grep -c '| ✓ \|^- ✓' "$out/$f.md" || true); k=$(grep -c '^| ◐ ' "$out/$f.md" || true); a=$(grep -c '^| [✓·◐] \|^- [✓·] \|^| `' "$out/$f.md" || true)
        if [ "$k" -gt 0 ]; then echo "- [$t]($f.md): $c ✓ of $a, $k more ◐ kept but doing nothing yet"; else echo "- [$t]($f.md): $c ✓ of $a"; fi
    done
} > "$out/README.md"
cat "$out/README.md"
