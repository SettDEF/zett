#!/usr/bin/env bash
# test.sh: every zett test in under 20 s: the crates' unit tests (no doc-tests: zett has none), then the
# window's checks in parallel (gui-check.sh). Exit 1 on any failure.
#   scripts/test.sh            build (release) and run both
#   scripts/test.sh --unit     the unit tests only
set -uo pipefail
cd "$(dirname "$0")/.."
start=$(date +%s%N)
cargo test --release -p zett-tui -p zett-edit -p zett-script -p zett-ycm -p zett-core --lib 2>&1 | grep -E "^test result|FAILED|panicked|^error" | grep -v "0 passed; 0 failed" || true
unit=${PIPESTATUS[0]}
[ "$unit" -eq 0 ] || { echo "unit tests failed"; exit 1; }
if [ "${1:-}" != "--unit" ]; then
    cargo build --release -p zett 2>&1 | grep -E "^error" && exit 1
    bash scripts/gui-check.sh | grep -E "FAIL|passed|kept"
    gui=${PIPESTATUS[0]}
fi
echo "all tests in $(( ($(date +%s%N) - start) / 1000000 )) ms"
exit "${gui:-0}"
