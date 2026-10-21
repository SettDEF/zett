//! The Doom side of the editor: windows, pickers, dashboard, modeline, and the hand-offs to zeit.

use super::*;

/// Windows as a tree of splits; each leaf is a window id.
#[derive(Clone, Debug)]
pub(crate) enum WTree {
    Leaf(usize),
    /// true: side by side
    Split(bool, Box<WTree>, Box<WTree>),
}

impl WTree {
    pub(crate) fn ids(&self) -> Vec<usize> {
        match self {
            WTree::Leaf(i) => vec![*i],
            WTree::Split(_, a, b) => {
                let mut v = a.ids();
                v.extend(b.ids());
                v
            }
        }
    }

    pub(crate) fn place(&self, r: Rect, out: &mut Vec<(usize, Rect)>, seps: &mut Vec<Rect>) {
        match self {
            WTree::Leaf(i) => out.push((*i, r)),
            WTree::Split(side, a, b) => {
                if *side {
                    let wa = r.width.saturating_sub(1) / 2;
                    a.place(Rect { width: wa, ..r }, out, seps);
                    seps.push(Rect { x: r.x + wa, width: 1, ..r });
                    b.place(Rect { x: r.x + wa + 1, width: r.width.saturating_sub(wa + 1), ..r }, out, seps);
                } else {
                    let ha = r.height / 2;
                    a.place(Rect { height: ha, ..r }, out, seps);
                    b.place(Rect { y: r.y + ha, height: r.height.saturating_sub(ha), ..r }, out, seps);
                }
            }
        }
    }

    fn split(&mut self, at: usize, new: usize, side: bool) {
        match self {
            WTree::Leaf(i) if *i == at => *self = WTree::Split(side, Box::new(WTree::Leaf(at)), Box::new(WTree::Leaf(new))),
            WTree::Leaf(_) => {}
            WTree::Split(_, a, b) => {
                a.split(at, new, side);
                b.split(at, new, side);
            }
        }
    }

    /// As split, with the new window first (left of or above the old one).
    fn split_before(&mut self, at: usize, new: usize, side: bool) {
        match self {
            WTree::Leaf(i) if *i == at => *self = WTree::Split(side, Box::new(WTree::Leaf(new)), Box::new(WTree::Leaf(at))),
            WTree::Leaf(_) => {}
            WTree::Split(_, a, b) => {
                a.split_before(at, new, side);
                b.split_before(at, new, side);
            }
        }
    }

    fn remove(self, id: usize) -> Option<WTree> {
        match self {
            WTree::Leaf(i) if i == id => None,
            WTree::Leaf(_) => Some(self),
            WTree::Split(s, a, b) => match (a.remove(id), b.remove(id)) {
                (Some(a), Some(b)) => Some(WTree::Split(s, Box::new(a), Box::new(b))),
                (Some(x), None) | (None, Some(x)) => Some(x),
                (None, None) => None,
            },
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Win {
    pub doc: usize,
    pub top: usize,
    pub left: usize,
}

/// What a picker lists, and what Enter does with it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Pick {
    Files(PathBuf),
    Recent,
    Buffers,
    Projects,
    Grep(PathBuf),
    Lines,
    Commands,
    Diagnostics,
    Docs,
    /// a query block's results (query.rs), "path:line: text"
    Query,
    /// the files and their saved histories (everything.rs), live as Grep
    Everything(PathBuf),
    /// commits (git.rs): the history of one block
    Commits,
}

impl Pick {
    /// Its history file's name.
    pub fn name(&self) -> &'static str {
        match self {
            Pick::Files(_) => "files",
            Pick::Recent => "recent",
            Pick::Buffers => "buffers",
            Pick::Projects => "projects",
            Pick::Grep(_) => "grep",
            Pick::Lines => "lines",
            Pick::Commands => "commands",
            Pick::Diagnostics => "diagnostics",
            Pick::Docs => "docs",
            Pick::Query => "query",
            Pick::Everything(_) => "everything",
            Pick::Commits => "commits",
        }
    }

    /// The source searches by itself as you type (ripgrep): no filtering here.
    pub fn live(&self) -> bool {
        matches!(self, Pick::Grep(_) | Pick::Everything(_))
    }

    /// Choices worth remembering, and listed first next time (not lines of one buffer, nor buffers: they come in the order they were used).
    pub fn remembers(&self) -> bool {
        matches!(self, Pick::Files(_) | Pick::Recent | Pick::Projects | Pick::Commands | Pick::Docs)
    }

    /// Enter with nothing matching uses what was typed: a new file, a folder.
    pub fn takes_typed(&self) -> bool {
        matches!(self, Pick::Files(_) | Pick::Projects)
    }
}

pub(crate) fn state_dir() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/state")).join("zett")
}

/// Recently opened files, newest first.
pub(crate) fn recent() -> Vec<String> {
    std::fs::read_to_string(state_dir().join("recent")).map(|t| t.lines().filter(|l| !l.is_empty()).map(String::from).collect()).unwrap_or_default()
}

pub(crate) fn remember(path: &Path) {
    let p = std::fs::canonicalize(path).unwrap_or(path.to_path_buf()).display().to_string();
    let mut list = recent();
    list.retain(|x| *x != p);
    list.insert(0, p);
    list.truncate(200);
    let _ = std::fs::create_dir_all(state_dir());
    let _ = std::fs::write(state_dir().join("recent"), list.join("\n") + "\n");
}

