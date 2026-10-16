# The zett terminal — plan

A terminal emulator **written entirely in Rust**, part of zett and usable on
its own. As smooth as Neovide, more useful than Warp, as customizable as Emacs,
and replacing WezTerm + tmux on this desktop.

Name: **zash** (zett + shell), chosen 2026-10-16. Clear on the US register; EU/DPMA still to check by hand before anything permanent.

## What it must be

1. **Buttery.** GPU-rendered at the display's refresh rate: pixel-smooth
   scrolling with momentum, an animated cursor (Neovide-style trail, any
   curve), smooth resize and font zoom, no tearing, no flicker on full-screen
   redraws. Input to photons under 8 ms.
2. **Interactive, not just a grid.** The terminal understands the shell: it
   knows where each prompt, command and output starts, what you are typing,
   and where your cursor is in it. That is what makes popups, blocks and
   rerunning possible.
3. **Customizable to the bone.** Every key, colour, animation, popup, layout,
   font, padding, status segment and behaviour is a setting or a Lua function.
   Nothing is hard-coded that a person might want different. Defaults are
   beautiful (the Doom rule).
4. **Part of zett.** Commands are blocks, Vim keys work in the scrollback,
   terminals live in pages, agents can drive a pane you watch.

## Headline features

### Completion popups for zsh (and bash, fish)
While you type, a **real GUI popup** under the cursor (not text drawn into the
grid):

- candidates from **zsh's own completion system** (every `_git`, `_docker`,
  `_cargo` completer works), captured through a small zsh plugin, plus history
  matches (like zsh-autosuggestions) and, optionally, an agent's suggestion
- each row: icon, the candidate, its description (`--force   overwrite
  existing files`), its group (`commands`, `files`, `branches`)
- a **preview pane**: file contents or image, directory listing, `git log` of a
  branch, the man-page paragraph for a flag, a process's details
- fuzzy filter as you type, `Tab`/`Enter` to accept, Vim keys to move, `Esc`
  to close; multi-select for files
- the ghost-text suggestion of the rest of the line, accepted with `→`
- all of it styled and laid out in Lua: position (below cursor, above,
  side panel), size, columns shown, icons, fonts, animation

### Command blocks
Every command plus its output is a **block** (via shell integration, OSC 133):

- jump between prompts (`[[ ]]`), select a whole output, copy it, collapse it
- sticky header: the running command stays visible while its output scrolls
- exit status, duration and cwd on each block; failed ones marked
- rerun, edit-and-rerun, **diff two runs**, search all past output
- send a block to a zett page (or link it), share it as a snippet

### Vim everywhere
The zett-edit engine drives the scrollback (`v`, `y`, `/`, `n`, text objects
over output) **and the command line itself**: edit the line you are typing
with real Vim grammar, with the same keymaps as your editor.

### Rich output
Images (Kitty graphics, Sixel, iTerm2), clickable links and file paths
(`file:line` opens in zett), JSON/CSV output as a collapsible tree or table on
demand, Markdown rendered, progress bars recognised. Custom **output
renderers** in Lua: "when a command prints X, show it as Y".

### Panes, tabs, sessions (tmux's job, built in)
Splits and tabs with the `SPC` leader; sessions run in a background server, so
closing the window does not kill them; attach from any window, the TUI, or
over SSH. Layouts saved and restored per project.

### Triggers and hooks (Lua)
- triggers: a regex on output runs an action (`error: .* at (file):(line)` →
  make it clickable; `password:` → offer the keyring)
- hooks: `command.start`, `command.end` (notify when a long command finishes
  while you are elsewhere), `cwd.changed`, `prompt`, `output`
- everything the terminal does is a zett command, so keys, menus and scripts
  can do it too

### Part of the desktop
tinct theme live (changes with the wallpaper), ii-decor's buttons as an
overlay, shell widgets (running commands, last failure), a Dotos task from a
failed command in one key, agents that run commands in a pane with approval.

## Architecture (all Rust)

