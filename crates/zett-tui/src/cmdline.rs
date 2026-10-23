//! What can be typed after `:`: the commands with an example each, and their arguments.

use super::*;

/// Vim's commands zett knows: (name, what follows, what it does, an example).
pub(crate) const EX: &[(&str, &str, &str, &str)] = &[
    ("w", "[file]", "save (to another file)", "w notes.md"),
    ("e", "[file]", "open a file (no name: find one)", "e src/main.rs"),
    ("q", "", "close the window (the last: quit)", "q"),
    ("wq", "", "save and close", "wq"),
    ("wa", "", "save every buffer", "wa"),
    ("qa", "", "quit (q! qa!: without saving)", "qa"),
    ("sp", "", "split below", "sp"),
    ("vs", "", "split right", "vs"),
    ("only", "", "only this window", "only"),
    ("bn", "", "next buffer", "bn"),
    ("bp", "", "previous buffer", "bp"),
    ("bd", "", "close the buffer", "bd"),
    ("bmove", "<+1 | -1 | N>", "move this buffer in the tab line", "bmove +1"),
    ("bonly", "", "close the other buffers", "bonly"),
    ("ls", "", "the buffers", "ls"),
    ("cd", "<folder>", "the project folder", "cd ~/code/zett"),
    ("help", "[topic]", "the documentation", "help lsp"),
    ("set", "<setting> <value>", "change a setting (no value: on)", "set explorer.width 40"),
    ("hi", "<group> [fg= bg= bold …]", "colours: show or change a group", "hi Comment fg=#7f8c8d italic"),
    ("compile", "<command>", "run a build as a zeit job", "compile cargo build"),
    ("rename", "<name>", "rename the symbol (language server)", "rename parse_line"),
    ("lua", "<code>", "run Lua with the zett API", "lua zett.notify(\"hi\")"),
    ("s", "/pattern/replacement/[g][i]", "substitute (% in front: every line)", "%s/foo/bar/g"),
    ("g", "/pattern/[d | p | s/…/…/ | normal …]", "on every line that matches", "g/TODO/d"),
    ("v", "/pattern/command", "on every line that does not match", "v/fn /d"),
    ("sort", "[n] [u] [i] [!]", "sort the lines (numbers, unique, reverse)", "sort u"),
    ("d", "", "delete the lines (range in front)", "3,5d"),
    ("y", "", "copy the lines to the clipboard", "%y"),
    ("m", "<line>", "move the lines after a line", "m0"),
    ("t", "<line>", "copy the lines after a line", "t$"),
    ("j", "[!]", "join lines", "j"),
    (">", "", "indent the lines", "'<,'>>"),
    ("<", "", "unindent the lines", "%<"),
    ("normal", "<keys>", "Vim keys on each line", "%normal A;"),
    ("!", "<command>", "run a shell command (range: filter lines)", "%!jq ."),
    ("r", "<file> | !<command>", "insert a file or a command's output", "r !date"),
    ("retab", "[width]", "tabs into spaces", "retab 4"),
    ("earlier", "<n> | <n>s m h d", "back in time (undo tree)", "earlier 5m"),
    ("later", "<n> | <n>s m h d", "forward in time", "later 1m"),
    ("undo", "[n]", "undo, or go to undo state n", "undo 3"),
    ("undolist", "", "the undo states", "undolist"),
    ("messages", "", "every message so far", "messages"),
    ("e!", "", "reload the file, dropping changes", "e!"),
    ("enew", "", "a new empty buffer", "enew"),
    ("new", "", "a new buffer below", "new"),
    ("vnew", "", "a new buffer at the right", "vnew"),
    ("saveas", "<file>", "save under a new name", "saveas copy.rs"),
    ("update", "", "save if changed", "update"),
    ("b", "<n | name | #>", "go to a buffer (#: the one before)", "b main"),
    ("bfirst", "", "the first buffer", "bfirst"),
    ("blast", "", "the last buffer", "blast"),
    ("bufdo", "<command>", "a command in every buffer", "bufdo %s/a/b/g | update"),
    ("pwd", "", "the project folder", "pwd"),
    ("file", "", "name, lines, changed", "file"),
    ("echo", "<lua>", "print a Lua value", "echo 6 * 7"),
    ("source", "[file | %]", "run a Lua file (% this one)", "so %"),
    ("nmap", "<keys> <command | :ex<CR>>", "map keys (for this session)", "nmap <leader>w :w<CR>"),
    ("unmap", "<keys>", "remove a mapping", "unmap <leader>w"),
    ("command", "<Name> <ex line>", "a new : command", "command Fmt %!rustfmt"),
    ("wincmd", "<h j k l w v s c o>", "window commands", "wincmd v"),
    ("terminal", "", "a terminal (zeit pane)", "terminal"),
    ("make", "", "compile", "make"),
    ("grep", "<pattern>", "search the project", "grep TODO"),
    ("Explore", "", "the explorer", "Explore"),
    ("UndotreeToggle", "", "the undo tree", "UndotreeToggle"),
    ("checkhealth", "", "what zett runs with, what is missing", "checkhealth"),
    ("LspInfo", "", "running language servers", "LspInfo"),
    ("version", "", "zett's version", "version"),
    ("=", "", "the number of lines", "="),
    ("<number>", "", "go to that line", "42"),
];