/// The folder with `.git` above a path (the project it belongs to).
pub(crate) fn project_of(p: &Path) -> Option<PathBuf> {
    let mut d = if p.is_dir() { Some(p) } else { p.parent() };
    while let Some(x) = d {
        if x.join(".git").exists() {
            return Some(x.to_path_buf());
        }
        d = x.parent();
    }
    None
}

fn home_short(p: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    match p.strip_prefix(&home) {
        Some(rest) if !home.is_empty() => format!("~{rest}"),
        _ => p.to_string(),
    }
}

/// The branch checked out in a project (from .git/HEAD; no git process).
pub(crate) fn git_branch(root: &Path) -> Option<String> {
    let head = std::fs::read_to_string(project_of(root)?.join(".git/HEAD")).ok()?;
    Some(head.trim().strip_prefix("ref: refs/heads/").map(String::from).unwrap_or_else(|| head.trim().chars().take(7).collect()))
}

pub(crate) use zett_script::file_icon;

const BANNER: &[&str] = &[
    "███████╗███████╗████████╗████████╗",
    "╚══███╔╝██╔════╝╚══██╔══╝╚══██╔══╝",
    "  ███╔╝ █████╗     ██║      ██║   ",
    " ███╔╝  ██╔══╝     ██║      ██║   ",
    "███████╗███████╗   ██║      ██║   ",
    "╚══════╝╚══════╝   ╚═╝      ╚═╝   ",
];

/// The dashboard's buttons: icon, label, keys, the command.
const DASH: &[(&str, &str, &str, &str)] = &[
    ("\u{f1da}", "Recently opened files", "SPC f r", "recent-files"),
    ("\u{f07b}", "Open project", "SPC p p", "switch-project"),
    ("\u{f002}", "Find file", "SPC SPC", "find-file"),
    ("\u{f002}", "Search project", "SPC /", "search-project"),
    ("\u{f013}", "Open private configuration", "SPC f p", "open-config"),
    ("\u{f11c}", "Keys", "SPC h k", "help-keys"),
];

impl App {
    /// `:help TOPIC`: the topic in a buffer of its own (Markdown, read like any other).
    /// `:help X`, found as Neovim finds a tag: a topic, a setting (`wrap`, `'ic'`), a command, then a
    /// heading in any topic; nothing found names the nearest. The place found pulses for a moment.
    pub(crate) fn open_help(&mut self, topic: &str) {
        let t = topic.trim().trim_matches('\'').trim_start_matches(':');
        let t = if t.is_empty() { "index" } else { t };
        // a topic by its name
        if t.len() >= 3 && "commands".starts_with(t) {
            let text = zett_script::docs::commands_topic(&self.engine);
            return self.show_help("commands", text, None);
        }
        if t.len() >= 3 && ("settings".starts_with(t) || "options".starts_with(t)) {
            return self.show_help("settings", crate::settings::settings_topic(), None);
        }
        if let Some((n, text)) = zett_script::docs::get(t) {
            return self.show_help(n, text.to_string(), None);
        }
        // a setting (by name or short name): its entry on the settings page
        if let Some(s) = crate::settings::find(t.trim_start_matches("no")).or_else(|| crate::settings::find(t)) {
            let text = crate::settings::settings_topic();
            let at = text.lines().position(|l| l.starts_with(&format!("- `{}`", s.name)));
            return self.show_help("settings", text, at.map(|l| (l, 2)));
        }
        // a command: its entry on the commands page
        if self.engine.commands().iter().any(|(n, _, _)| *n == t) {
            let text = zett_script::docs::commands_topic(&self.engine);
            let at = text.lines().position(|l| l.starts_with(&format!("- `{t}`")));
            return self.show_help("commands", text, at.map(|l| (l, 1)));
        }
        // a heading in any topic: the whole section
        let want = t.to_lowercase();
        for (name, _, text) in zett_script::docs::TOPICS {
            let lines: Vec<&str> = text.lines().collect();
            if let Some(h) = lines.iter().position(|l| l.starts_with('#') && l.trim_start_matches('#').trim().to_lowercase().contains(&want)) {
                let level = lines[h].chars().take_while(|c| *c == '#').count();
                let end = lines.iter().skip(h + 1).position(|l| l.starts_with('#') && l.chars().take_while(|c| *c == '#').count() <= level).map(|e| h + 1 + e).unwrap_or(lines.len());
                return self.show_help(name, text.to_string(), Some((h, (end - h).min(14))));
            }
        }
        // nothing: the nearest names
        let mut names: Vec<String> = zett_script::docs::TOPICS.iter().map(|(n, _, _)| n.to_string()).collect();
        names.extend(crate::settings::SETTINGS.iter().map(|s| s.name.to_string()));
        names.extend(self.engine.commands().iter().map(|(n, _, _)| n.to_string()));
        let mut near: Vec<(usize, String)> = names.into_iter().map(|n| (crate::settings::distance(&want, &n.to_lowercase()), n)).filter(|(d, n)| *d <= 3 || n.contains(&want)).collect();
        near.sort();
        near.truncate(4);
        if near.is_empty() {
            self.say(format!("no help for \"{t}\": :help index lists the topics"));
        } else {
            self.say(format!("no help for \"{t}\"; maybe: {}", near.into_iter().map(|(_, n)| n).collect::<Vec<_>>().join(", ")));
        }
    }

