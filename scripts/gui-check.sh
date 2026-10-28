#!/usr/bin/env bash
# gui-check.sh: every zett --gui test hook run headless on a throwaway workspace; each must draw a picture
# without a QML error, and the ones that change files must leave the right result. Exit 1 on any failure.
#   scripts/gui-check.sh [ZETT_BINARY]      (default: $CARGO_TARGET_DIR/release/zett)
# Uses clangd for the language server checks when it is installed (skipped otherwise).
set -uo pipefail
BIN=${1:-${CARGO_TARGET_DIR:-$(dirname "$0")/../target}/release/zett}
W=$(mktemp -d /tmp/zett-gui-check.XXXXXX)
OUT=$W/.out
mkdir -p "$OUT"
trap 'rm -rf "$W"' EXIT
pass=0 fail=0

# the workspace: a page with code, a Rust file, a C file with two errors, a template, a git repository
printf '# Check\n\nSome text worth selecting here.\n\n```lua\nfor i = 1, 3 do print("hi " .. i) end\n```\n\nEnd.\n' > "$W/check.md"
{ echo 'fn main() {'; for i in $(seq 1 50); do echo "    println!(\"line $i\");   "; done; echo '}'; } > "$W/main.rs"
printf 'int main(void) {\n    int x = "text";\n    undefined_fn();\n    return x;\n}\n' > "$W/bad.c"
printf 'int add(int a, int b);\nint main(void) {\n    add(1, 2);\n    return 0;\n}\n' > "$W/sig.c"
printf 'int main(void) {\n    int x = 1\n    return x;\n}\n' > "$W/fix.c"
printf 'a\nb\nc\n' > "$W/lines.rs"
printf 'fn main() {\n    let a = 1;\n    let b = 2;\n}\n' > "$W/core.rs"
cp "$W/core.rs" "$W/core2.rs"; cp "$W/core.rs" "$W/core3.rs"; cp "$W/core.rs" "$W/core4.rs"; cp "$W/core.rs" "$W/core5.rs"
for i in $(seq 1 1000); do echo "impl T$i {"; for j in $(seq 1 98); do echo "    fn f$j(&self) -> u32 { $j }"; done; echo "}"; done > "$W/big.rs"
cp "$W/fix.c" "$W/fix2.c"; cp "$W/bad.c" "$W/bad2.c"   # untouched copies for the new core's server checks
mkdir -p "$W/templates"; printf '# Meeting\n' > "$W/templates/Meeting.md"
git -C "$W" init -q && git -C "$W" add -A && git -C "$W" -c user.email=c@c -c user.name=c commit -qm init
echo '// changed' >> "$W/main.rs"

# NAME ENGINE [CHECK]: run one hook; CHECK is a shell test run afterwards
check() {
    local name=$1 engine=$2 test=${3:-true}
    local png=$OUT/$name.png log=$OUT/$name.log
    ZETT_DIR=$W QT_QPA_PLATFORM=offscreen QT_FORCE_STDERR_LOGGING=1 ZETT_SHOT=$png ZETT_SHOT_ENGINE=$engine timeout 40 nice -n 19 "$BIN" --gui check.md > "$log" 2>&1
    local code=$?
    local why=""
    [ $code -eq 0 ] || why="exit $code"
    [ -s "$png" ] || why="${why:+$why, }no picture"
    grep -qE "TypeError|ReferenceError|is not a function|Cannot read property" "$log" && why="${why:+$why, }$(grep -m1 -oE "(TypeError|ReferenceError)[^\n]{0,80}" "$log")"
    eval "$test" || why="${why:+$why, }check failed: $test"
    if [ -z "$why" ]; then pass=$((pass + 1)); printf '  ok    %s\n' "$name"; else fail=$((fail + 1)); printf '  FAIL  %s: %s\n' "$name" "$why"; fi
}

