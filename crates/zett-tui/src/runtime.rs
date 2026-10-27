//! Files zett reads as Vim does: :runtime :packadd :packloadall :scriptnames :colorscheme :compiler
//! (Neovim's own compiler files too), :mksession :mkview :loadview, and .vim files as Ex lines.

use super::*;

const NAMES: &[(&str, &str)] = &[
    ("runtime", "ru"), ("packadd", "pa"), ("packloadall", "packl"), ("scriptnames", "scr"), ("colorscheme", "colo"),
    ("compiler", "comp"), ("mksession", "mks"), ("mkview", "mkvie"), ("loadview", "lo"), ("trust", "trust"),
];

/// A typed name as one of these commands (its full name).
pub(crate) fn runtime_name(typed: &str) -> Option<&'static str> {
    NAMES.iter().find(|(full, short)| typed.len() >= short.len() && full.starts_with(typed)).map(|(f, _)| *f)
}

/// A .vim file's lines as Ex commands: `"` comments and blank lines left out, `\` continuation lines
/// joined to the line before.
pub(crate) fn vim_lines(src: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for l in src.lines() {
        let t = l.trim_start();
        if let Some(rest) = t.strip_prefix('\\') {
            if let Some(last) = out.last_mut() {
                last.push_str(rest);
            }
            continue;
        }
        if t.is_empty() || t.starts_with('"') {
            continue;
        }
        out.push(t.to_string());
    }
    out
}

/// A :set value as Vim's lines write it: `\ ` a space, `\\` a backslash, `\|` a bar.
fn set_value(v: &str) -> String {
    let mut out = String::new();
    let mut it = v.chars();
    while let Some(c) = it.next() {
        match (c, it.clone().next()) {
            ('\\', Some(n @ (' ' | '\\' | '|' | '"'))) => {
                out.push(n);
                it.next();
            }
            _ => out.push(c),
        }
    }
    out
}

/// The makeprg and errorformat a Vim compiler file sets (`CompilerSet makeprg=…`, `errorformat+=…`).
pub(crate) fn compiler_settings(src: &str) -> Vec<(String, String, bool)> {
    let mut out = Vec::new();
    for l in vim_lines(src) {
        let Some(rest) = l.strip_prefix("CompilerSet ") else { continue };
        let rest = rest.trim();
        for (name, cut) in [("makeprg", "makeprg"), ("makeprg", "mp"), ("errorformat", "errorformat"), ("errorformat", "efm")] {
            if let Some(v) = rest.strip_prefix(&format!("{cut}+=")) {
                out.push((name.to_string(), set_value(v), true));
                break;
            }
            if let Some(v) = rest.strip_prefix(&format!("{cut}=")) {
                out.push((name.to_string(), set_value(v), false));
                break;
            }
        }
    }
    out
}

impl App {
    /// 'runtimepath' for zett: its value when set, else zett's own folders (the config, then the
    /// data folder's site), and the packages :packadd added.
    pub(crate) fn rtp(&self) -> Vec<PathBuf> {
        let mut dirs: Vec<PathBuf> = match self.engine.opt("runtimepath") {
            Some(v) => v.split(',').filter(|d| !d.is_empty()).map(|d| PathBuf::from(d.replace("$HOME", &std::env::var("HOME").unwrap_or_default()))).collect(),
            None => {
                let data = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share"));
                vec![zett_script::config_dir(), data.join("zett/site")]
            }
        };
        dirs.extend(self.packs.iter().cloned());
        // start packages added since (zett.plug clones or links them while the config runs)
        let roots: Vec<PathBuf> = dirs.iter().map(|d| d.join("pack")).collect();
        for root in roots {
            for pack in std::fs::read_dir(&root).into_iter().flatten().flatten() {
                let mut found: Vec<PathBuf> = std::fs::read_dir(pack.path().join("start")).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
                found.sort();
                for p in found {
                    if !dirs.contains(&p) {
                        dirs.push(p);
                    }
                }
            }
        }
        dirs
    }

