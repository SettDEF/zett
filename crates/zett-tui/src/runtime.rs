//! Files zett reads as Vim does: :runtime :packadd :packloadall :scriptnames :colorscheme :compiler
//! (Neovim's own compiler files too), :mksession :mkview :loadview, and .vim files as Ex lines.

use super::*;

const NAMES: &[(&str, &str)] = &[
    ("runtime", "ru"), ("packadd", "pa"), ("packloadall", "packl"), ("scriptnames", "scr"), ("colorscheme", "colo"),
    ("compiler", "comp"), ("mksession", "mks"), ("mkview", "mkvie"), ("loadview", "lo"),
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
        let code = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
        if !self.sourced.iter().any(|s| s == p) {
            self.sourced.push(p.to_path_buf());
        }
        if p.extension().is_some_and(|e| e == "vim") {
            for l in vim_lines(&code) {
                self.command(&l);
            }
            return Ok(());
        }
        let fx = self.with_lua(|e| e.eval(&code))?.1;
        self.after_lua(fx);
        Ok(())
    }

    /// A package's folder by name: pack/*/opt/NAME (start/NAME with `start`) under the runtime folders.
    fn package(&self, name: &str, start: bool) -> Vec<PathBuf> {
        let kind = if start { "start" } else { "opt" };
        let mut out = Vec::new();
        for base in self.rtp() {
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
                let file = self.runtime_files(&format!("colors/{arg}.lua")).into_iter().chain(self.runtime_files(&format!("colors/{arg}.vim"))).next();
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
        app.command("compiler mycc");
        assert_eq!(app.setting(app.cur, "makeprg"), "mycc -x");
        app.command("scriptnames");
        let rows = match &app.popup { Some(crate::Popup::Help(_, r)) => r.clone(), _ => Vec::new() };
        assert!(rows.iter().any(|(_, f)| f.ends_with("mine.vim")) && rows.iter().any(|(_, f)| f.ends_with("foo.vim")), "{rows:?}");
        let _ = std::fs::remove_dir_all(&base);
    }
}