    /// A help page as a buffer (one per topic, reused), at `at`: (line, lines to pulse).
    fn show_help(&mut self, name: &str, text: String, at: Option<(usize, usize)>) {
        let title = format!("help: {name}");
        let i = match self.docs.iter().position(|d| d.name == title) {
            Some(i) => i,
            None => {
                self.docs.push(Doc::new(Editor::from_str(&text), title, "Markdown".into()));
                self.docs.len() - 1
            }
        };
        self.show_doc(i);
        if let Some((line, n)) = at {
            self.goto_line(line + 1);
            // the heading near the top of the window, the section under it
            self.docs[i].top = line.saturating_sub(2);
            if let Some(Some(w)) = self.wins.get_mut(self.focus) {
                w.top = line.saturating_sub(2);
            }
            self.flash = Some((i, line, line + n.max(1), Instant::now()));
        }
    }

    /// Enter in a help buffer: a topic name opens it, a command name its entry in `commands`.
    pub(crate) fn follow_help(&mut self) -> bool {
        let d = &self.docs[self.cur];
        if !d.name.starts_with("help: ") {
            return false;
        }
        let c = d.ed.cursor();
        let line: Vec<char> = d.ed.buf.line(c.line).chars().collect();
        let word_char = |ch: char| ch.is_alphanumeric() || ch == '-' || ch == '_';
        let (mut a, mut b) = (c.col.min(line.len()), c.col.min(line.len()));
        while a > 0 && word_char(line[a - 1]) {
            a -= 1;
        }
        while b < line.len() && word_char(line[b]) {
            b += 1;
        }
        let w: String = line[a..b].iter().collect();
        if w.is_empty() {
            return false;
        }
        if w == "commands" || zett_script::docs::TOPICS.iter().any(|(n, _, _)| *n == w) {
            self.open_help(&w);
            return true;
        }
        if self.engine.commands().iter().any(|(n, _, _)| *n == w) {
            self.open_help("commands");
            let at = (0..self.docs[self.cur].ed.buf.len_lines()).find(|&l| self.docs[self.cur].ed.buf.line(l).starts_with(&format!("- `{w}`")));
            if let Some(l) = at {
                self.goto_line(l + 1);
            }
            return true;
        }
        false
    }

    // ── windows ──

    pub(crate) fn focus_win(&mut self, id: usize) {
        if let Some(w) = self.wins.get(id).and_then(|w| w.as_ref()) {
            self.focus = id;
            self.cur = w.doc.min(self.docs.len() - 1);
        }
    }

    fn split(&mut self, side: bool) {
        let w = self.wins[self.focus].clone().unwrap_or_default();
        self.wins.push(Some(w));
        let new = self.wins.len() - 1;
        self.tree.split(self.focus, new, side);
        self.focus_win(new);
    }

    pub(crate) fn close_win(&mut self) -> bool {
        if self.tree.ids().len() < 2 {
            return false;
        }
        let old = std::mem::replace(&mut self.tree, WTree::Leaf(0));
        self.tree = old.remove(self.focus).unwrap_or(WTree::Leaf(0));
        self.wins[self.focus] = None;
        let next = self.tree.ids()[0];
        self.focus_win(next);
        true
    }

    fn only_win(&mut self) {
        for id in self.tree.ids() {
            if id != self.focus {
                self.wins[id] = None;
            }
        }
        self.tree = WTree::Leaf(self.focus);
    }

    /// The window beside the focused one (left, right, up, down), nearest centre first.
    fn neighbour(&self, dir: &str) -> Option<usize> {
        let &(_, r) = self.win_rects.iter().find(|(i, _)| *i == self.focus)?;
        let (cx, cy) = (r.x as i32 + r.width as i32 / 2, r.y as i32 + r.height as i32 / 2);
        self.win_rects
            .iter()
            .filter(|(i, o)| {
                *i != self.focus
                    && match dir {
                        "left" => o.right() <= r.x + 1,
                        "right" => o.x + 1 >= r.right(),
                        "up" => o.bottom() <= r.y + 1,
                        _ => o.y + 1 >= r.bottom(),
                    }
            })
            .min_by_key(|(_, o)| (o.x as i32 + o.width as i32 / 2 - cx).abs() + (o.y as i32 + o.height as i32 / 2 - cy).abs())
            .map(|(i, _)| *i)
    }

    /// The buffer goes to the window on that side (a new one there when there is none); this
    /// window shows the buffer before, or closes when it would only show the same again.
    pub(crate) fn move_to_win(&mut self, dir: &str) {
        let (src, doc) = (self.focus, self.cur);
        if let Some(w) = self.wins[src].as_mut() {
            w.doc = doc;
        }
        let other = self.last_doc.filter(|l| *l != doc && *l < self.docs.len()).or_else(|| (0..self.docs.len()).find(|i| *i != doc));
        let target = match self.neighbour(dir) {
            Some(t) => t,
            None => {
                if other.is_none() {
                    return self.say("one buffer: nothing to move (SPC w v splits)");
                }
                let w = self.wins[src].clone().unwrap_or_default();
                self.wins.push(Some(w));
                let new = self.wins.len() - 1;
                let side = dir == "left" || dir == "right";
                if dir == "left" || dir == "up" { self.tree.split_before(src, new, side) } else { self.tree.split(src, new, side) }
                new
            }
        };
        if let Some(w) = self.wins[target].as_mut() {
            if w.doc != doc {
                w.doc = doc;
                w.top = 0;
                w.left = 0;
            }
        }
        match other {
            Some(o) => {
                if let Some(w) = self.wins[src].as_mut() {
                    w.doc = o;
                    w.top = 0;
                    w.left = 0;
                }
                self.focus_win(target);
            }
            None => {
                self.focus_win(target);
                let keep = self.focus;
                self.focus = src;
                self.close_win();
                self.focus_win(keep);
            }
        }
    }