    /// The files `pattern` (a path in the runtime folders; `*` in its last part) names, in order.
    fn runtime_files(&self, pattern: &str) -> Vec<PathBuf> {
        let (dir, file) = match pattern.rsplit_once('/') {
            Some((d, f)) => (d.to_string(), f.to_string()),
            None => (String::new(), pattern.to_string()),
        };
        let mut out = Vec::new();
        for base in self.rtp() {
            let d = base.join(&dir);
            if !file.contains('*') {
                let p = d.join(&file);
                if p.is_file() {
                    out.push(p);
                }
                continue;
            }
            let (pre, post) = file.split_once('*').unwrap_or((&file, ""));
            let mut found: Vec<PathBuf> = std::fs::read_dir(&d).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| {
                let n = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                p.is_file() && n.starts_with(pre) && n.ends_with(post) && n.len() >= pre.len() + post.len()
            }).collect();
            found.sort();
            out.extend(found);
        }
        out
    }

    /// Runs a file: Lua, or a .vim file's Ex lines. Remembered for :scriptnames.
    pub(crate) fn source_file(&mut self, p: &Path) -> Result<(), String> {
        // verbose 1 and up: what is read is said
        if self.num(self.cur, "verbose") >= 1 {
            self.say(format!("sourcing \"{}\"", p.display()));
        }
        let code = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
        if !self.sourced.iter().any(|s| s == p) {
            self.sourced.push(p.to_path_buf());
        }
        // a .vim file: Vimscript (vimscript.rs), its Ex lines run as they come
        let rc = p.file_name().is_some_and(|n| matches!(n.to_str(), Some(".nvimrc" | ".exrc" | ".vimrc" | "_vimrc" | "vimrc")));
        if rc || p.extension().is_some_and(|e| e == "vim") {
            let name = p.display().to_string();
            self.vimscript_run(&code, Some(&name));
            return Ok(());
        }
        let fx = self.with_lua(|e| e.eval(&code))?.1;
        self.after_lua(fx);
        Ok(())
    }

    /// packpath: where packages are looked for (its value when set, else the runtime folders)
    fn packpath(&self) -> Vec<PathBuf> {
        match self.engine.opt("packpath") {
            Some(v) => v.split(',').filter(|d| !d.is_empty()).map(|d| PathBuf::from(d.replace("$HOME", &std::env::var("HOME").unwrap_or_default()).replacen('~', &std::env::var("HOME").unwrap_or_default(), 1))).collect(),
            None => self.rtp(),
        }
    }

    /// A package's folder by name: pack/*/opt/NAME (start/NAME with `start`) under the runtime folders.
    fn package(&self, name: &str, start: bool) -> Vec<PathBuf> {
        let kind = if start { "start" } else { "opt" };
        let mut out = Vec::new();
        for base in self.packpath() {
            for group in std::fs::read_dir(base.join("pack")).into_iter().flatten().flatten() {
                let d = group.path().join(kind);
                if start {
                    let mut all: Vec<PathBuf> = std::fs::read_dir(&d).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
                    all.sort();
                    out.extend(all);
                } else if d.join(name).is_dir() {
                    out.push(d.join(name));
                }
            }
        }
        out
    }

    /// A package added: in the runtime folders, and its plugin/ files run (not with `!`, as Vim).
    fn add_package(&mut self, dir: PathBuf, bang: bool) {
        if !self.packs.contains(&dir) {
            self.packs.push(dir.clone());
        }
        self.engine.add_lua_dir(&dir);
        if bang {
            return;
        }
        let mut files: Vec<PathBuf> = std::fs::read_dir(dir.join("plugin")).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "lua" || e == "vim")).collect();
        files.sort();
        for f in files {
            if let Err(e) = self.source_file(&f) {
                self.say(e);
            }
        }
    }

    /// One of these commands.
    pub(crate) fn runtime_command(&mut self, name: &str, arg: &str, bang: bool) {
        let Some(cmd) = runtime_name(name) else { return };
        let arg = arg.trim();
        match cmd {
            "runtime" => {
                let mut ran = 0;
                for pat in arg.split_whitespace() {
                    let files = self.runtime_files(pat);
                    for f in if bang { files } else { files.into_iter().take(1).collect() } {
                        match self.source_file(&f) {
                            Ok(()) => ran += 1,
                            Err(e) => return self.say(e),
                        }
                    }
                }
                if ran == 0 {
                    self.say(format!("not found in 'runtimepath': \"{arg}\""));
                }
            }
            "packadd" => {
                let Some(dir) = self.package(arg, false).into_iter().next() else { return self.say(format!("E919: Directory not found in 'packpath': \"pack/*/opt/{arg}\"")) };
                self.add_package(dir, bang);
            }
            "packloadall" => {
                if self.packs_loaded && !bang {
                    return;
                }
                self.packs_loaded = true;
                for dir in self.package("", true) {
                    self.add_package(dir, false);
                }
            }
            "trust" => self.trust_command(arg),
            "scriptnames" => {
                let mut rows: Vec<(String, String)> = vec![("1".into(), zett_script::config_dir().join("init.lua").display().to_string())];
                rows.extend(self.engine.plugins.iter().map(|p| (String::new(), format!("plugin {p}"))));
                rows.extend(self.sourced.iter().map(|p| (String::new(), p.display().to_string())));
                for (i, r) in rows.iter_mut().enumerate() {
                    r.0 = format!("{:>3}", i + 1);
                }
                self.popup = Some(Popup::Help("scriptnames".into(), rows));
            }
            "colorscheme" => {
                if arg.is_empty() {
                    let name = self.colors_name.clone();
                    return self.say(name);
                }
                // the runtime path first (plugins, ~/.config/zett/colors), then the ones Neovim ships
                let file = self.runtime_files(&format!("colors/{arg}.lua")).into_iter().chain(self.runtime_files(&format!("colors/{arg}.vim"))).chain(nvim_colors().into_iter().filter(|p| p.file_stem().is_some_and(|s| s == arg))).next();
                match file {
                    Some(f) => match self.source_file(&f) {
                        Ok(()) => self.colors_name = arg.to_string(),
                        Err(e) => self.say(e),
                    },
                    None => self.say(format!("E185: Cannot find color scheme '{arg}'")),
                }
            }
            "compiler" => {
                if arg.is_empty() {
                    let mut names: Vec<String> = self.runtime_files("compiler/*").into_iter().chain(nvim_compilers()).filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_string())).collect();
                    names.sort();
                    names.dedup();
                    return self.say(names.join(" "));
                }
                // zett's own compiler/NAME.lua or .vim first, then Neovim's compiler files
                let own = self.runtime_files(&format!("compiler/{arg}.lua")).into_iter().chain(self.runtime_files(&format!("compiler/{arg}.vim"))).next();
                let file = own.or_else(|| nvim_compilers().into_iter().find(|p| p.file_stem().is_some_and(|s| s == arg)));
                let Some(f) = file else { return self.say(format!("E666: Compiler not supported: {arg}")) };
                if f.extension().is_some_and(|e| e == "lua") {
                    if let Err(e) = self.source_file(&f) {
                        self.say(e);
                    }
                    return;
                }
                let src = std::fs::read_to_string(&f).unwrap_or_default();
                let sets = compiler_settings(&src);
                if sets.is_empty() {
                    return self.say(format!("{}: no CompilerSet line zett can read", f.display()));
                }
                for (name, value, add) in sets {
                    let value = if add { format!("{},{value}", self.setting(self.cur, &name)).trim_start_matches(',').to_string() } else { value };
                    // :compiler! for every buffer, :compiler for this one (Vim's CompilerSet is :setlocal)
                    self.set_core(&format!("{name}={value}"), !bang);
                }
                self.say(format!("compiler {arg}: makeprg {}", self.setting(self.cur, "makeprg")));
            }
            "mksession" => {
                let file = if arg.is_empty() { "Session.vim" } else { arg };
                let p = self.root.join(file);
                if p.exists() && !bang {
                    return self.say(format!("E189: \"{}\" exists (add ! to override)", p.display()));
                }
                let text = self.session_text();
                match std::fs::write(&p, text) {
                    Ok(()) => self.say(format!("session in {}", p.display())),
                    Err(e) => self.say(format!("E190: Cannot open \"{}\" for writing: {e}", p.display())),
                }
            }
            "mkview" | "loadview" => {
                let Some(file) = self.docs[self.cur].abs_path() else { return self.say("E32: No file name") };
                // :mkview N / :loadview N: one of nine more per file; a name: that file
                let slot = arg.parse::<u8>().ok().filter(|n| (1..=9).contains(n));
                let view = if !arg.is_empty() && slot.is_none() {
                    self.root.join(arg)
                } else {
                    let dir = PathBuf::from(expand_tilde(&self.setting(self.cur, "viewdir")));
                    let dir = if self.setting(self.cur, "viewdir").is_empty() { crate::doom::state_dir().join("view") } else { dir };
                    dir.join(format!("{}={}.vim", file.display().to_string().replace('/', "=+"), slot.map(|n| n.to_string()).unwrap_or_default()))
                };
                if cmd == "loadview" {
                    if !view.is_file() {
                        return self.say(format!("E484: Can't open file {}", view.display()));
                    }
                    if let Err(e) = self.source_file(&view) {
                        self.say(e);
                    }
                    return;
                }
                if view.exists() && !bang && !arg.is_empty() && slot.is_none() {
                    return self.say(format!("E189: \"{}\" exists (add ! to override)", view.display()));
                }
                let c = self.docs[self.cur].ed.cursor();
                let mut lines = vec!["\" zett view".to_string()];
                for (k, v) in self.docs[self.cur].local.iter() {
                    lines.push(format!("setlocal {k}={}", v.replace(' ', "\\ ")));
                }
                lines.push(format!("{}", c.line + 1));
                lines.push(format!("normal! 0{}", if c.col > 0 { format!("{}l", c.col) } else { String::new() }));
                let _ = std::fs::create_dir_all(view.parent().unwrap_or(Path::new(".")));
                if let Err(e) = std::fs::write(&view, lines.join("\n") + "\n") {
                    self.say(format!("E190: Cannot open \"{}\" for writing: {e}", view.display()));
                }
            }
            _ => {}
        }
    }

    /// :mksession's Ex lines: the folder, the buffers, the windows as split (each with its file and
    /// line), and which one has the focus.
    fn session_text(&self) -> String {
        let mut out = vec!["\" zett session: :source it (or zett -S) to come back here".to_string()];
        out.push(format!("cd {}", self.root.display().to_string().replace(' ', "\\ ")));
        out.push("only".into());
        out.push("set splitbelow splitright".into());
        for d in &self.docs {
            if let Some(p) = d.abs_path() {
                out.push(format!("badd {}", p.display().to_string().replace(' ', "\\ ")));
            }
        }
        self.session_tree(&self.tree, &mut out);
        // built, the first window has the focus; on to the one that had it (windows go in the tree's order)
        let at = self.tree.ids().iter().position(|&w| w == self.focus).unwrap_or(0);
        out.extend(std::iter::repeat_n("wincmd w".to_string(), at));
        out.join("\n") + "\n"
    }

    /// A part of the window tree as Ex lines: a window its file and line; a split the second part
    /// split off and filled first, then back to the first part (which ends with the focus).
    fn session_tree(&self, t: &crate::doom::WTree, out: &mut Vec<String>) {
        use crate::doom::WTree;
        match t {
            WTree::Leaf(id) => {
                let di = self.win_doc(*id);
                let d = &self.docs[di];
                if let Some(p) = d.abs_path() {
                    out.push(format!("edit {}", p.display().to_string().replace(' ', "\\ ")));
                }
                let line = if *id == self.focus { d.ed.cursor().line } else { self.wins.get(*id).and_then(|w| w.as_ref()).map_or(0, |w| w.top) };
                out.push(format!("{}", line + 1));
            }
            WTree::Split(side, a, b, _) => {
                out.push(if *side { "vsplit".into() } else { "split".into() });
                self.session_tree(b, out);
                out.push(if *side { "wincmd h".into() } else { "wincmd k".into() });
                self.session_tree(a, out);
            }
        }
    }
}

