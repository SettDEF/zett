//! The current folder in Vim's three scopes: :cd :chdir (global), :tcd :tchdir (the tab: zett's
//! tabs are its buffers), :lcd :lchdir (the window); :pwd, with 'verbose' the scope ('cdpath', 'cdhome').

use super::*;

const NAMES: &[(&str, &str)] = &[("cd", "cd"), ("chdir", "chd"), ("tcd", "tc"), ("tchdir", "tch"), ("lcd", "lc"), ("lchdir", "lch")];

/// A typed name as one of these commands (its full name).
pub(crate) fn dir_name(typed: &str) -> Option<&'static str> {
    NAMES.iter().find(|(full, short)| typed.len() >= short.len() && full.starts_with(typed)).map(|(f, _)| *f)
}

/// ~ at the start as the home folder.
fn expand_home(s: &str) -> String {
    s.strip_prefix('~').map(|r| format!("{}{r}", std::env::var("HOME").unwrap_or_default())).unwrap_or_else(|| s.to_string())
}

/// Which scope set the folder that applies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Scope {
    Global,
    Tab,
    Window,
}

impl App {
    /// The folder that applies here: the window's (:lcd), else the tab's (:tcd), else the global one.
    pub(crate) fn dir_here(&self) -> (PathBuf, Scope) {
        if let Some(d) = self.wins.get(self.focus).and_then(|w| w.as_ref()).and_then(|w| w.lcd.clone()) {
            return (d, Scope::Window);
        }
        if let Some(d) = self.docs.get(self.cur).and_then(|d| d.tcd.clone()) {
            return (d, Scope::Tab);
        }
        (self.global_dir.clone(), Scope::Global)
    }

    /// Before each frame: the project folder and the process's folder (:! runs there) follow the
    /// scope that applies. Not under 'autochdir', which moves the process's folder itself.
    pub(crate) fn sync_dir(&mut self) {
        let (d, _) = self.dir_here();
        if d != self.root && !self.flag(self.cur, "autochdir") {
            let _ = std::env::set_current_dir(&d);
            self.root = d;
        }
    }

    /// The global folder (:cd, a project picked): the window and the tab lose their own, as Vim's :cd.
    pub(crate) fn set_global_dir(&mut self, d: PathBuf) {
        let old = std::mem::replace(&mut self.global_dir, d);
        self.prev_dirs[0] = Some(old);
        if let Some(Some(w)) = self.wins.get_mut(self.focus) {
            w.lcd = None;
        }
        if let Some(doc) = self.docs.get_mut(self.cur) {
            doc.tcd = None;
        }
        self.sync_dir();
    }

    /// :pwd; with 'verbose' the scope it comes from, as Neovim prints it.
    pub(crate) fn pwd(&mut self) {
        let (d, scope) = self.dir_here();
        let tag = match scope {
            Scope::Global => "[global] ",
            Scope::Tab => "[tabpage] ",
            Scope::Window => "[window] ",
        };
        let verbose = self.num(self.cur, "verbose") > 0;
        self.say(format!("{}{}", if verbose { tag } else { "" }, d.display()));
    }

    /// The folder a :cd argument names: ~, -, %:h, relative ones through 'cdpath'.
    fn dir_target(&self, arg: &str, prev: Option<&PathBuf>) -> Result<PathBuf, String> {
        let arg = arg.trim();
        if arg.is_empty() {
            // 'cdhome' off: Neovim prints the folder there only on non-Unix systems; home here
            return std::env::var("HOME").map(PathBuf::from).map_err(|_| "E472: Command failed".to_string());
        }
        if arg == "-" {
            return prev.cloned().ok_or_else(|| "E186: No previous directory".to_string());
        }
        let arg = match arg {
            "%:h" | "%:p:h" => self.docs[self.cur].abs_path().and_then(|p| p.parent().map(Path::to_path_buf)).map(|p| p.display().to_string()).unwrap_or_default(),
            a => expand_home(a),
        };
        let p = PathBuf::from(&arg);
        let found = if p.is_absolute() || arg.starts_with("./") || arg.starts_with("../") {
            Some(self.root.join(&p)).filter(|d| d.is_dir())
        } else {
            // 'cdpath': its folders in order, "" (,,) for the current one
            let cdpath = self.setting(self.cur, "cdpath");
            cdpath.split(',').map(|base| if base.is_empty() { self.root.clone() } else { PathBuf::from(expand_home(base)) }).map(|b| b.join(&p)).find(|d| d.is_dir())
        };
        found.map(|d| std::fs::canonicalize(&d).unwrap_or(d)).ok_or_else(|| format!("E344: Can't find directory \"{arg}\" in cdpath"))
    }