    fn focus_dir(&mut self, dir: &str) {
        // from a panel: back to the windows (towards them), else stay
        if let Some(id) = self.engine.focused_panel() {
            let side = self.engine.panels().into_iter().find(|p| p.id == id).map(|p| p.side).unwrap_or_default();
            if !(side == dir) {
                self.engine.focus_panel(None);
            }
            return;
        }
        let ids = self.tree.ids();
        if dir == "next" {
            let i = ids.iter().position(|x| *x == self.focus).unwrap_or(0);
            return self.focus_win(ids[(i + 1) % ids.len()]);
        }
        match self.neighbour(dir) {
            Some(i) => self.focus_win(i),
            None => {
                self.focus_panel_at(dir);
            }
        }
    }

    /// A buffer went away: windows that showed it show another.
    pub(crate) fn doc_removed(&mut self, gone: usize) {
        let n = self.docs.len().max(1);
        for w in self.wins.iter_mut().flatten() {
            if w.doc > gone {
                w.doc -= 1;
            } else if w.doc == gone {
                w.doc = gone.min(n - 1);
                w.top = 0;
                w.left = 0;
            }
        }
    }

    // ── pickers (the minibuffer) ──

    /// The minibuffer over a ready list (references, symbols).
    pub(crate) fn pick_list(&mut self, kind: Pick, title: &str, items: Vec<Cand>) {
        self.dash = false;
        self.minibuf = Some(Minibuf::new(kind, title, items));
    }

    pub(crate) fn pick(&mut self, kind: Pick) {
        let (title, items): (String, Vec<Cand>) = match &kind {
            Pick::Files(root) => (format!("Find file in {}", root.file_name().map_or("/".into(), |n| n.to_string_lossy())), list_files(root).into_iter().map(Cand::plain).collect()),
            Pick::Recent => ("Recent file".into(), recent().into_iter().map(|p| Cand::plain(home_short(&p))).collect()),
            Pick::Buffers => {
                // as Emacs: the buffer before this one first, this one last
                let mut order: Vec<usize> = (0..self.docs.len()).filter(|&i| i != self.cur).collect();
                if let Some(p) = self.last_doc.and_then(|l| order.iter().position(|&i| i == l)) {
                    let l = order.remove(p);
                    order.insert(0, l);
                }
                order.push(self.cur);
                let note = |d: &Doc| format!("{}{}", d.syntax, if d.dirty() { " · modified" } else { "" });
                ("Switch to buffer".into(), order.into_iter().map(|i| Cand::noted(self.buffer_label(&self.docs[i]), note(&self.docs[i]))).collect())
            }
            Pick::Projects => {
                let mut list: Vec<String> = Vec::new();
                let here = project_of(&self.root).unwrap_or(self.root.clone()).display().to_string();
                list.push(here);
                for f in recent() {
                    if let Some(p) = project_of(Path::new(&f)) {
                        let p = p.display().to_string();
                        if !list.contains(&p) {
                            list.push(p);
                        }
                    }
                }
                ("Switch to project".into(), list.iter().map(|p| Cand::plain(home_short(p))).collect())
            }
            Pick::Grep(root) => (format!("Search {}", home_short(&root.display().to_string())), Vec::new()),
            Pick::Lines => {
                let d = &self.docs[self.cur];
                ("Go to line".into(), (0..d.ed.buf.len_lines()).map(|i| Cand::plain(format!("{:>5}│ {}", i + 1, d.ed.buf.line(i).trim_end()))).collect())
            }
            Pick::Commands => {
                let keys = self.engine.keys_of(self.keymode());
                let emacs = self.emacs_keys();
                let items = self
                    .engine
                    .commands()
                    .into_iter()
                    .map(|(n, d, _)| {
                        let k = keys.get(&n).and_then(|v| v.first()).map(|k| if emacs { minibuf::emacs_keys(k) } else { k.clone() }).unwrap_or_default();
                        Cand { keys: k, ..Cand::noted(n, d) }
                    })
                    .collect();
                ("M-x ".into(), items)
            }
            Pick::Query => ("Query results".into(), Vec::new()),
            Pick::Commits => ("Commits".into(), Vec::new()),
            Pick::Everything(root) => (format!("Search everything in {} (⟲ = its history)", home_short(&root.display().to_string())), Vec::new()),
            Pick::Docs => ("Documentation".into(), zett_script::docs::TOPICS.iter().map(|(n, about, _)| Cand::noted(*n, *about)).collect()),
            Pick::Diagnostics => {
                let mut rows = Vec::new();
                for d in &self.docs {
                    let name = d.name.strip_prefix(&format!("{}/", self.root.display())).unwrap_or(&d.name).to_string();
                    let mut ds = d.diags.clone();
                    ds.sort_by_key(|x| (x.severity, x.line));
                    for x in ds.iter().filter(|x| x.severity <= 2) {
                        rows.push(Cand::plain(format!("{name}:{}: {} {}", x.line + 1, if x.severity == 1 { "error" } else { "warning" }, x.message)));
                    }
                }
                ("Errors and warnings".into(), rows)
            }
        };
        self.pick_list(kind, &title, items);
    }

