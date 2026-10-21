//! A git buffer as Magit's status (C-x g / SPC g g): the branch and where it stands, unstaged,
//! staged and untracked files, recent commits. On a file: s stages, u unstages, x discards (twice),
//! Tab shows its diff inline, Enter opens it; on a heading, s / u take the whole group; c commits
//! (the message in the echo area), g refreshes, q closes. And the history of one block (C-c g b /
//! SPC g b): the commits that changed the section at the cursor, Enter shows one.

use super::*;
use zett_edit::outline;

pub(crate) const NAME: &str = "*git*";

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Group {
    Unstaged,
    Staged,
    Untracked,
}

/// What a line of the buffer is about.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Row {
    Head(Group),
    File(Group, String),
    Commit(String),
    Other,
}

#[derive(Default)]
pub(crate) struct GitView {
    pub root: PathBuf,
    pub rows: Vec<Row>,
    /// files whose diff is shown (group, path)
    pub open: Vec<(Group, String)>,
    /// x pressed once on this file: the second press discards
    pub armed: Option<String>,
}

pub(crate) fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let o = std::process::Command::new("git").args(args).current_dir(root).stdin(std::process::Stdio::null()).output().map_err(|e| format!("git: {e}"))?;
    if o.status.success() {
        Ok(String::from_utf8_lossy(&o.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&o.stderr).lines().find(|l| !l.trim().is_empty()).unwrap_or("git failed").trim().to_string())
    }
}

/// `git status --porcelain=v1` → (unstaged, staged, untracked), each (what, path).
pub(crate) fn parse_status(s: &str) -> (Vec<(String, String)>, Vec<(String, String)>, Vec<String>) {
    let word = |c: char| match c { 'M' => "modified", 'A' => "new file", 'D' => "deleted", 'R' => "renamed", 'C' => "copied", 'T' => "typechange", 'U' => "unmerged", _ => "changed" }.to_string();
    let (mut un, mut st, mut ut) = (Vec::new(), Vec::new(), Vec::new());
    for l in s.lines().filter(|l| l.len() > 3) {
        let (x, y) = (l.chars().next().unwrap_or(' '), l.chars().nth(1).unwrap_or(' '));
        let path = l[3..].rsplit(" -> ").next().unwrap_or(&l[3..]).to_string();
        if x == '?' {
            ut.push(path);
            continue;
        }
        if x != ' ' {
            st.push((word(x), path.clone()));
        }
        if y != ' ' {
            un.push((word(y), path));
        }
    }
    (un, st, ut)
}

impl App {
    pub(crate) fn git_root(&self) -> Option<PathBuf> {
        let here = std::fs::canonicalize(&self.docs[self.cur].name).ok().unwrap_or(self.root.clone());
        doom::project_of(&here).or_else(|| doom::project_of(&self.root))
    }

    /// C-x g: the status buffer of the project.
    pub(crate) fn git_status(&mut self) {
        let Some(root) = self.git_root() else { return self.say("not in a git repository") };
        let keep = self.git.take().filter(|g| g.root == root).map(|g| g.open).unwrap_or_default();
        self.git = Some(GitView { root, open: keep, ..Default::default() });
        self.git_render();
    }

