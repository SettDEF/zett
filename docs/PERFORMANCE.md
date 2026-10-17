# Performance — budgets, not hopes

Performance is a feature of zett, measured on every change. Each budget below
has a check in `cargo run --release -p zett-edit --example perf` (and later a CI
job); a change that breaks a budget is a bug, like a failing test.

Reference machine: ROG Flow Z13 (Ryzen AI Max+ 395, Radeon 8060S), on battery
in the Quiet profile. Budgets hold there, not on a desktop with a 4090.

## Budgets

### Feel (what a person notices)
| What | Budget |
|---|---|
| keypress → pixels on screen (editor, terminal) | < 8 ms (p99), < 4 ms typical |
| scrolling, cursor animation, canvas pan/zoom | display refresh rate (120 Hz), zero dropped frames |
| cold start → an open page | < 150 ms |
| open any page in a 100k-block workspace | < 16 ms (one frame) |
| quick open / search results while typing | < 16 ms per keystroke |
| completion popup in the terminal appears | < 30 ms after the keystroke |
| idle CPU with nothing changing | ~0 % (no timers, no polling, no animations running) |

### Engine (what the checks measure)
| What | Budget |
|---|---|
| one editing key in a 100k-line buffer | < 50 µs |
| `dd` / `p` / `u` in a 100k-line buffer | < 200 µs |
| create 100k blocks | < 500 ms |
| export / import a 100k-block workspace | < 300 ms each |
| Markdown mirror of a 10k-block page | < 20 ms |
| terminal: parse 100 MB of output (`cat bigfile`) | < 1 s, UI stays responsive |
| memory: 100k-block workspace | < 300 MB |

## Rules that keep it fast

- **No work when nothing changes.** No polling timers, no endless animations
  (the shell taught this: one spinning icon kept a GPU busy all day). Animations
  stop when they finish; idle means idle.
- **Do less, not faster.** Render only what is on screen (virtualised lists,
  glyph atlases, damage tracking); update indexes incrementally, never rebuild.
- **Never block the UI thread.** File I/O, parsing, search, sync, scripts and
  plugins run off the frame; the UI shows what is ready.
- **Measure first, then optimise.** No speculative cleverness; every
  optimisation comes with the number it moved.
- **Snapshots are not undo.** M0's undo copies the whole text per change,
  which is fine for small buffers and breaks the 200 µs budget on huge ones;
  M1 replaces it with operation-based undo through the CRDT. The perf check
  will say so until then.
- **Plugins are budgeted too.** Lua and WASM hooks get a time slice; a slow
  one is reported by `zett doctor` with its name and its cost.