    /// The project search, again for the query typed so far (ripgrep, at most 400 lines).
    pub(crate) fn grep_update(&mut self) {
        let Some(m) = self.minibuf.as_mut() else { return };
        let (Pick::Grep(root) | Pick::Everything(root)) = &m.kind else { return };
        let history = matches!(m.kind, Pick::Everything(_));
        if m.query().chars().count() < 2 {
            m.set_items(Vec::new());
            return;
        }
        let out = std::process::Command::new("rg")
            .args(["--line-number", "--no-heading", "--smart-case", "--max-columns", "200", "--max-count", "50", "--fixed-strings", "--", m.query()])
            .current_dir(root)
            .stderr(std::process::Stdio::null())
            .output();
        let mut rows: Vec<Cand> = out.map(|o| String::from_utf8_lossy(&o.stdout).lines().take(400).map(Cand::plain).collect()).unwrap_or_default();
        // and what the files once had
        if history {
            rows.extend(everything::past(root, m.query(), 200).into_iter().map(Cand::plain));
        }
        m.set_items(rows);
    }

    /// Enter in a picker.
    pub(crate) fn picked(&mut self, kind: Pick, item: String) {
        let expand_home = |s: &str| s.strip_prefix('~').map(|r| format!("{}{r}", std::env::var("HOME").unwrap_or_default())).unwrap_or(s.to_string());
        match kind {
            Pick::Files(root) => self.open(&root.join(&item)),
            Pick::Recent => self.open(Path::new(&expand_home(&item))),
            Pick::Buffers => {
                if let Some(i) = self.docs.iter().position(|d| self.buffer_label(d) == item) {
                    self.show_doc(i);
                }
            }
            Pick::Projects => {
                let p = PathBuf::from(expand_home(&item));
                self.root = p.clone();
                self.pick(Pick::Files(p));
            }
            Pick::Everything(root) if self.open_past(&root, &item) => {}
            Pick::Commits => self.picked_commit(&item),
            Pick::Grep(root) | Pick::Everything(root) => {
                let mut parts = item.splitn(3, ':');
                if let (Some(f), Some(l)) = (parts.next(), parts.next().and_then(|l| l.parse::<usize>().ok())) {
                    self.open(&root.join(f));
                    self.goto_line(l);
                }
            }
            Pick::Lines => {
                if let Some(l) = item.split('│').next().and_then(|n| n.trim().parse::<usize>().ok()) {
                    self.goto_line(l);
                }
            }
            Pick::Diagnostics | Pick::Query => {
                let mut parts = item.splitn(3, ':');
                if let (Some(f), Some(l)) = (parts.next(), parts.next().and_then(|l| l.trim().parse::<usize>().ok())) {
                    let p = if Path::new(f).is_absolute() { PathBuf::from(f) } else { self.root.join(f) };
                    self.open(&p);
                    self.goto_line(l);
                }
            }
            Pick::Docs => {
                let name = item.split_whitespace().next().unwrap_or("").to_string();
                self.open_help(&name);
            }
            Pick::Commands => {
                let name = item.split_whitespace().next().unwrap_or("").to_string();
                match self.with_lua(|e| e.run(&name)) {
                    Ok(fx) => self.after_lua(fx),
                    Err(e) => self.say(e),
                }
            }
        }
    }

    /// A buffer as the minibuffer lists it: inside the project relative to it, else from ~.
    fn buffer_label(&self, d: &Doc) -> String {
        let root = format!("{}/", self.root.display());
        d.name.strip_prefix(&root).map(String::from).unwrap_or_else(|| home_short(&d.name))
    }

    pub(crate) fn goto_line(&mut self, l: usize) {
        let last = self.docs[self.cur].ed.buf.len_lines().saturating_sub(1);
        self.doc().ed.set_cursor(Pos { line: l.saturating_sub(1).min(last), col: 0 });
    }

    /// Shows buffer `i` in the focused window, remembering the one before.
    /// The current buffer one place left (-1) or right (+1) in the tab line; the windows keep theirs.
    pub(crate) fn move_doc(&mut self, by: i32) {
        let from = self.cur;
        let to = (from as i32 + by).clamp(0, self.docs.len() as i32 - 1) as usize;
        if to == from {
            return;
        }
        let d = self.docs.remove(from);
        self.docs.insert(to, d);
        let remap = |k: usize| if k == from { to } else if from < to && k > from && k <= to { k - 1 } else if to < from && k >= to && k < from { k + 1 } else { k };
        for w in self.wins.iter_mut().flatten() {
            w.doc = remap(w.doc);
        }
        self.last_doc = self.last_doc.map(remap);
        self.cur = to;
    }

