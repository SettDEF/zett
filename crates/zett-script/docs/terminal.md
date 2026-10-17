# zett with zeit and zash

zett runs in any terminal; with zeit (the multiplexer) and zash (the terminal)
it does more:

- `SPC o t`  a terminal below: a zeit pane in the project
- `SPC g g`  lazygit in a zeit window
- `SPC c c`  compile: the command runs as a zeit job; its progress is in the
  bar, a notification comes when it ends (`SPC c C` runs it again)

Outside zeit these open zash windows instead.

In zash, `zash web ADDRESS` shows a web page in the terminal; inside zeit it
becomes a pane of its own.