/// Neovim's own compiler files, when Neovim is installed.
fn nvim_compilers() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = ["/usr/share/nvim/runtime/compiler", "/usr/local/share/nvim/runtime/compiler"].iter().flat_map(|d| std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path())).filter(|p| p.extension().is_some_and(|e| e == "vim")).collect();
    v.sort();
    v
}

fn expand_tilde(s: &str) -> String {
    s.strip_prefix('~').map(|r| format!("{}{r}", std::env::var("HOME").unwrap_or_default())).unwrap_or_else(|| s.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn vim_lines_and_compiler_files() {
        let src = "\" a comment\nset ts=4\n  \\ sw=4\n\nCompilerSet makeprg=cargo\\ build\nCompilerSet errorformat=%f:%l:%c:\\ %m\nCompilerSet efm+=%-G%.%#\n";
        assert_eq!(super::vim_lines(src), vec!["set ts=4 sw=4", "CompilerSet makeprg=cargo\\ build", "CompilerSet errorformat=%f:%l:%c:\\ %m", "CompilerSet efm+=%-G%.%#"]);
        let sets = super::compiler_settings(src);
        assert_eq!(sets[0], ("makeprg".into(), "cargo build".into(), false));
        assert_eq!(sets[1], ("errorformat".into(), "%f:%l:%c: %m".into(), false));
        assert_eq!(sets[2], ("errorformat".into(), "%-G%.%#".into(), true));
    }

    #[test]
    fn neovims_compiler_files() {
        let Some(src) = std::fs::read_to_string("/usr/share/nvim/runtime/compiler/cargo.vim").ok() else { return };
        let sets = super::compiler_settings(&src);
        assert!(sets.iter().any(|(n, v, _)| n == "makeprg" && v == "cargo $*"), "{sets:?}");
        let efm = sets.iter().find(|(n, _, add)| n == "errorformat" && *add).map(|s| s.1.clone()).unwrap_or_default();
        assert!(efm.starts_with("%-G%\\s%#Downloading%.%#,") && efm.contains("Could not compile"), "{efm}");
    }

    #[test]
    fn a_session_brings_the_windows_back() {
        let dir = std::env::temp_dir().join(format!("zett-session-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (a, b) = (dir.join("a.txt"), dir.join("b.txt"));
        std::fs::write(&a, "a1\na2\na3\n").unwrap();
        std::fs::write(&b, "b1\nb2\n").unwrap();
        let keep = std::env::current_dir().unwrap();
        let mut app = crate::App::new(vec![a.clone()]);
        app.command(&format!("cd {}", dir.display()));
        app.command("vsplit");
        app.command(&format!("e {}", b.display()));
        app.goto_line(2);
        app.command("mksession");
        let text = std::fs::read_to_string(dir.join("Session.vim")).unwrap();
        assert!(text.contains("vsplit") && text.contains("b.txt") && text.contains("a.txt"), "{text}");
        let mut fresh = crate::App::new(Vec::new());
        fresh.command(&format!("source {}", dir.join("Session.vim").display()));
        let shown: Vec<String> = fresh.tree.ids().iter().map(|&w| fresh.docs[fresh.win_doc(w)].name.clone()).collect();
        assert_eq!(shown.len(), 2, "{shown:?}");
        assert!(fresh.docs[fresh.cur].name.ends_with("b.txt"), "the focus back on b: {}", fresh.docs[fresh.cur].name);
        assert_eq!(fresh.docs[fresh.cur].ed.cursor().line, 1, "on its line");
        let _ = std::env::set_current_dir(&keep);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn runtime_packages_colors_and_sessions() {
        let base = std::env::temp_dir().join(format!("zett-rtp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("colors")).unwrap();
        std::fs::create_dir_all(base.join("pack/x/opt/foo/plugin")).unwrap();
        std::fs::create_dir_all(base.join("compiler")).unwrap();
        std::fs::write(base.join("colors/mine.vim"), "set tabstop=7\n").unwrap();
        std::fs::write(base.join("pack/x/opt/foo/plugin/foo.vim"), "set shiftwidth=5\n").unwrap();
        std::fs::write(base.join("compiler/mycc.vim"), "CompilerSet makeprg=mycc\\ -x\n").unwrap();
        let mut app = crate::App::new(Vec::new());
        app.command(&format!("set runtimepath={}", base.display()));
        app.command("colorscheme mine");
        assert_eq!(app.colors_name, "mine");
        assert_eq!(app.setting(app.cur, "tabstop"), "7", ":colorscheme ran colors/mine.vim");
        app.command("packadd foo");
        assert_eq!(app.setting(app.cur, "shiftwidth"), "5", ":packadd ran its plugin/ files");
        // packpath: packages looked for there, not in the runtime folders
        let other = base.join("other");
        std::fs::create_dir_all(other.join("pack/y/opt/bar/plugin")).unwrap();
        std::fs::write(other.join("pack/y/opt/bar/plugin/bar.vim"), "set softtabstop=3\n").unwrap();
        app.command(&format!("set packpath={}", other.display()));
        app.command("packadd bar");
        assert_eq!(app.setting(app.cur, "softtabstop"), "3", "bar found by packpath");
        app.command("compiler mycc");
        assert_eq!(app.setting(app.cur, "makeprg"), "mycc -x");
        app.command("scriptnames");
        let rows = match &app.popup { Some(crate::Popup::Help(_, r)) => r.clone(), _ => Vec::new() };
        assert!(rows.iter().any(|(_, f)| f.ends_with("mine.vim")) && rows.iter().any(|(_, f)| f.ends_with("foo.vim")), "{rows:?}");
        let _ = std::fs::remove_dir_all(&base);
    }
}

/// What :trust says of a folder config (exrc): allowed with this content, denied, or not known.
#[derive(Debug, PartialEq)]
pub(crate) enum Trust {
    Allowed,
    Denied,
    Unknown,
}

#[cfg(test)]
thread_local! {
    /// a test's own trust list (tests run side by side; the state folder is shared)
    static TEST_TRUST: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// The trust list: one line per file, "HASH PATH" allowed (that content only) or "! PATH" denied.
fn trust_file() -> PathBuf {
    #[cfg(test)]
    if let Some(p) = TEST_TRUST.with(|t| t.borrow().clone()) {
        return p;
    }
    doom::state_dir().join("trust")
}

fn content_hash(p: &Path) -> Option<String> {
    let bytes = std::fs::read(p).ok()?;
    Some(format!("{:016x}", bytes.iter().fold(0xcbf29ce484222325u64, |h, b| (h ^ *b as u64).wrapping_mul(0x100000001b3))))
}

fn trust_entries() -> Vec<(String, String)> {
    std::fs::read_to_string(trust_file()).unwrap_or_default().lines().filter_map(|l| l.split_once(' ').map(|(a, b)| (a.to_string(), b.to_string()))).collect()
}

/// Whether file `p` is trusted as it is now (a changed file is not, as in Neovim).
pub(crate) fn trust_of(p: &Path) -> Trust {
    let path = std::fs::canonicalize(p).unwrap_or(p.to_path_buf()).display().to_string();
    match trust_entries().into_iter().find(|(_, f)| *f == path) {
        Some((h, _)) if h == "!" => Trust::Denied,
        Some((h, _)) if Some(&h) == content_hash(p).as_ref() => Trust::Allowed,
        _ => Trust::Unknown,
    }
}

impl App {
    /// :trust [++deny | ++remove] [file]: the file (the current buffer's) allowed as it is now,
    /// denied, or taken off the list.
    pub(crate) fn trust_command(&mut self, arg: &str) {
        let (mode, file) = match arg.split_once(' ').map(|(a, b)| (a, b.trim())).unwrap_or((arg, "")) {
            (m @ ("++deny" | "++remove"), f) => (m, f.to_string()),
            (f, "") if !f.starts_with("++") => ("", f.to_string()),
            (m, f) => (m, f.to_string()),
        };
        let file = if file.is_empty() { self.docs[self.cur].abs_path().map(|p| p.display().to_string()).unwrap_or_default() } else { file };
        if file.is_empty() {
            return self.say("E5300: :trust needs a file (this buffer has none)");
        }
        let p = PathBuf::from(&file);
        let path = std::fs::canonicalize(&p).unwrap_or(p.clone()).display().to_string();
        let mut entries: Vec<(String, String)> = trust_entries().into_iter().filter(|(_, f)| *f != path).collect();
        let said = match mode {
            "++remove" => format!("\"{path}\" removed from the trust list"),
            "++deny" => {
                entries.push(("!".into(), path.clone()));
                format!("\"{path}\" denied")
            }
            _ => match content_hash(&p) {
                Some(h) => {
                    entries.push((h, path.clone()));
                    format!("\"{path}\" trusted")
                }
                None => return self.say(format!("E5300: cannot read \"{path}\"")),
            },
        };
        if let Some(d) = trust_file().parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let text: String = entries.iter().map(|(h, f)| format!("{h} {f}\n")).collect();
        let _ = std::fs::write(trust_file(), text);
        self.say(said);
    }
}

#[cfg(test)]
mod trust_tests {
    use super::*;

    #[test]
    fn trust_allow_deny_remove_and_changed_content() {
        let dir = std::env::temp_dir().join(format!("zett-trust-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        TEST_TRUST.with(|t| *t.borrow_mut() = Some(dir.join("trust")));
        let rc = dir.join(".exrc");
        std::fs::write(&rc, "set tw=40\n").unwrap();
        let mut app = crate::App::new(Vec::new());
        assert_eq!(trust_of(&rc), Trust::Unknown);
        app.command(&format!("trust {}", rc.display()));
        assert_eq!(trust_of(&rc), Trust::Allowed);
        std::fs::write(&rc, "set tw=41\n").unwrap();
        assert_eq!(trust_of(&rc), Trust::Unknown, "changed since it was trusted");
        app.command(&format!("trust ++deny {}", rc.display()));
        assert_eq!(trust_of(&rc), Trust::Denied);
        app.command(&format!("trust ++remove {}", rc.display()));
        assert_eq!(trust_of(&rc), Trust::Unknown);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// Neovim's runtime layout in zett's config folder (and the data folder's zett/site): plugin/ at
/// startup, ftplugin/ and indent/ for a file type, each after/ folder last, as Neovim loads them.
impl App {
    /// The runtime folders that are not packages (theirs are sourced by :packadd / :packloadall).
    fn own_rtp(&self) -> Vec<PathBuf> {
        self.rtp().into_iter().filter(|d| !self.packs.contains(d)).collect()
    }

    /// Every .lua / .vim under `dir` of each folder in `bases`, depth-first, in name order.
    fn source_tree(&mut self, bases: &[PathBuf], dir: &str) {
        fn walk(d: &std::path::Path, out: &mut Vec<PathBuf>) {
            let mut es: Vec<PathBuf> = std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path()).collect();
            es.sort();
            for p in es {
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|e| e == "lua" || e == "vim") {
                    out.push(p);
                }
            }
        }
        let mut files = Vec::new();
        for b in bases {
            walk(&b.join(dir), &mut files);
        }
        for f in files {
            if let Err(e) = self.source_file(&f) {
                self.say(e);
            }
        }
    }

    /// At startup, after init.lua: plugin/ of the config, the start packages, then every after/plugin/.
    pub(crate) fn load_runtime_plugins(&mut self) {
        if self.runtime_loaded {
            return;
        }
        self.runtime_loaded = true;
        let own = self.own_rtp();
        self.source_tree(&own, "plugin");
        self.runtime_command("packloadall", "", false);
        let mut after: Vec<PathBuf> = own.iter().map(|d| d.join("after")).collect();
        after.extend(self.packs.iter().map(|d| d.join("after")));
        self.source_tree(&after, "plugin");
    }

    /// A file of type `ft` opened: ftplugin/ and indent/ for it (ft.lua, ft_*.lua, ft/*.lua, and
    /// the .vim ones), from every runtime folder, then from every after/.
    pub(crate) fn load_ftplugins(&mut self, ft: &str) {
        if ft.is_empty() || !ft.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return;
        }
        let bases = self.rtp();
        let afters: Vec<PathBuf> = bases.iter().map(|d| d.join("after")).collect();
        for set in [bases, afters] {
            for kind in ["ftplugin", "indent"] {
                let mut files = Vec::new();
                for b in &set {
                    let d = b.join(kind);
                    for ext in ["vim", "lua"] {
                        let one = d.join(format!("{ft}.{ext}"));
                        if one.is_file() {
                            files.push(one);
                        }
                    }
                    let mut more: Vec<PathBuf> = std::fs::read_dir(&d).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| {
                        let n = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                        p.is_file() && n.starts_with(&format!("{ft}_")) && (n.ends_with(".lua") || n.ends_with(".vim"))
                    }).collect();
                    more.sort();
                    files.extend(more);
                    let mut inner: Vec<PathBuf> = std::fs::read_dir(d.join(ft)).into_iter().flatten().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "lua" || e == "vim")).collect();
                    inner.sort();
                    files.extend(inner);
                }
                for f in files {
                    if let Err(e) = self.source_file(&f) {
                        self.say(e);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod live_theme_tests {
    // tinct.nvim as a plugin: :colorscheme tinct, then a new palette on disk shows without a restart
    // (vim.uv.new_fs_event). Sets XDG_* for itself: run alone (--ignored).
    #[test]
    #[ignore]
    fn tinct_nvim_follows_the_palette_file() {
        let src = std::path::Path::new("/mnt/storage/dev/projects/tinct/tinct.nvim");
        let real = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".local/state/quickshell/user/generated/nvim_colors.lua");
        if !src.is_dir() || !real.is_file() {
            return;
        }
        let dir = std::env::temp_dir().join(format!("zett-tinct-{}", std::process::id()));
        let gdir = dir.join("state/quickshell/user/generated");
        std::fs::create_dir_all(&gdir).unwrap();
        let palette = std::fs::read_to_string(&real).unwrap();
        std::fs::write(gdir.join("nvim_colors.lua"), &palette).unwrap();
        unsafe {
            std::env::set_var("XDG_STATE_HOME", dir.join("state"));
            std::env::set_var("XDG_DATA_HOME", dir.join("data"));
        }
        let mut app = crate::App::new(Vec::new());
        app.command(&format!("lua zett.plug('tinct.nvim', {{ dir = {:?}, opts = {{}} }})", src.display().to_string()));
        app.command("colorscheme tinct");
        let bg = |app: &crate::App| {
            crate::hl::sync(&app.engine);
            crate::hl::hl("Normal").bg
        };
        let before = bg(&app);
        assert!(before.is_some(), "no Normal background after :colorscheme tinct ({:?})", app.msg);
        // a new palette: every colour the same, so whichever role Normal uses, it becomes it
        let mut same = String::new();
        let cs: Vec<char> = palette.chars().collect();
        let mut i = 0;
        while i < cs.len() {
            if cs[i] == '#' && i + 7 <= cs.len() && cs[i + 1..i + 7].iter().all(|c| c.is_ascii_hexdigit()) {
                same.push_str("#102030");
                i += 7;
            } else {
                same.push(cs[i]);
                i += 1;
            }
        }
        std::fs::write(gdir.join("nvim_colors.lua"), same).unwrap();
        let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < end && bg(&app) == before {
            let fx = app.with_lua(|e| e.run_timers());
            app.after_lua(fx);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let after = bg(&app);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(after, Some(ratatui::style::Color::Rgb(0x10, 0x20, 0x30)), "Normal did not follow the palette (was {before:?})");
    }
}

/// The colorschemes Neovim ships (habamax, retrobox, sorbet …).
fn nvim_colors() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = ["/usr/share/nvim/runtime/colors", "/usr/local/share/nvim/runtime/colors"].iter().flat_map(|d| std::fs::read_dir(d).into_iter().flatten().flatten().map(|e| e.path())).filter(|p| p.extension().is_some_and(|e| e == "vim" || e == "lua") && p.file_stem().is_some_and(|s| s != "vim")).collect();
    v.sort();
    v
}

impl App {
    /// Every colorscheme :colorscheme finds: (name, where from), the runtime path's first.
    pub(crate) fn colorschemes(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        let own = self.runtime_files("colors/*.lua").into_iter().chain(self.runtime_files("colors/*.vim"));
        for (p, from) in own.map(|p| (p, "")).chain(nvim_colors().into_iter().map(|p| (p, "Neovim"))) {
            let Some(n) = p.file_stem().map(|s| s.to_string_lossy().to_string()) else { continue };
            if !out.iter().any(|o| o.0 == n) {
                let from = if from.is_empty() { p.parent().and_then(|d| d.parent()).and_then(|d| d.file_name()).map(|s| s.to_string_lossy().to_string()).unwrap_or_default() } else { from.to_string() };
                out.push((n, from));
            }
        }
        out
    }
}

#[cfg(test)]
mod gitsigns_live_tests {
    // gitsigns.nvim (a clone in GITSIGNS_DIR) on a repo with a changed, added and deleted line: the
    // signs it places must be those three kinds, on those lines. Sets XDG_* itself: run alone (--ignored).
    #[test]
    #[ignore]
    fn gitsigns_marks_the_changes() {
        let Ok(src) = std::env::var("GITSIGNS_DIR") else { return };
        let dir = std::env::temp_dir().join(format!("zett-gs-{}", std::process::id()));
        let repo = dir.join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        let git = |args: &[&str]| assert!(std::process::Command::new("git").arg("-C").arg(&repo).args(args).env("GIT_AUTHOR_NAME", "t").env("GIT_AUTHOR_EMAIL", "t@t").env("GIT_COMMITTER_NAME", "t").env("GIT_COMMITTER_EMAIL", "t@t").output().unwrap().status.success());
        git(&["init", "-q"]);
        let lines: Vec<String> = (1..=10).map(|i| format!("line {i}")).collect();
        std::fs::write(repo.join("f.txt"), lines.join("\n") + "\n").unwrap();
        git(&["add", "f.txt"]);
        git(&["commit", "-q", "-m", "one"]);
        // line 2 changed, two added after line 5, line 9 gone
        let mut now = lines.clone();
        now[1] = "line two".into();
        now.insert(5, "new a".into());
        now.insert(6, "new b".into());
        now.remove(10);
        std::fs::write(repo.join("f.txt"), now.join("\n") + "\n").unwrap();
        unsafe {
            std::env::set_var("XDG_DATA_HOME", dir.join("data"));
            std::env::set_var("XDG_STATE_HOME", dir.join("state"));
        }
        let out = dir.join("signs.txt");
        let mut app = crate::App::new(Vec::new());
        app.command(&format!("lua zett.plug('gitsigns.nvim', {{ dir = {:?}, opts = {{ debug_mode = true }} }})", src));
        app.open(&repo.join("f.txt"));
        let probe = format!(
            "lua local t = {{}} for _, m in ipairs(vim.api.nvim_buf_get_extmarks(0, -1, 0, -1, {{ details = true }})) do if m[4].sign_text then t[#t + 1] = (m[2] + 1) .. ':' .. vim.trim(m[4].sign_text) .. ':' .. (m[4].sign_hl_group or '') end end local f = io.open({:?}, 'w') f:write(table.concat(t, ' ')) f:close() local g = io.open({:?} .. '.log', 'w') g:write(vim.inspect(vim.b.gitsigns_status_dict) .. string.char(10) .. vim.inspect(require('gitsigns.debug.log').get(true))) g:close()",
            out.display().to_string(),
            out.display().to_string()
        );
        // frames are drawn as the editor does: the decoration providers (gitsigns' signs) run before each
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(100, 30)).unwrap();
        let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut signs = String::new();
        while std::time::Instant::now() < end {
            let fx = app.with_lua(|e| e.run_timers());
            app.after_lua(fx);
            app.sync_profile();
            let fx = app.with_lua(|e| e.decorate());
            app.after_lua(fx);
            term.draw(|f| app.draw(f)).unwrap();
            app.command(&probe);
            signs = std::fs::read_to_string(&out).unwrap_or_default();
            if signs.split(' ').filter(|s| !s.is_empty()).count() >= 4 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        // and drawn: the sign column of the frame shows them
        let screen: Vec<String> = {
            let b = term.backend().buffer();
            (0..b.area.height).map(|y| (0..b.area.width).map(|x| b[(x, y)].symbol().to_string()).collect::<String>()).collect()
        };
        eprintln!("screen:\n{}", screen.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
        let errors = app.with_lua(|e| std::mem::take(&mut e.errors)).join("\n");
        app.command(&probe);
        eprintln!("probe message: {:?}", app.msg);
        eprintln!("gitsigns log:\n{}", std::fs::read_to_string(out.with_extension("txt.log")).unwrap_or_default());
        let _ = std::fs::remove_dir_all(&dir);
        eprintln!("signs: {signs}\nmessage: {:?}\nerrors: {errors}", app.msg);
        // changed line 2, added lines 6 and 7, a delete marked at line 10 (after "line 8")
        for want in ["2:", "6:", "7:"] {
            assert!(signs.split(' ').any(|s| s.starts_with(want)), "no sign at {want} in {signs:?} ({errors})");
        }
        assert!(signs.contains("Delete") || signs.contains("_"), "no delete sign in {signs:?}");
    }
}