    /// One of these commands.
    pub(crate) fn dir_command(&mut self, name: &str, arg: &str) {
        let Some(cmd) = dir_name(name) else { return };
        let slot = match cmd {
            "cd" | "chdir" => 0,
            "tcd" | "tchdir" => 1,
            _ => 2,
        };
        let target = match self.dir_target(arg, self.prev_dirs[slot].as_ref()) {
            Ok(d) => d,
            Err(e) => return self.say(e),
        };
        let (now, _) = self.dir_here();
        match slot {
            0 => self.set_global_dir(target),
            1 => {
                // the tab's; the window loses its own
                if let Some(Some(w)) = self.wins.get_mut(self.focus) {
                    w.lcd = None;
                }
                self.docs[self.cur].tcd = Some(target);
                self.prev_dirs[1] = Some(now);
            }
            _ => {
                if let Some(Some(w)) = self.wins.get_mut(self.focus) {
                    w.lcd = Some(target);
                }
                self.prev_dirs[2] = Some(now);
            }
        }
        self.sync_dir();
    }
}

#[cfg(test)]
mod tests {
    use super::Scope;

    fn app() -> crate::App {
        crate::App::new(Vec::new())
    }

    #[test]
    fn names() {
        assert_eq!(super::dir_name("cd"), Some("cd"));
        assert_eq!(super::dir_name("chd"), Some("chdir"));
        assert_eq!(super::dir_name("tc"), Some("tcd"));
        assert_eq!(super::dir_name("lch"), Some("lchdir"));
        assert_eq!(super::dir_name("c"), None);
        assert_eq!(super::dir_name("ch"), None);
    }

    #[test]
    fn three_scopes_as_vim_has_them() {
        let keep = std::env::current_dir().unwrap();
        let base = std::env::temp_dir().join(format!("zett-dirs-{}", std::process::id()));
        let (a, b, c) = (base.join("a"), base.join("b"), base.join("c"));
        for d in [&a, &b, &c] {
            std::fs::create_dir_all(d).unwrap();
        }
        let (a, b, c) = (a.canonicalize().unwrap(), b.canonicalize().unwrap(), c.canonicalize().unwrap());
        let mut app = app();
        app.command(&format!("cd {}", a.display()));
        assert_eq!(app.dir_here(), (a.clone(), Scope::Global));
        assert_eq!(app.root, a, "root follows");
        app.command(&format!("tcd {}", b.display()));
        assert_eq!(app.dir_here(), (b.clone(), Scope::Tab));
        app.command(&format!("lcd {}", c.display()));
        assert_eq!(app.dir_here(), (c.clone(), Scope::Window));
        app.command("lcd -");
        assert_eq!(app.dir_here(), (b.clone(), Scope::Window), "back to what applied before");
        // :tcd clears the window's own folder; :cd clears both
        app.command(&format!("tcd {}", c.display()));
        assert_eq!(app.dir_here(), (c.clone(), Scope::Tab));
        app.command(&format!("cd {}", b.display()));
        assert_eq!(app.dir_here(), (b.clone(), Scope::Global));
        app.command("cd -");
        assert_eq!(app.dir_here(), (a.clone(), Scope::Global));
        // 'cdpath': a relative folder searched in its folders
        app.command(&format!("set cdpath=,{}", base.display()));
        app.command("cd c");
        assert_eq!(app.dir_here().0, c, "found through 'cdpath'");
        app.command("cd nothere");
        assert!(app.dir_here().0 == c, "a folder not found changes nothing");
        let _ = std::env::set_current_dir(&keep);
        let _ = std::fs::remove_dir_all(&base);
    }
}