    /// Builds the buffer's text from git, keeping the cursor's line.
    fn git_render(&mut self) {
        let Some(g) = self.git.as_mut() else { return };
        let root = g.root.clone();
        let (mut text, mut rows) = (Vec::<String>::new(), Vec::<Row>::new());
        let line = |t: String, r: Row, text: &mut Vec<String>, rows: &mut Vec<Row>| {
            text.push(t);
            rows.push(r);
        };
        let head = git(&root, &["status", "-sb", "--porcelain=v1"]).unwrap_or_default();
        let branch = head.lines().next().unwrap_or("").trim_start_matches("## ").to_string();
        let subject = git(&root, &["log", "-1", "--format=%h %s"]).unwrap_or_default();
        line(format!("Head:     {}", branch.split("...").next().unwrap_or(&branch)), Row::Other, &mut text, &mut rows);
        if let Some((_, up)) = branch.split_once("...") {
            line(format!("Upstream: {up}"), Row::Other, &mut text, &mut rows);
        }
        line(format!("Last:     {}", subject.trim()), Row::Other, &mut text, &mut rows);
        let (un, st, ut) = parse_status(&head.lines().skip(1).collect::<Vec<_>>().join("\n"));
        let groups = [(Group::Unstaged, "Unstaged changes", un), (Group::Staged, "Staged changes", st), (Group::Untracked, "Untracked files", ut.into_iter().map(|p| ("".to_string(), p)).collect())];
        for (grp, title, files) in groups {
            if files.is_empty() {
                continue;
            }
            line(String::new(), Row::Other, &mut text, &mut rows);
            line(format!("{title} ({})", files.len()), Row::Head(grp.clone()), &mut text, &mut rows);
            for (what, path) in files {
                let shown = if what.is_empty() { format!("  {path}") } else { format!("  {what:<11}{path}") };
                line(shown, Row::File(grp.clone(), path.clone()), &mut text, &mut rows);
                if g.open.contains(&(grp.clone(), path.clone())) {
                    let diff = match grp {
                        Group::Staged => git(&root, &["diff", "--cached", "--", &path]),
                        Group::Unstaged => git(&root, &["diff", "--", &path]),
                        Group::Untracked => std::fs::read_to_string(root.join(&path)).map(|t| t.lines().take(40).map(|l| format!("+{l}\n")).collect()).map_err(|e| e.to_string()),
                    }
                    .unwrap_or_else(|e| e);
                    // from the first hunk (no diff --git / index / --- / +++ header); an untracked file is all + lines
                    let from_hunk = grp == Group::Untracked;
                    for d in diff.lines().skip_while(|l| !from_hunk && !l.starts_with("@@")).take(200) {
                        line(format!("    {d}"), Row::File(grp.clone(), path.clone()), &mut text, &mut rows);
                    }
                }
            }
        }
        let log = git(&root, &["log", "-10", "--format=%h %s (%ar)"]).unwrap_or_default();
        if !log.trim().is_empty() {
            line(String::new(), Row::Other, &mut text, &mut rows);
            line("Recent commits".into(), Row::Other, &mut text, &mut rows);
            for l in log.lines() {
                line(format!("  {l}"), Row::Commit(l.split_whitespace().next().unwrap_or("").to_string()), &mut text, &mut rows);
            }
        }
        line(String::new(), Row::Other, &mut text, &mut rows);
        line("s stage · u unstage · x discard · Tab diff · Enter open · c commit · g refresh · q close".into(), Row::Other, &mut text, &mut rows);
        g.rows = rows;
        let body = text.join("\n") + "\n";
        let at = self.docs.iter().position(|d| d.name == NAME);
        let keep = at.map(|i| self.docs[i].ed.cursor()).unwrap_or(Pos { line: 0, col: 0 });
        let mut ed = Editor::from_str(&body);
        ed.set_emacs(self.emacs_keys());
        let d = Doc::new(ed, NAME.into(), "Diff".into());
        let i = match at {
            Some(i) => {
                self.docs[i] = d;
                i
            }
            None => {
                self.docs.push(d);
                self.docs.len() - 1
            }
        };
        self.show_doc(i);
        let last = self.docs[i].ed.buf.len_lines().saturating_sub(1);
        self.docs[i].ed.set_cursor(Pos { line: keep.line.min(last), col: 0 });
    }