/// One suggestion: what Tab puts on the line, the name shown, its arguments, a description.
#[derive(Clone)]
pub(crate) struct Suggestion {
    pub fill: String,
    pub name: String,
    pub args: String,
    pub about: String,
    pub example: String,
}

impl App {
    /// Suggestions for the line as typed so far.
    pub(crate) fn suggestions(&self, line: &str) -> Vec<Suggestion> {
        // a range in front (%, 3,5, '<,'>) stays on the line; suggestions are for what follows
        let cur = self.docs[self.cur].ed.cursor().line;
        let (range, after) = ex::parse_range(line, cur, usize::MAX / 2, None);
        let prefix = &line[..line.len() - after.len()];
        if range.is_some() {
            return self
                .suggestions(after)
                .into_iter()
                .map(|mut s| {
                    s.fill = format!("{prefix}{}", s.fill);
                    s
                })
                .collect();
        }
        let (head, rest) = match line.split_once(' ') {
            Some((h, r)) => (h, Some(r)),
            None => (line, None),
        };
        // arguments of a command
        if let Some(arg) = rest {
            // a file name: :e :w :sp :vs :r :cd … (files.rs)
            if matches!(head, "e" | "edit" | "sp" | "split" | "vs" | "vsplit" | "new" | "vnew" | "r" | "read" | "w" | "write" | "sav" | "saveas" | "cd" | "lcd" | "tcd" | "find" | "fin" | "sf" | "sfind" | "tabe" | "tabedit" | "badd" | "source" | "so" | "diffsplit" | "wq" | "x" | "up" | "update") {
                return self.file_suggestions(head, arg);
            }
            let pick = |items: Vec<(String, String, String)>| -> Vec<Suggestion> {
                items
                    .into_iter()
                    .filter(|(n, _, _)| n.starts_with(arg.split(' ').next().unwrap_or("")) || arg.is_empty())
                    .map(|(n, args, about)| {
                        // a list is set in init.lua; anything else gets an example with its value
                        let list = args.starts_with('{');
                        let example = if head == "set" && !list { format!("set {n} {}", args.split(" (default").next().unwrap_or("")) } else { String::new() };
                        let args = if list { "{…} (init.lua)".to_string() } else if args.chars().count() > 26 { format!("{}…", args.chars().take(25).collect::<String>()) } else { args };
                        Suggestion { fill: format!("{head} {n} "), name: n, args, about, example }
                    })
                    .collect()
            };
            return match head {
                "set" => pick(
                    self.engine
                        .options()
                        .into_iter()
                        .map(|(n, cur, def, about)| (n, if cur == def { cur } else { format!("{cur} (default {def})") }, about))
                        .chain([("number", "on / nonumber", "line numbers"), ("relativenumber", "on / norelativenumber", "relative line numbers"), ("tabs", "on / notabs", "buffer tabs")].map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string())))
                        .collect(),
                ),
                "hi" | "highlight" => {
                    let items: Vec<Suggestion> = self
                        .engine
                        .highlights()
                        .into_iter()
                        .filter(|(n, _, _)| n.to_lowercase().starts_with(&arg.split(' ').next().unwrap_or("").to_lowercase()))
                        .map(|(n, raw, _)| Suggestion { fill: format!("hi {n} "), example: format!("hi {n} {}", if raw.starts_with("link") { "fg=3 bold".to_string() } else { raw.clone() }), name: n, args: raw, about: String::new() })
                        .collect();
                    items
                }
                "h" | "help" => pick(zett_script::docs::TOPICS.iter().map(|(n, about, _)| (n.to_string(), String::new(), about.to_string())).chain([("commands".to_string(), String::new(), "every command and its keys".to_string())]).collect()),
                "compile" => {
                    let root = doom::project_of(&self.root).unwrap_or(self.root.clone());
                    let mut v: Vec<(String, String, String)> = Vec::new();
                    if let Some(l) = &self.last_compile {
                        v.push((l.clone(), String::new(), "the last one".into()));
                    }
                    if root.join("Cargo.toml").exists() {
                        for c in ["cargo build", "cargo run", "cargo test", "cargo clippy", "cargo build --release"] {
                            v.push((c.into(), String::new(), "Cargo".into()));
                        }
                    }
                    if root.join("Makefile").exists() {
                        v.push(("make".into(), String::new(), "Makefile".into()));
                    }
                    v.into_iter()
                        .filter(|(n, _, _)| n.starts_with(arg))
                        .map(|(n, args, about)| Suggestion { fill: format!("compile {n}"), name: n, args, about, example: String::new() })
                        .collect()
                }
                _ => Vec::new(),
            };
        }
        // a command name: Vim's first, then zett's (M-x), names starting with it before others
        let q = head.to_lowercase();
        let mut out: Vec<(u8, Suggestion)> = Vec::new();
        for (n, args, about, ex) in EX {
            let rank = if n.starts_with(&q) { 0 } else if fuzzy(&q.chars().collect::<Vec<_>>(), n).is_some() { 2 } else { continue };
            if *n == "<number>" && !q.is_empty() && !q.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let fill = if args.is_empty() || *n == "<number>" { n.to_string() } else { format!("{n} ") };
            out.push((rank, Suggestion { fill, name: n.to_string(), args: args.to_string(), about: about.to_string(), example: ex.to_string() }));
        }
        for (n, about, _) in self.engine.commands() {
            let rank = if n.starts_with(&q) { 1 } else if fuzzy(&q.chars().collect::<Vec<_>>(), &n).is_some() { 3 } else { continue };
            out.push((rank, Suggestion { fill: n.clone(), name: n, args: String::new(), about, example: String::new() }));
        }
        out.sort_by_key(|(r, s)| (*r, s.name.len()));
        out.into_iter().map(|(_, s)| s).collect()
    }

    /// The list over the command line: name, what follows, what it does; an example under it.
    /// Where `:` asks (the `cmdline` setting): the line's own row, whether it sits in a box, and
    /// whether suggestions open below it (a floating line) or above (the bottom row).
    pub(crate) fn cmd_place(&self, body: Rect, cmd: Rect) -> (Rect, bool, bool) {
        let w = (self.num(self.cur, "cmdwidth").max(20) as u16).min(body.width.saturating_sub(4)).max(10);
        let boxed = |x: u16, y: u16| (Rect { x: x + 1, y: y + 1, width: w.saturating_sub(2), height: 1 }, true, true);
        match self.setting(self.cur, "cmdline").as_str() {
            "top" => (Rect { x: body.x, y: body.y, width: body.width, height: 1 }, false, true),
            "center" => boxed(body.x + (body.width.saturating_sub(w)) / 2, body.y + body.height / 3),
            "cursor" => {
                let (cx, cy) = self.cursor_xy;
                let x = cx.saturating_sub(2).clamp(body.x, body.right().saturating_sub(w));
                // under the cursor's line, or over it near the bottom
                let y = if cy + 4 < body.bottom() { cy + 1 } else { cy.saturating_sub(3).max(body.y) };
                boxed(x, y)
            }
            _ => (cmd, false, false),
        }
    }

    pub(crate) fn draw_suggestions(&mut self, f: &mut Frame, cmd: Rect, boxed: bool, below: bool) {
        // nowildmenu: Tab still completes, no list is shown
        if !self.flag(self.cur, "wildmenu") {
            return;
        }
        let Some(line) = self.cmdline.clone() else { return };
        if self.prompt.is_some() {
            return;
        }
        let all = self.suggestions(&line);
        if all.is_empty() {
            return;
        }
        let shown = 10.min(all.len());
        self.cmd_sel = self.cmd_sel.min(all.len() - 1);
        let first = self.cmd_sel.saturating_sub(shown - 1);
        let sel = &all[self.cmd_sel];
        let foot = if !sel.example.is_empty() { 1 } else { 0 };
        let h = (shown + foot + 2) as u16;
        let w = f.area().width.min(90);
        let a = if below {
            // under the line (and its box), as far as the screen goes
            let y = cmd.y + 1 + boxed as u16;
            let x = cmd.x.saturating_sub(boxed as u16);
            Rect { x, y, width: w.min(f.area().width.saturating_sub(x)), height: h.min(f.area().height.saturating_sub(y)) }
        } else {
            Rect { x: cmd.x, y: cmd.y.saturating_sub(h), width: w, height: h.min(cmd.y) }
        };
        let name_w = all.iter().skip(first).take(shown).map(|s| s.name.chars().count() + 1 + s.args.chars().count()).max().unwrap_or(8).min(40);
        let mut lines = Vec::new();
        for (i, s) in all.iter().enumerate().skip(first).take(shown) {
            let on = i == self.cmd_sel;
            let bg = if on { hl("SuggestSel") } else { Style::default() };
            let used = s.name.chars().count() + 1 + s.args.chars().count();
            lines.push(Line::from(vec![
                Span::styled(format!(" {}", s.name), bg.patch(hl("SuggestName")).add_modifier(if on { Modifier::BOLD } else { Modifier::empty() })),
                Span::styled(format!(" {}", s.args), bg.patch(hl("SuggestArgs"))),
                Span::styled(" ".repeat(name_w.saturating_sub(used) + 2), bg),
                Span::styled(format!("{:<width$}", s.about, width = (w as usize).saturating_sub(name_w + 6)), bg.patch(hl(if on { "File" } else { "MsgArea" }))),
            ]));
        }
        if foot == 1 {
            lines.push(Line::from(vec![Span::styled(" e.g.  ", hl("Muted")), Span::styled(format!(":{}", sel.example), hl("SuggestExample"))]));
        }
        crate::layer(f, a);
        let more = if all.len() > shown { format!(" {}/{} ", self.cmd_sel + 1, all.len()) } else { String::new() };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(crate::display::float_set())
            .border_style(hl("FloatBorderDim"))
            .title_bottom(Line::from(Span::styled(format!(" Tab completes · ↑↓ choose{more}"), hl("FloatBorderDim"))).right_aligned());
        f.render_widget(Paragraph::new(lines).block(block), a);
    }

    /// Tab: the chosen suggestion onto the line.
    pub(crate) fn complete_cmdline(&mut self) {
        let Some(line) = self.cmdline.clone() else { return };
        // wildmode longest: first as far as every match agrees
        if self.setting(self.cur, "wildmode").split(',').next().is_some_and(|m| m.starts_with("longest")) {
            let list = self.suggestions(&line);
            if list.len() > 1 {
                let first: Vec<char> = list[0].fill.chars().collect();
                let n = list.iter().skip(1).map(|s| s.fill.chars().zip(&first).take_while(|(a, b)| a == *b).count()).min().unwrap_or(0);
                let common: String = first[..n].iter().collect();
                if common.chars().count() > line.chars().count() {
                    self.cmdline = Some(common);
                    return;
                }
            }
        }
        if let Some(s) = self.suggestions(&line).get(self.cmd_sel) {
            self.cmdline = Some(s.fill.clone());
            self.cmd_sel = 0;
        }
    }

    /// `:set a b=c d+=e`: each argument in turn (settings::set_args).
    pub(crate) fn set_command(&mut self, arg: &str) {
        let args = crate::settings::set_args(arg);
        if args.is_empty() {
            return self.set_one("");
        }
        for a in args {
            self.set_one(&a);
        }
    }

    /// One `:set` argument: `NAME=VALUE` (also `number`, `nonumber`, Vim's way), a plugin's option too.
    fn set_one(&mut self, arg: &str) {
        let (name, value) = arg.split_once([' ', '=']).map(|(a, b)| (a.trim(), b.trim())).unwrap_or((arg.trim(), ""));
        let flag = |s: &str| -> Option<(&'static str, bool)> {
            let (on, n) = match s.strip_prefix("no") {
                Some(n) => (false, n),
                None => (true, s),
            };
            Some((
                match n {
                    "number" | "nu" => "numbers",
                    "relativenumber" | "rnu" => "relative",
                    "tabs" => "tabs",
                    _ => return None,
                },
                on && !matches!(value, "off" | "false" | "0"),
            ))
        };
        // :set alone lists the core settings; a core one is set by settings.rs
        if name.is_empty() {
            return self.settings_list();
        }
        if self.set_core(arg, false) {
            return;
        }
        if let Some((which, on)) = flag(name) {
            match which {
                "numbers" => self.numbers = on,
                "relative" => self.relative = on,
                _ => self.tabs = on,
            }
            return self.say(format!("{name}: {}", if on { "on" } else { "off" }));
        }
        if value.is_empty() {
            if let Some((n, cur, def, about)) = self.engine.options().into_iter().find(|o| o.0 == name).filter(|o| o.2 != "true" && o.2 != "false") {
                return self.say(format!("{n} = {cur} (default {def}) · {about}"));
            }
        }
        match self.engine.set_option(name, value) {
            Ok(m) => {
                self.say(m);
                self.emit("option_set");
            }
            Err(e) => self.say(e),
        }
    }
}

