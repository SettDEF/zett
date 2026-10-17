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
    ("ls", "", "the buffers", "ls"),
    ("cd", "<folder>", "the project folder", "cd ~/code/zett"),
    ("help", "[topic]", "the documentation", "help lsp"),
    ("set", "<setting> <value>", "change a setting (no value: on)", "set explorer.width 40"),
    ("compile", "<command>", "run a build as a zeit job", "compile cargo build"),
    ("rename", "<name>", "rename the symbol (language server)", "rename parse_line"),
    ("lua", "<code>", "run Lua with the zett API", "lua zett.notify(\"hi\")"),
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
        let (head, rest) = match line.split_once(' ') {
            Some((h, r)) => (h, Some(r)),
            None => (line, None),
        };
        // arguments of a command
        if let Some(arg) = rest {
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
    pub(crate) fn draw_suggestions(&mut self, f: &mut Frame, cmd: Rect) {
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
        let a = Rect { x: cmd.x, y: cmd.y.saturating_sub(h), width: w, height: h.min(cmd.y) };
        let name_w = all.iter().skip(first).take(shown).map(|s| s.name.chars().count() + 1 + s.args.chars().count()).max().unwrap_or(8).min(40);
        let mut lines = Vec::new();
        for (i, s) in all.iter().enumerate().skip(first).take(shown) {
            let on = i == self.cmd_sel;
            let bg = if on { Style::default().bg(Color::Indexed(0)) } else { Style::default() };
            let used = s.name.chars().count() + 1 + s.args.chars().count();
            lines.push(Line::from(vec![
                Span::styled(format!(" {}", s.name), bg.fg(Color::Indexed(6)).add_modifier(if on { Modifier::BOLD } else { Modifier::empty() })),
                Span::styled(format!(" {}", s.args), bg.fg(Color::Indexed(8))),
                Span::styled(" ".repeat(name_w.saturating_sub(used) + 2), bg),
                Span::styled(format!("{:<width$}", s.about, width = (w as usize).saturating_sub(name_w + 6)), bg.fg(if on { Color::Reset } else { Color::Indexed(7) })),
            ]));
        }
        if foot == 1 {
            lines.push(Line::from(vec![Span::styled(" e.g.  ", Style::default().fg(Color::Indexed(8))), Span::styled(format!(":{}", sel.example), Style::default().fg(Color::Indexed(2)))]));
        }
        f.render_widget(Clear, a);
        let more = if all.len() > shown { format!(" {}/{} ", self.cmd_sel + 1, all.len()) } else { String::new() };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Indexed(8)))
            .title_bottom(Line::from(Span::styled(format!(" Tab completes · ↑↓ choose{more}"), Style::default().fg(Color::Indexed(8)))).right_aligned());
        f.render_widget(Paragraph::new(lines).block(block), a);
    }

    /// Tab: the chosen suggestion onto the line.
    pub(crate) fn complete_cmdline(&mut self) {
        let Some(line) = self.cmdline.clone() else { return };
        if let Some(s) = self.suggestions(&line).get(self.cmd_sel) {
            self.cmdline = Some(s.fill.clone());
            self.cmd_sel = 0;
        }
    }

    /// `:set NAME VALUE` (also `:set number`, `:set nonumber`, Vim's way).
    pub(crate) fn set_command(&mut self, arg: &str) {
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
        if name.is_empty() {
            self.cmdline = Some("set ".into());
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