    /// The git buffer's own keys; false for any other buffer or key.
    pub(crate) fn git_key(&mut self, k: &KeyEvent) -> bool {
        if self.docs[self.cur].name != NAME || !self.keyseq.is_empty() || k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) {
            return false;
        }
        let Some(g) = self.git.as_ref() else { return false };
        let line = self.docs[self.cur].ed.cursor().line;
        let row = g.rows.get(line).cloned().unwrap_or(Row::Other);
        let root = g.root.clone();
        let say_err = |a: &mut App, r: Result<String, String>| if let Err(e) = r { a.say(e) };
        match (k.code, &row) {
            (KeyCode::Char('q'), _) => {
                if let Some(i) = self.docs.iter().position(|d| d.name == NAME) {
                    self.docs.remove(i);
                    if self.docs.is_empty() {
                        self.docs.push(Doc::new(Editor::from_str(""), "[scratch]".into(), "Plain Text".into()));
                    }
                    self.cur = self.cur.min(self.docs.len() - 1);
                    let c = self.cur;
                    self.show_doc(c);
                }
                self.git = None;
            }
            (KeyCode::Char('g'), _) => self.git_render(),
            (KeyCode::Char('n'), _) | (KeyCode::Char('j'), _) => self.doc().ed.set_cursor(Pos { line: line + 1, col: 0 }),
            (KeyCode::Char('p'), _) | (KeyCode::Char('k'), _) => self.doc().ed.set_cursor(Pos { line: line.saturating_sub(1), col: 0 }),
            (KeyCode::Char('s'), Row::File(grp, p)) if *grp != Group::Staged => {
                let r = git(&root, &["add", "--", p]);
                say_err(self, r);
                self.git_render();
            }
            (KeyCode::Char('s'), Row::Head(Group::Unstaged)) => {
                let r = git(&root, &["add", "-u"]);
                say_err(self, r);
                self.git_render();
            }
            (KeyCode::Char('s'), Row::Head(Group::Untracked)) => {
                let r = git(&root, &["add", "-A", "--", "."]);
                say_err(self, r);
                self.git_render();
            }
            (KeyCode::Char('u'), Row::File(Group::Staged, p)) => {
                let r = git(&root, &["restore", "--staged", "--", p]);
                say_err(self, r);
                self.git_render();
            }
            (KeyCode::Char('u'), Row::Head(Group::Staged)) => {
                let r = git(&root, &["restore", "--staged", "--", "."]);
                say_err(self, r);
                self.git_render();
            }
            (KeyCode::Char('x'), Row::File(grp, p)) if *grp != Group::Staged => {
                if self.git.as_ref().and_then(|g| g.armed.as_ref()) != Some(p) {
                    if let Some(g) = self.git.as_mut() {
                        g.armed = Some(p.clone());
                    }
                    self.say(format!("x again throws away the changes to {p}"));
                } else {
                    let r = if *grp == Group::Untracked { std::fs::remove_file(root.join(p)).map(|_| String::new()).map_err(|e| e.to_string()) } else { git(&root, &["restore", "--", p]) };
                    say_err(self, r);
                    if let Some(g) = self.git.as_mut() {
                        g.armed = None;
                    }
                    self.git_render();
                }
            }
            (KeyCode::Tab, Row::File(grp, p)) => {
                if let Some(g) = self.git.as_mut() {
                    let key = (grp.clone(), p.clone());
                    if let Some(i) = g.open.iter().position(|x| *x == key) {
                        g.open.remove(i);
                    } else {
                        g.open.push(key);
                    }
                }
                self.git_render();
            }
            (KeyCode::Enter, Row::File(_, p)) => self.open(&root.join(p)),
            (KeyCode::Enter, Row::Commit(h)) => {
                let h = h.clone();
                self.git_show(&root, &h);
            }
            (KeyCode::Char('c'), _) => {
                self.prompt = Some("Commit message: ".into());
                self.committing = true;
                self.cmdline = Some(String::new());
            }
            // the buffer is git's: other plain keys do nothing here (the arrows still move)
            (KeyCode::Char(_), _) => {}
            _ => return false,
        }
        true
    }

    /// The commit, with the message from the echo area.
    pub(crate) fn git_commit(&mut self, msg: &str) {
        let Some(root) = self.git.as_ref().map(|g| g.root.clone()) else { return };
        if msg.trim().is_empty() {
            return self.say("no message: nothing committed");
        }
        match git(&root, &["commit", "-m", msg.trim()]) {
            Ok(out) => self.say(out.lines().next().unwrap_or("committed").to_string()),
            Err(e) => self.say(e),
        }
        self.git_render();
    }

    /// One commit in a buffer of its own.
    pub(crate) fn git_show(&mut self, root: &Path, hash: &str) {
        let text = git(root, &["show", "--stat", "--patch", hash]).unwrap_or_else(|e| e);
        let name = format!("*git {hash}*");
        if let Some(i) = self.docs.iter().position(|d| d.name == name) {
            return self.show_doc(i);
        }
        self.docs.push(Doc::new(Editor::from_str(&text), name, "Diff".into()));
        let i = self.docs.len() - 1;
        self.show_doc(i);
    }

    /// C-c g b: the commits that changed the section at the cursor (git log -L), newest first.
    pub(crate) fn git_blame_section(&mut self) {
        let Some(root) = self.git_root() else { return self.say("not in a git repository") };
        let d = &mut self.docs[self.cur];
        let Ok(file) = std::fs::canonicalize(&d.name) else { return self.say("this buffer is not a file") };
        let secs = d.ed.sections();
        let line = d.ed.cursor().line;
        let (a, b, title) = match outline::at(&secs, line) {
            Some(i) => (secs[i].start + 1, secs[i].end, secs[i].title.clone()),
            None => (line + 1, line + 1, format!("line {}", line + 1)),
        };
        let rel = file.strip_prefix(&root).unwrap_or(&file).display().to_string();
        match git(&root, &["log", "--no-patch", "--format=%h %an · %ar · %s", "-L", &format!("{a},{b}:{rel}")]) {
            Err(e) => self.say(e),
            Ok(out) => {
                let rows: Vec<crate::minibuf::Cand> = out.lines().filter(|l| !l.trim().is_empty()).map(crate::minibuf::Cand::plain).collect();
                if rows.is_empty() {
                    return self.say(format!("{title}: not committed yet"));
                }
                let t = title.trim_start_matches('#').trim().chars().take(40).collect::<String>();
                self.git = Some(GitView { root, ..Default::default() });
                self.pick_list(crate::doom::Pick::Commits, &format!("{} commits changed {t}", rows.len()), rows);
            }
        }
    }

    /// Enter on a commit in that list.
    pub(crate) fn picked_commit(&mut self, item: &str) {
        let root = self.git.as_ref().map(|g| g.root.clone()).or_else(|| self.git_root()).unwrap_or(self.root.clone());
        if let Some(h) = item.split_whitespace().next() {
            self.git_show(&root, h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_porcelain_status() {
        let (un, st, ut) = parse_status(" M src/a.rs\nM  src/b.rs\nMM src/c.rs\nA  new.rs\nR  old.rs -> moved.rs\n?? notes.md\n D gone.rs\n");
        assert_eq!(un, vec![("modified".into(), "src/a.rs".into()), ("modified".into(), "src/c.rs".into()), ("deleted".into(), "gone.rs".into())]);
        assert_eq!(st, vec![("modified".into(), "src/b.rs".into()), ("modified".into(), "src/c.rs".into()), ("new file".into(), "new.rs".into()), ("renamed".into(), "moved.rs".into())]);
        assert_eq!(ut, vec!["notes.md".to_string()]);
    }
}