echo "zett --gui check ($BIN)"
check page ""
check home today
check settings settings
check shortcuts shortcuts
check select select
check run run "grep -q 'hi 3' '$W/check.md'"
check run-again run "[ \$(grep -c '^\`\`\`output' '$W/check.md') -eq 1 ]"
check file file:main.rs
check term term
check find filefind:main.rs:line
check filesearch filesearch:println
check replace filereplace:println:print_line "grep -q 'print_line' '$W/main.rs' && ! grep -q 'println' '$W/main.rs'"
check edit fileedit:main.rs "head -1 '$W/main.rs' | grep -q 'edited in zett' && ! grep -qE '[[:blank:]]$' '$W/main.rs'"
check side fileside:main.rs:check.md
check git git
check core core:core.rs "grep -q 'core rows 3 lines 5 fold 2' \"\$OUT/core.log\" && grep -q '    ylet a' '$W/core.rs' && grep -q '    ylet b' '$W/core.rs' && ! grep -q xlet '$W/core.rs'"
# speed: a 100,000 line file opens in under 300 ms, five keys at its end in under 200 ms
check corebig corebig:big.rs "awk '/corebig open/ { o = \$5; k = \$(NF-1) } END { exit !(o < 300 && k < 200) }' \"\$OUT/corebig.log\""
check core2 core2:core2.rs "grep -q 'core2 found 2 replaced 2' \"\$OUT/core2.log\" && [ \"\$(sed -n 3p '$W/core2.rs')\" = '    // var a = 1;' ]"
check core3 core3:core3.rs "grep -q 'core3 word \\[a\\] indented \\[        let a = 1;\\] cursors 2' \"\$OUT/core3.log\" && grep -q '^    xlet a' '$W/core3.rs' && grep -q '^    xlet b' '$W/core3.rs'"
check coresave coresave:core4.rs "! grep -qE '[[:blank:]]$' '$W/core4.rs' && [ \"\$(tail -c1 '$W/core4.rs' | od -An -c | tr -d ' ')\" = '\\n' ] && tail -1 '$W/core4.rs' | grep -q tail"
check corewords corewords:core5.rs "grep -q 'corewords \\[\"let\"\\] \\[\"main\"\\]' \"\$OUT/corewords.log\""
check corekeep corekeep:core5.rs,lines.rs "grep -q 'corekeep away 1 back \\[kept fn main' \"\$OUT/corekeep.log\" && grep -q 'dirty true' \"\$OUT/corekeep.log\""
check corenote corenote:core5.rs "grep -q 'corenote pages \\[\"core5.rs › main\"\\] notes \\[\"core5.rs › main\"\\] side [0-9]' \"\$OUT/corenote.log\""
check welcome welcome
check lines filelines:lines.rs "[ \"\$(tr '\\n' '|' < '$W/lines.rs')\" = 'b|// a|// a|c|' ]"
check tooltip acttip
check newfile newfile "[ -f '$W/made/new.rs' ] && [ -d '$W/emptydir' ]"
check wrap filewrap:main.rs
check sticky filesticky:main.rs "grep -q 'sticky \\[\"fn main' \"\$OUT/sticky.log\""
check multicursor filemulti:main.rs "[ \$(grep -cE '^// (fn|    print)' '$W/main.rs') -eq 3 ]"
check panel problems-panel "grep -q 'problems \\[\"src/main.rs:2' \"\$OUT/panel.log\""
check diff filediff:main.rs "grep -q 'zett-probe: diff [1-9]' \"\$OUT/diff.log\""
check tasks tasks "grep -q 'zett-probe: palette' \"\$OUT/tasks.log\""
check quick quick:mnrs "grep -q 'quick \[\"file:main.rs' "$OUT/quick.log""
if command -v clangd >/dev/null; then check symbols filelsp:bad.c:symbols "grep -q 'jumps .*main' \"\$OUT/symbols.log\""; check rename filelsp:bad.c:rename:1:8:count "grep -q 'int count' '$W/bad.c' && grep -q 'return count' '$W/bad.c'"; fi
if command -v clangd >/dev/null; then check signature filelsp:sig.c:sig:2:12 "grep -q 'signature add(int a, int b).*\\[int b\\]' \"\$OUT/signature.log\""; check actions filelsp:fix.c:act:2:4 "grep -q 'int x = 1;' '$W/fix.c'"; fi
if command -v clangd >/dev/null; then check coreaction corelsp:fix2.c:act:2:4 "grep -q 'int x = 1;' '$W/fix2.c'"; check corerename corelsp:bad2.c:rename:1:8:count "grep -q 'int count' '$W/bad2.c' && grep -q 'return count' '$W/bad2.c'"; fi
if command -v clangd >/dev/null; then check problems fileproblems:bad.c; else echo "  skip  problems (no clangd)"; fi
echo "$pass passed, $fail failed  (pictures: $OUT, kept only on failure)"
if [ $fail -gt 0 ]; then trap - EXIT; echo "workspace kept: $W"; exit 1; fi