```
crates/
  zash-vt        VT/xterm state machine and grid: vte parser, scrollback
                 (compressed), reflow on resize, Kitty keyboard + graphics,
                 OSC 7 / 8 / 52 / 133, sixel; no I/O, fully tested
  zash-pty       ptys and processes (rustix), the session server, attach/detach
  zash-shell     shell integration: zsh/bash/fish plugins speak to the terminal
                 over OSC + a socket (prompt marks, buffer, cursor, completion
                 candidates with descriptions)
  zash-render    wgpu renderer: glyph atlas, ligatures (harfbuzz via rustybuzz),
                 font fallback, emoji, undercurl, smooth scroll, cursor
                 animation, popups and blocks drawn as GUI layers
  zash-ui        popups, block chrome, panes/tabs layout, all Lua-styleable
  zash           the standalone app (winit + wgpu)
zett-app         embeds the same renderer as a pane inside zett's window
```

The grid and the renderer are separate so the same terminal state draws in the
standalone window, inside zett, and (simplified) in the TUI.

## Milestones

- **T0 — a terminal.** vte + grid + pty + wgpu text; runs zsh, Neovim, htop
  correctly (vttest passes the core screens); true colour; resize with reflow.
- **T1 — smooth.** Pixel scrolling with momentum, cursor animation, ligatures,
  font fallback, 120 Hz without dropped frames (measured).
- **T2 — shell integration.** zsh plugin; prompt/command/output marks; blocks;
  jump, select, collapse, rerun; exit codes and durations.
- **T3 — completion popup.** zsh completions with descriptions, history
  ghost text, preview pane, fuzzy filter; bash and fish after.
- **T4 — Vim and Lua.** zett-edit in scrollback and on the command line;
  `term.lua`: every setting, keymap, popup style, trigger, hook, renderer.
- **T5 — panes and sessions.** splits, tabs, the session server, attach over
  SSH, saved layouts; WezTerm and tmux can retire.
- **T6 — zett.** Terminal panes and live terminal blocks in pages, command
  blocks into pages, agents driving panes, shell widgets.

## Name

Clear on the US trademark register on 2026-10-16 (EU/DPMA still to check by
hand; ask before anything permanent):

| Name | Why |
|---|---|
| **zash** | zett + shell (zsh/bash); short, sounds like a terminal |
| **zelt** | German for tent: the place you work from |
| **zettel** | a note slip; zett's terminal where every command is a note |
| **kastl** | from Zettelkasten, the box the notes live in |

Taken for software: kasten, kast, flint, glint, wick, ember, tinder.

## Starting config: from WezTerm

zash's default configuration starts from the user's WezTerm setup
(`~/.config/wezterm/`), **only its important features**: fonts and sizes,
colours (tinct), padding and opacity/blur, the tmux-style keys and pane/tab
bindings, the status/tab bar look, and the ii-decor button overlay. Not a
line-by-line port: each feature is re-expressed the zash way (`term.lua`), and
WezTerm-only workarounds are dropped. Do this at T4 (Lua config).

## The bar: better than the best, measured

zash is compared, with numbers, against the best terminals there are, on
their own strengths. A release that loses a comparison is not done.

| Rival | Its strength | zash must |
|---|---|---|
| Ghostty | native feel, correctness, speed | match its vttest/esctest correctness and its throughput |
| Kitty | graphics + keyboard protocols, kittens | support both protocols fully; Lua does what kittens do |
| Alacritty | raw throughput, low latency | ≥ its vtebench throughput, ≤ its input latency |
| WezTerm | Lua config, multiplexing, SSH domains | everything your WezTerm config uses, better (see "Starting config") |
| Warp | blocks, completions, AI | blocks and popups as good, local-first, no account, no cloud |
| Neovide | smoothness, cursor animation | equally smooth, in a real terminal |

Measured on every release, on this laptop: input latency (Typometer-style),
throughput (vtebench), frame time while scrolling at 120 Hz, cold start, memory
per tab, idle CPU (must be ~0), and correctness (vttest, esctest).