    /// Every other buffer closed, but the ones with unsaved changes (they stay, and it says so).
    pub(crate) fn close_others(&mut self) {
        let keep = self.cur;
        let mut kept = 0;
        for i in (0..self.docs.len()).rev() {
            if i == keep { continue; }
            if self.docs[i].dirty() { kept += 1; continue; }
            self.docs.remove(i);
            if self.cur > i { self.cur -= 1; }
            self.doc_removed(i);
        }
        self.last_doc = None;
        if kept > 0 {
            self.say(format!("{kept} with unsaved changes stay open"));
        }
    }

    pub(crate) fn show_doc(&mut self, i: usize) {
        if i != self.cur {
            self.last_doc = Some(self.cur);
        }
        self.cur = i.min(self.docs.len() - 1);
        self.dash = false;
    }

    // ── the editor's UI commands (z.ui from editor.lua) ──

    /// Runs one; false when the name is not one of these.
    pub(crate) fn doom_ui(&mut self, name: &str, arg: &str) -> bool {
        match name {
            "findFile" => {
                let root = project_of(&self.root).unwrap_or(self.root.clone());
                self.pick(Pick::Files(root));
            }
            "findFileHere" => {
                let here = Path::new(&self.docs[self.cur].name).parent().filter(|p| p.is_dir()).map(Path::to_path_buf).unwrap_or(self.root.clone());
                self.pick(Pick::Files(here));
            }
            "recentFiles" => self.pick(Pick::Recent),
            "switchBuffer" => self.pick(Pick::Buffers),
            "switchProject" => self.pick(Pick::Projects),
            "grep" => {
                let root = project_of(&self.root).unwrap_or(self.root.clone());
                self.pick(Pick::Grep(root));
            }
            "searchEverything" => {
                let root = project_of(&self.root).unwrap_or(self.root.clone());
                self.pick(Pick::Everything(root));
            }
            "grepHere" => {
                let here = Path::new(&self.docs[self.cur].name).parent().filter(|p| p.is_dir()).map(Path::to_path_buf).unwrap_or(self.root.clone());
                self.pick(Pick::Grep(here));
            }
            "searchLines" => self.pick(Pick::Lines),
            "commands" => self.pick(Pick::Commands),
            "timeTravel" => self.travel_start(),
            "queryOpen" => self.query_open(),
            "backlinks" => self.backlinks(),
            "notebookRun" => self.notebook_run(),
            "gitStatus" => self.git_status(),
            "gitBlockHistory" => self.git_blame_section(),
            "notebookInsert" => self.notebook_insert(),
            "agentAsk" => self.agent_ask(),
            "save" => self.save(None),
            "yankPath" => {
                let p = self.docs[self.cur].name.clone();
                let ok = clipboard::copy(&p);
                self.say(if ok { format!("copied {p}") } else { p });
            }
            "killBuffer" => {
                // the last buffer gives way to the dashboard, not to quitting
                self.close(false);
                if self.quit {
                    self.quit = false;
                    self.dash = true;
                }
            }
            "nextBuffer" => self.show_doc((self.cur + 1) % self.docs.len()),
            "prevBuffer" => self.show_doc((self.cur + self.docs.len() - 1) % self.docs.len()),
            "lastBuffer" => {
                if let Some(l) = self.last_doc.filter(|l| *l < self.docs.len()) {
                    self.show_doc(l);
                }
            }
            "moveBufferLeft" => self.move_doc(-1),
            "moveBufferRight" => self.move_doc(1),
            "gotoBuffer" => {
                if let Some(i) = arg.trim().parse::<usize>().ok().filter(|n| (1..=self.docs.len()).contains(n)) {
                    self.show_doc(i - 1);
                }
            }
            "closeOtherBuffers" => self.close_others(),
            "newBuffer" | "scratch" => {
                self.docs.push(Doc::scratch());
                let i = self.docs.len() - 1;
                self.show_doc(i);
            }
            "dashboard" => self.dash = true,
            "split" => self.split(arg == "right"),
            "focusWin" => self.focus_dir(arg),
            "moveToWin" => self.move_to_win(arg),
            "layer" => self.layer_toggle(),
            "closeWin" => {
                if !self.close_win() {
                    self.say("the last window: SPC q q quits");
                }
            }
            "onlyWin" => self.only_win(),
            "terminal" | "terminalWindow" => self.terminal(name == "terminalWindow", None),
            "git" => self.terminal(true, Some("lazygit")),
            "compile" => {
                let last = self.last_compile.clone().unwrap_or_else(|| {
                    let root = project_of(&self.root).unwrap_or(self.root.clone());
                    if root.join("Cargo.toml").exists() { "cargo build".into() } else if root.join("Makefile").exists() { "make".into() } else { String::new() }
                });
                self.cmdline = Some(format!("compile {last}"));
            }
            "recompile" => match self.last_compile.clone() {
                Some(c) => self.compile(&c),
                None => self.say("nothing compiled yet: SPC c c"),
            },
            "toggle" => {
                match arg {
                    "numbers" => self.numbers = !self.numbers,
                    "relative" => self.relative = !self.relative,
                    "tabs" => self.tabs = !self.tabs,
                    _ => {}
                }
            }
            "comment" => self.comment(),
            "docs" => {
                let mut rows: Vec<Cand> = zett_script::docs::TOPICS.iter().map(|(n, about, _)| Cand::noted(*n, *about)).collect();
                rows.push(Cand::noted("commands", "every command and its keys"));
                self.pick_list(Pick::Docs, "Documentation", rows);
            }
            "definition" => self.lsp_definition(),
            "hover" => self.lsp_hover(),
            "diag" => self.next_diag(arg != "prev"),
            "diagnostics" => self.pick(Pick::Diagnostics),
            "references" => self.lsp_references(),
            "symbols" => self.lsp_symbols(),
            "format" => self.lsp_format(),
            "rename" => {
                let w = self.word_at_cursor();
                self.cmdline = Some(format!("rename {w}"));
            }
            "quit" => {
                if arg != "force" && self.docs.iter().any(|d| d.dirty()) {
                    self.say("unsaved changes: SPC q Q quits anyway, :wqa saves them");
                } else {
                    self.quit = true;
                }
            }
            _ => return false,
        }
        true
    }

