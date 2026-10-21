//! Emacs's VC (C-x v) on git: diffs and logs of the file or the whole tree, annotate (blame), the
//! next action (add, then commit), revert, branches, tags, merges, worktrees, ignore, rename and
//! delete. Logs are the commit picker (Enter shows one); diffs and blame are buffers of their own;
//! pull and push run in a terminal pane (zeit), so the network never holds the editor.

use super::*;
use crate::git::git;

impl App {
    /// The buffer's file and the repository: (root, path relative to it).
    fn vc_file(&self) -> Option<(PathBuf, String)> {
        let root = self.git_root()?;
        let file = std::fs::canonicalize(self.docs[self.cur].abs_path()?).ok()?;
        let rel = file.strip_prefix(&root).ok()?.display().to_string();
        Some((root, rel))
    }

    /// Output in a buffer named `name` (made again when it is there), with `syntax`.
    pub(crate) fn vc_buffer(&mut self, name: &str, text: String, syntax: &str) {
        let text = if text.trim().is_empty() { "(nothing)\n".to_string() } else { text };
        match self.docs.iter().position(|d| d.name == name) {
            Some(i) => {
                self.docs[i].ed = Editor::from_str(&text);
                self.show_doc(i);
            }
            None => {
                self.docs.push(Doc::new(Editor::from_str(&text), name.to_string(), syntax.into()));
                let i = self.docs.len() - 1;
                self.show_doc(i);
            }
        }
    }

    /// Commits to pick from (Enter shows one): `git log` with these arguments.
    fn vc_log(&mut self, root: &Path, args: &[&str], title: &str) {
        let mut a = vec!["log", "--no-patch", "--format=%h %an · %ar · %s", "-200"];
        a.extend_from_slice(args);
        match git(root, &a) {
            Err(e) => self.say(e),
            Ok(out) => {
                let rows: Vec<Cand> = out.lines().filter(|l| !l.trim().is_empty()).map(Cand::plain).collect();
                if rows.is_empty() {
                    return self.say(format!("{title}: no commits"));
                }
                self.git = Some(crate::git::GitView { root: root.to_path_buf(), ..Default::default() });
                self.pick_list(Pick::Commits, &format!("{title} ({})", rows.len()), rows);
            }
        }
    }

    /// After git changed files: the buffers read them again, the git buffer redraws.
    fn vc_changed(&mut self, msg: String) {
        self.checktime();
        if self.docs.iter().any(|d| d.name == crate::git::NAME) {
            self.git_status();
        }
        self.say(msg);
    }

    fn vc_run(&mut self, root: &Path, args: &[&str], ok: &str) {
        match git(root, args) {
            Ok(out) => {
                let first = out.lines().find(|l| !l.trim().is_empty()).map(str::to_string);
                self.vc_changed(first.unwrap_or_else(|| ok.to_string()));
            }
            Err(e) => self.say(e),
        }
    }

