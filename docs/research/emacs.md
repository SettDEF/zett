# What zett takes from Emacs

Studied from GNU Emacs 31.1 (Org 9.8.7) as installed here: the Lisp reference
manual (`elisp.info`), the Org manual, and the sources in
`/usr/share/emacs/31.1/lisp` (which-key, transient, ob-core, org-element,
help, nadvice). Doom Emacs is not installed; its parts below are from its
documented design, to be checked against its source later.

## 1. Keys resolve through a stack of keymaps

Emacs searches, in order (elisp manual §23.8):

1. `overriding-terminal-local-map` (transient maps: a popup that grabs keys)
2. `overriding-local-map`
3. the `keymap` property **of the text at point**
4. emulation maps (this is where Evil lives), minor-mode maps
5. the `local-map` property at point, else the major mode's map
6. the global map

A binding can be a command, or another key sequence (a macro), and a found
command can still be **remapped** to another command.

**zett:** the same stack, with zett's nouns:

| Emacs | zett |
|---|---|
| transient map | an open popup / picker / which-key menu |
| keymap at point | the **block type under the cursor** (a table cell, a canvas card, a code block bring their own keys) |
| emulation maps (Evil) | the **mode** layer: normal / insert / visual / command |
| minor modes | **modules** (`tools.git`, `canvas`…), each can be on or off |
| major mode | the **view** (text, page, canvas, table) |
| global map | global |

Commands can be remapped (`zett.remap("page.delete", "my.trash")`), which
lets a module change behaviour everywhere without rebinding keys.

## 2. Everything is a command, and the editor explains itself

Every Emacs action is a command with an interactive spec (how to ask for its
arguments) and a docstring. `C-h k` names the command behind any key, `C-h f`
shows its doc and links to its source. That self-documentation is what makes
a huge system learnable.

**zett:** commands carry id, title, doc, param schema (how to prompt for each
param: picker, text, number) and their source location. `SPC h k`, `SPC h c`
and `SPC h b` (block types) show it; the palette and which-key read the same
titles.

## 3. Hooks and advice let anything extend anything

`nadvice` wraps any function: `:before`, `:after`, `:around`, `:override`,
`:filter-args`, `:filter-return`, removable at any time. Hooks run lists of
functions at named moments.

**zett:** Lua `zett.advise("block.delete", "before", fn)` with the same six
kinds on **commands** (not every internal function, so the core stays
optimisable), and named hooks (`page.open`, `block.changed`, `save`,
`mode.enter`…). Every advice and hook shows up in `SPC h c` for its command.

## 4. which-key and transient

- which-key (built in since Emacs 30): idle delay **1.0 s** and a side window
  of at most **25%** of the frame by default; Doom shortens the delay to about
  0.3 s. Pages through long menus with `C-h`.
- transient (Magit's menus, built in): a menu that stays open, with **toggles
  and options** you set before running the action (`-f force`, `--author=`).

**zett:** which-key popup at 0.3 s, sorted groups (`+files`, `+search`), paging.
Transient-style menus for commands with options (git, export, publish, query
builder), so complex commands stay keyboard-driven without memorising flags.

## 5. Org Babel is the model for live blocks

A source block runs and writes its result into the document. Header arguments
say how (Org manual §16.6):

- **collection**: `value` (the last expression, like a function) or `output`
  (stdout)
- **type**: table, list, scalar, file (an image)
- **format**: raw, code, html, drawer…
- **handling**: replace, append, prepend, silent
- plus `:session` (one interpreter kept alive), `:var x=other-block` (one
  block's result is another's input), `:cache`, noweb references

**zett:** live blocks keep this model, typed instead of string header args:
`{ lang, collect: value|output, as: table|list|text|image|chart, keep:
replace|append|silent, session, inputs: { x: "#block-id" }, cache }`.
Because inputs name other blocks, a page becomes a **reactive notebook**:
change a block and everything that reads it reruns. Org reruns by hand.

## 6. Org elements: structure with properties

Org parses a buffer into elements (headline, paragraph, table, src-block…),
each with properties (property drawers), and the agenda queries properties
across all files.

**zett:** the same, but the structure is the real data (the block tree), not
re-parsed text, and the agenda generalises to queries and database views.

## 7. One picker for everything

`completing-read` is the single API every chooser uses; Vertico, Orderless,
Consult and Embark replace its UI, matching, sources and **actions on the
highlighted candidate**, and everything that uses it improves at once.

**zett:** one picker API (sources, fuzzy matching, preview, Embark-style
action menu on any candidate). Quick open, commands, blocks, links, the clipper
and the Firefox extension all use it.

## 8. Doom's layer (from its design; verify against its source)

- `SPC` leader tree with mnemonic groups, the same everywhere
- `init.el` lists modules with flags (`(lang rust +lsp)`); `doom sync` installs
  and byte-compiles, a lockfile pins versions; `doom doctor` diagnoses
- aggressive lazy loading for fast start
- popup rules: help, REPLs and search open in dismissable side popups

All in zett's plan (PLAN.md, "The Doom Emacs layer").

## Next to study

- Evil's operator/motion/text-object grammar and its tests, for zett-edit
  (Neovim's behaviour is the reference; Evil shows how to build it on an
  Emacs-like core)
- Doom's module loader and `doom doctor` checks (needs a Doom checkout)
- Emacs's undo model vs undo-tree / vundo, for history over the CRDT