    /// A terminal for the project: a zeit pane below (or a window), else a zash window.
    fn terminal(&mut self, window: bool, cmd: Option<&str>) {
        let root = project_of(&self.root).unwrap_or(self.root.clone()).display().to_string();
        let ok = if std::env::var_os("ZEIT_PANE").is_some() {
            let mut a: Vec<String> = if window { vec!["new-window".into(), "-c".into(), root.clone()] } else { vec!["split-window".into(), "-v".into(), "-p".into(), "30".into(), "-c".into(), root.clone()] };
            if let Some(c) = cmd {
                if window {
                    a.splice(1..1, ["-n".to_string(), c.to_string()]);
                }
                a.push(c.into());
            }
            std::process::Command::new("zeit").args(&a).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().is_ok_and(|s| s.success())
        } else {
            let mut a = vec!["-d".to_string(), root.clone()];
            if let Some(c) = cmd {
                a.extend(["--".to_string(), c.to_string()]);
            }
            std::process::Command::new("setsid").arg("zash").args(&a).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().is_ok()
        };
        if !ok {
            self.say("no terminal: run zett inside zeit or zash");
        }
    }

    /// `:compile CMD`: a zeit job in a pane below (its line in the bar, a note when done),
    /// else a zash window.
    pub(crate) fn compile(&mut self, cmd: &str) {
        let cmd = cmd.trim();
        if cmd.is_empty() {
            return;
        }
        self.last_compile = Some(cmd.to_string());
        let root = project_of(&self.root).unwrap_or(self.root.clone()).display().to_string();
        let ok = if std::env::var_os("ZEIT_PANE").is_some() {
            std::process::Command::new("zeit").args(["job-run", "-c", &root, "--", cmd]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().is_ok_and(|s| s.success())
        } else {
            std::process::Command::new("setsid")
                .args(["zash", "-d", &root, "--", "sh", "-c", &format!("{cmd}; printf '\\n[exit %s] Enter closes' $?; read _")])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .is_ok()
        };
        self.say(if ok { format!("compile: {cmd}") } else { "could not start it".into() });
    }

    // ── drawing ──

    /// The dashboard's banner, buttons and footer switch: the config's (zett.dashboard{…}), each
    /// part falling back to zett's own when the config leaves it out.
    fn dash_parts(&self) -> (Vec<String>, Vec<(String, String, String, String)>, bool) {
        let own = || DASH.iter().map(|(i, l, k, c)| (i.to_string(), l.to_string(), k.to_string(), c.to_string())).collect();
        match self.engine.dashboard() {
            Some(d) => (
                if d.banner.is_empty() { BANNER.iter().map(|s| s.to_string()).collect() } else { d.banner },
                if d.buttons.is_empty() { own() } else { d.buttons },
                d.footer,
            ),
            None => (BANNER.iter().map(|s| s.to_string()).collect(), own(), true),
        }
    }

    /// The dashboard, Doom's way: the banner, the buttons, a line about the config.
    pub(crate) fn draw_dashboard(&self, f: &mut Frame, a: Rect) {
        let (banner, buttons, footer) = self.dash_parts();
        let mut lines: Vec<Line> = Vec::new();
        let banner_h = banner.len() as u16;
        let rows = banner_h + 2 + buttons.len() as u16 * 2 + 3;
        let pad = a.height.saturating_sub(rows) / 2;
        for _ in 0..pad {
            lines.push(Line::raw(""));
        }
        for b in &banner {
            lines.push(Line::from(Span::styled(b.clone(), hl("DashboardHeader"))).centered());
        }
        lines.push(Line::raw(""));
        lines.push(Line::raw(""));
        for (i, (icon, label, keys, _)) in buttons.iter().enumerate() {
            let sel = i == self.dash_sel;
            let st = if sel { hl("DashboardHeader") } else { Style::default() };
            let text = format!("{icon}  {label:<30}");
            lines.push(Line::from(vec![Span::styled(text, st), Span::styled(format!("{keys:>8}"), hl("DashboardKey"))]).centered());
            lines.push(Line::raw(""));
        }
        lines.push(Line::raw(""));
        if footer {
            let n = self.engine.plugins.len();
            let foot = format!("zett loaded {n} plugin{} in {} ms · {}", if n == 1 { "" } else { "s" }, self.started_ms, home_short(&self.root.display().to_string()));
            lines.push(Line::from(Span::styled(foot, hl("DashboardFooter"))).centered());
        }
        f.render_widget(Paragraph::new(lines), a);
    }

    /// Enter on the dashboard runs the chosen button.
    pub(crate) fn dash_run(&mut self) {
        let buttons = self.dash_parts().1;
        let Some(name) = buttons.get(self.dash_sel.min(buttons.len().saturating_sub(1))).map(|b| b.3.clone()) else { return };
        match self.with_lua(|e| e.run(&name)) {
            Ok(fx) => self.after_lua(fx),
            Err(e) => self.say(e),
        }
    }

    pub(crate) fn dash_len(&self) -> usize {
        self.dash_parts().1.len().max(1)
    }

    /// A window's modeline (doom-modeline): state, icon, name, position | branch, mode, place.
    pub(crate) fn draw_modeline(&self, f: &mut Frame, a: Rect, di: usize, focused: bool) {
        let d = &self.docs[di];
        let (label, color) = mode_badge(d);
        let c = d.ed.cursor();
        let lines = d.ed.buf.len_lines().max(1);
        let place = if d.top == 0 && lines <= a.height as usize { "All".to_string() } else if c.line == 0 { "Top".into() } else if c.line + 1 >= lines { "Bot".into() } else { format!("{}%", c.line * 100 / (lines - 1).max(1)) };
        let rel = d.name.strip_prefix(&format!("{}/", self.root.display())).unwrap_or(&d.name).to_string();
        let dim = hl("StatusLineDim");
        let mut left = vec![];
        if focused {
            left.push(Span::styled(format!(" {label} "), hl(color)));
        } else {
            left.push(Span::styled(format!(" {label} "), dim));
        }
        left.push(Span::raw(" "));
        left.push(Span::styled(format!("{} ", file_icon(&d.name)), if focused { hl("StatusLineFile") } else { dim }));
        left.push(Span::styled(rel, if focused { Style::default().add_modifier(Modifier::BOLD) } else { dim }));
        if d.dirty() {
            left.push(Span::styled(" ●", hl("Modified")));
        }
        left.push(Span::styled(format!("  {}:{}", c.line + 1, c.col + 1), dim));
        let mut right: Vec<Span> = Vec::new();
        let (errs, warns) = (d.diags.iter().filter(|x| x.severity == 1).count(), d.diags.iter().filter(|x| x.severity == 2).count());
        if errs > 0 {
            right.push(Span::styled(format!("\u{f00d} {errs}  "), hl("DiagnosticError")));
        }
        if warns > 0 {
            right.push(Span::styled(format!("\u{f071} {warns}  "), hl("DiagnosticWarn")));
        }
        if d.lsp && errs + warns == 0 && focused {
            right.push(Span::styled("\u{f00c}  ", hl("DiagnosticOk")));
        }
        if let Some(b) = &self.branch {
            right.push(Span::styled(format!("\u{e725} {b}  "), if focused { hl("StatusLineBranch") } else { dim }));
        }
        right.push(Span::styled(format!("{}  ", d.syntax), if focused { Style::default() } else { dim }));
        right.push(Span::styled(format!("{place} "), dim));
        let used: usize = left.iter().chain(right.iter()).map(|s| s.content.chars().count()).sum();
        let mut spans = left;
        spans.push(Span::raw(" ".repeat((a.width as usize).saturating_sub(used))));
        spans.extend(right);
        f.render_widget(Paragraph::new(Line::from(spans)).style(hl(if focused { "StatusLine" } else { "StatusLineNC" })), a);
    }
}

/// The line-comment marker for a file, by its extension.
pub(crate) fn comment_prefix(name: &str) -> &'static str {
    match Path::new(name).extension().and_then(|e| e.to_str()).unwrap_or("") {
        "py" | "sh" | "zsh" | "bash" | "fish" | "toml" | "yaml" | "yml" | "conf" | "nix" | "rb" | "pl" | "r" | "ini" | "cfg" | "mk" | "cmake" => "#",
        "lua" | "sql" | "hs" | "elm" => "--",
        "vim" => "\"",
        "tex" | "sty" => "%",
        "el" | "lisp" | "clj" | "scm" => ";;",
        _ if name.ends_with("Makefile") || name.ends_with(".gitignore") => "#",
        _ => "//",
    }
}