    /// One VC command by name ("diff", "log", …); `arg` is what its prompt asked for.
    pub(crate) fn vc(&mut self, what: &str, arg: &str) {
        let Some(root) = self.git_root() else { return self.say("not in a git repository") };
        let file = self.vc_file().map(|f| f.1);
        let need_file = || file.clone().ok_or("this buffer is not a file in the repository".to_string());
        let arg = arg.trim();
        match what {
            "diff" | "diff-outgoing" | "root-diff" | "root-diff-outgoing" | "diff-mergebase" | "root-diff-mergebase" => {
                let whole = what.starts_with("root");
                let base = match what {
                    w if w.ends_with("outgoing") => "@{upstream}".to_string(),
                    w if w.ends_with("mergebase") => match git(&root, &["merge-base", "HEAD", arg]) {
                        Ok(b) => b.trim().to_string(),
                        Err(e) => return self.say(e),
                    },
                    _ => "HEAD".to_string(),
                };
                let mut a = vec!["diff", base.as_str()];
                let f;
                if !whole {
                    f = match need_file() {
                        Ok(f) => f,
                        Err(e) => return self.say(e),
                    };
                    a.extend(["--", f.as_str()]);
                }
                match git(&root, &a) {
                    Ok(out) => self.vc_buffer("*vc-diff*", out, "Diff"),
                    Err(e) => self.say(e),
                }
            }
            "log" | "root-log" | "log-outgoing" | "log-incoming" | "branch-log" | "root-branch-log" | "log-mergebase" => {
                let range = match what {
                    "log-outgoing" => "@{upstream}..HEAD".to_string(),
                    "log-incoming" => "HEAD..@{upstream}".to_string(),
                    "branch-log" | "root-branch-log" => arg.to_string(),
                    "log-mergebase" => format!("{arg}...HEAD"),
                    _ => "HEAD".to_string(),
                };
                let f = file.clone().filter(|_| matches!(what, "log" | "branch-log"));
                let mut a = vec![range.as_str()];
                if let Some(f) = &f {
                    a.extend(["--", f.as_str()]);
                }
                let title = match (&f, what) {
                    (Some(f), _) => format!("Commits of {f}"),
                    (None, "log-outgoing") => "Commits not pushed".into(),
                    (None, "log-incoming") => "Commits not pulled (as of the last fetch)".into(),
                    _ => format!("Commits of {range}"),
                };
                self.vc_log(&root, &a, &title);
            }
            "annotate" => {
                let f = match need_file() {
                    Ok(f) => f,
                    Err(e) => return self.say(e),
                };
                match git(&root, &["blame", "--date=short", "--", &f]) {
                    Ok(out) => self.vc_buffer(&format!("*Annotate {f}*"), out, "Plain Text"),
                    Err(e) => self.say(e),
                }
            }
            "revision" => {
                let f = match need_file() {
                    Ok(f) => f,
                    Err(e) => return self.say(e),
                };
                let rev = if arg.is_empty() { "HEAD" } else { arg };
                let syntax = self.docs[self.cur].syntax.clone();
                match git(&root, &["show", &format!("{rev}:{f}")]) {
                    Ok(out) => {
                        self.doom_ui("split", "right");
                        self.vc_buffer(&format!("{f}.~{rev}~"), out, &syntax);
                    }
                    Err(e) => self.say(e),
                }
            }
            // C-x v v: an untracked file is added; a changed one asks for the message and is committed
            "next-action" => {
                let f = match need_file() {
                    Ok(f) => f,
                    Err(e) => return self.say(e),
                };
                let st = git(&root, &["status", "--porcelain=v1", "--", &f]).unwrap_or_default();
                if st.starts_with("??") {
                    self.vc_run(&root, &["add", "--", &f], &format!("Registered {f}"));
                } else if st.trim().is_empty() {
                    self.say(format!("{f} is up to date"));
                } else {
                    self.prompt(&format!("Commit {f}: \tvc:commit {{1}}"));
                }
            }
            "commit" => {
                let f = match need_file() {
                    Ok(f) => f,
                    Err(e) => return self.say(e),
                };
                if arg.is_empty() {
                    return self.say("no message: nothing committed");
                }
                // unsaved changes go in too (Emacs asks; here they are saved first)
                if self.docs[self.cur].dirty() {
                    self.save(None);
                }
                self.vc_run(&root, &["commit", "-m", arg, "--", &f], "committed");
            }
            "register" => match need_file() {
                Ok(f) => self.vc_run(&root, &["add", "--", &f], &format!("Registered {f}")),
                Err(e) => self.say(e),
            },
            "revert" => {
                let f = match need_file() {
                    Ok(f) => f,
                    Err(e) => return self.say(e),
                };
                if arg != "yes" {
                    return self.prompt(&format!("Discard the changes in {f}? (yes or no) \tvc:revert {{1}}"));
                }
                self.vc_run(&root, &["restore", "--", &f], &format!("Reverted {f}"));
            }
            "delete" => {
                let f = match need_file() {
                    Ok(f) => f,
                    Err(e) => return self.say(e),
                };
                if arg != "yes" {
                    return self.prompt(&format!("Delete {f} from the repository and the disk? (yes or no) \tvc:delete {{1}}"));
                }
                match git(&root, &["rm", "-f", "--", &f]) {
                    Ok(_) => {
                        self.doom_ui("killBuffer", "");
                        self.vc_changed(format!("Deleted {f}"));
                    }
                    Err(e) => self.say(e),
                }
            }
            "rename" => {
                let f = match need_file() {
                    Ok(f) => f,
                    Err(e) => return self.say(e),
                };
                if arg.is_empty() {
                    return self.prompt(&format!("Rename {f} to: \tvc:rename {{1}}"));
                }
                match git(&root, &["mv", "--", &f, arg]) {
                    Ok(_) => {
                        let new = root.join(arg);
                        self.command(&format!("file {}", new.display()));
                        self.vc_changed(format!("Renamed {f} to {arg}"));
                    }
                    Err(e) => self.say(e),
                }
            }
            "ignore" => {
                let pat = if arg.is_empty() { file.clone().unwrap_or_default() } else { arg.to_string() };
                if pat.is_empty() {
                    return self.say("nothing to ignore");
                }
                let gi = root.join(".gitignore");
                let mut text = std::fs::read_to_string(&gi).unwrap_or_default();
                if text.lines().any(|l| l.trim() == pat) {
                    return self.say(format!("{pat} is ignored already"));
                }
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push('\n');
                }
                text.push_str(&format!("{pat}\n"));
                match std::fs::write(&gi, text) {
                    Ok(()) => self.vc_changed(format!("Ignored {pat}")),
                    Err(e) => self.say(format!(".gitignore: {e}")),
                }
            }
            "create-branch" => self.vc_run(&root, &["switch", "-c", arg], &format!("Switched to a new branch {arg}")),
            "switch-branch" => self.vc_run(&root, &["switch", arg], &format!("Switched to {arg}")),
            "merge" => self.vc_run(&root, &["merge", "--no-edit", arg], &format!("Merged {arg}")),
            "create-tag" => self.vc_run(&root, &["tag", arg], &format!("Tagged {arg}")),
            "retrieve-tag" => self.vc_run(&root, &["checkout", arg], &format!("At {arg}")),
            "update" => self.command("terminal git pull"),
            "push" => self.command("terminal git push"),
            "add-worktree" => {
                // "DIR BRANCH": a new working tree for a branch (made when it does not exist)
                let mut p = arg.split_whitespace();
                let (Some(dir), branch) = (p.next(), p.next()) else { return self.say("needs a folder") };
                let r = match branch {
                    Some(b) if git(&root, &["rev-parse", "--verify", "--quiet", b]).is_ok() => git(&root, &["worktree", "add", dir, b]),
                    Some(b) => git(&root, &["worktree", "add", "-b", b, dir]),
                    None => git(&root, &["worktree", "add", dir]),
                };
                match r {
                    Ok(_) => self.say(format!("Working tree in {dir}")),
                    Err(e) => self.say(e),
                }
            }
            "switch-worktree" => {
                let p = PathBuf::from(arg);
                if !p.is_dir() {
                    return self.say(format!("no working tree at {arg}"));
                }
                self.set_global_dir(p);
                self.doom_ui("findFile", "");
            }
            "delete-worktree" => self.vc_run(&root, &["worktree", "remove", arg], &format!("Removed the working tree {arg}")),
            "move-worktree" => {
                let mut p = arg.split_whitespace();
                let (Some(a), Some(b)) = (p.next(), p.next()) else { return self.say("needs the working tree and where it goes") };
                self.vc_run(&root, &["worktree", "move", a, b], &format!("Moved {a} to {b}"));
            }
            other => self.say(format!("vc: no {other}")),
        }
    }

    /// Branches, tags, worktrees: candidates for VC prompts.
    pub(crate) fn vc_items(&self, what: &str) -> Vec<Cand> {
        let Some(root) = self.git_root() else { return Vec::new() };
        let out = match what {
            "branches" => git(&root, &["branch", "--all", "--format=%(refname:short)"]),
            "tags" => git(&root, &["tag", "--sort=-creatordate"]),
            _ => git(&root, &["worktree", "list", "--porcelain"]).map(|t| t.lines().filter_map(|l| l.strip_prefix("worktree ")).collect::<Vec<_>>().join("\n")),
        };
        out.unwrap_or_default().lines().filter(|l| !l.trim().is_empty()).map(Cand::plain).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A repository in a temporary folder, one file committed.
    /// A folder that goes when the test ends.
    struct Tmp(PathBuf);
    impl Tmp {
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn repo() -> (Tmp, App) {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let d = Tmp(std::env::temp_dir().join(format!("zett-vc-{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed))));
        std::fs::create_dir_all(d.path()).unwrap();
        let run = |a: &[&str]| assert!(std::process::Command::new("git").args(a).current_dir(d.path()).output().unwrap().status.success(), "{a:?}");
        run(&["init", "-q", "-b", "main"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        std::fs::write(d.path().join("a.txt"), "one\n").unwrap();
        run(&["add", "a.txt"]);
        run(&["commit", "-qm", "first"]);
        let mut app = App::new(Vec::new());
        app.root = d.path().to_path_buf();
        app.open(&d.path().join("a.txt"));
        (d, app)
    }

    #[test]
    fn diff_next_action_and_log() {
        let (d, mut a) = repo();
        std::fs::write(d.path().join("a.txt"), "one\ntwo\n").unwrap();
        a.vc("diff", "");
        assert!(a.docs[a.cur].ed.text().contains("+two"), "{}", a.docs[a.cur].ed.text());
        a.open(&d.path().join("a.txt"));
        a.checktime();
        a.vc("commit", "second");
        assert_eq!(git(d.path(), &["log", "-1", "--format=%s"]).unwrap().trim(), "second");
        a.vc("root-log", "");
        assert_eq!(a.minibuf.as_ref().map(|m| m.count()), Some(2), "both commits to pick from");
    }

    #[test]
    fn branches_tags_ignore() {
        let (d, mut a) = repo();
        a.vc("create-branch", "feature");
        assert_eq!(git(d.path(), &["branch", "--show-current"]).unwrap().trim(), "feature");
        assert!(a.vc_items("branches").iter().any(|c| c.text == "main"));
        a.vc("create-tag", "v1");
        assert!(a.vc_items("tags").iter().any(|c| c.text == "v1"));
        a.vc("ignore", "*.log");
        assert!(std::fs::read_to_string(d.path().join(".gitignore")).unwrap().contains("*.log"));
    }
}