impl App {
    /// `:hi` (every group), `:hi NAME` (how it looks), `:hi NAME fg=3 bold` (change it).
    pub(crate) fn hi_command(&mut self, arg: &str) {
        // Vim's forms: :hi [default] link From To, :hi clear Group, :hi default Group …
        let arg = arg.trim();
        let arg = arg.strip_prefix("default ").map(str::trim_start).unwrap_or(arg);
        if let Some(rest) = arg.strip_prefix("link ") {
            let mut w = rest.split_whitespace();
            if let (Some(from), Some(to)) = (w.next(), w.next()) {
                let args = if to.eq_ignore_ascii_case("NONE") { "clear".to_string() } else { format!("link={to}") };
                if let Err(e) = self.engine.set_highlight(from, &args) {
                    self.say(e);
                }
                return;
            }
        }
        if let Some(group) = arg.strip_prefix("clear ") {
            if let Err(e) = self.engine.set_highlight(group.trim(), "clear") {
                self.say(e);
            }
            return;
        }
        let (name, args) = arg.split_once(' ').map(|(a, b)| (a, b.trim())).unwrap_or((arg.trim(), ""));
        if name.is_empty() {
            let rows = self.engine.highlights().into_iter().map(|(n, raw, _)| (n, raw)).collect();
            self.popup = Some(Popup::Help("Highlight groups (:hi NAME fg=… bg=… bold)".into(), rows));
            return;
        }
        if args.is_empty() {
            let m = match self.engine.highlights().into_iter().find(|h| h.0 == name) {
                Some((n, raw, look)) if raw != look => format!("{n} {raw} → {look}"),
                Some((n, raw, _)) => format!("{n} {raw}"),
                None => format!("no group {name} (:hi lists them)"),
            };
            return self.say(m);
        }
        match self.engine.set_highlight(name, args) {
            // quiet as in Vim (nvim_set_hl from plugins goes through here)
            Ok(_) => {}
            Err(e) => self.say(e),
        }
    }
}