impl App {
    /// gcc / visual gc: the line, or the selected lines, commented out or back in.
    pub(crate) fn comment(&mut self) {
        let prefix = comment_prefix(&self.docs[self.cur].name);
        let d = &mut self.docs[self.cur];
        let (from, to) = match d.ed.selection_range() {
            Some((a, b)) => (d.ed.buf.to_pos(a).line, d.ed.buf.to_pos(b.saturating_sub(1).max(a)).line),
            None => (d.ed.cursor().line, d.ed.cursor().line),
        };
        let before = d.ed.cursor().line;
        d.ed.toggle_comment(from, to, prefix);
        d.touched(before.min(from));
    }
}


/// The modeline's first field and its highlight group: Vim's mode, or in the Emacs profile
/// "E" ("MARK" while a region is active).
fn mode_badge(d: &Doc) -> (&'static str, &'static str) {
    if d.ed.is_emacs() {
        return if d.ed.region().is_some() { ("MARK", "ModeVisual") } else { ("E", "ModeNormal") };
    }
    match d.ed.mode() {
        Mode::Normal => ("N", "ModeNormal"),
        Mode::Insert => ("I", "ModeInsert"),
        Mode::Visual => ("V", "ModeVisual"),
        Mode::VisualLine => ("L", "ModeVisual"),
    }
}
