//! ChangeLog files (C-x 4 a, C-x v a): GNU's dated entries, newest on top.

use super::*;

/// Today's heading: "2026-10-23  Name  <mail>" (git's user, else $USER).
fn heading(dir: &Path) -> String {
    let git = |k: &str| std::process::Command::new("git").arg("-C").arg(dir).args(["config", k]).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).filter(|s| !s.is_empty());
    let name = git("user.name").or_else(|| std::env::var("USER").ok()).unwrap_or_default();
    let mail = git("user.email").map(|m| format!("  <{m}>")).unwrap_or_default();
    format!("{}  {name}{mail}", today())
}

fn today() -> String {
    std::process::Command::new("date").arg("+%Y-%m-%d").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
}

/// The ChangeLog for `file`: the nearest one in its folder or above, else a new one in the project's root.
fn find(file: &Path, root: &Path) -> PathBuf {
    let mut d = file.parent();
    while let Some(dir) = d {
        let c = dir.join("ChangeLog");
        if c.is_file() {
            return c;
        }
        if dir == root {
            break;
        }
        d = dir.parent();
    }
    root.join("ChangeLog")
}

/// The text with an entry for `rel` (and the function it is in) under today's heading: the heading
/// added when the newest is another day's or another person's; returns it and where the cursor goes.
pub(crate) fn add_entry(text: &str, head: &str, rel: &str, func: Option<&str>) -> (String, usize) {
    let item = match func {
        Some(f) => format!("\t* {rel} ({f}): "),
        None => format!("\t* {rel}: "),
    };
    if text.lines().next() == Some(head) {
        // under today's heading, first
        let at = head.len() + 2;
        let mut t = text.to_string();
        let ins = format!("{item}\n");
        if at <= t.len() {
            t.insert_str(at, &ins);
        } else {
            t.push_str(&format!("\n{ins}"));
        }
        return (t, at + item.chars().count());
    }
    let new = format!("{head}\n\n{item}\n\n");
    let at = head.chars().count() + 2 + item.chars().count();
    (format!("{new}{text}"), at)
}

impl App {
    /// C-x 4 a: the ChangeLog in another window, an entry for this file started under today's heading.
    pub(crate) fn changelog_entry(&mut self) {
        let Some(file) = self.docs[self.cur].abs_path() else { return self.say("this buffer has no file") };
        let root = self.git_root().unwrap_or_else(|| file.parent().map(Path::to_path_buf).unwrap_or_default());
        let log = find(&file, &root);
        let base = log.parent().unwrap_or(&root).to_path_buf();
        let rel = file.strip_prefix(&base).unwrap_or(&file).display().to_string();
        let func = self.outline_name_here();
        self.command(&format!("split {}", log.display()));
        let head = heading(&base);
        let old = self.docs[self.cur].ed.text();
        let (text, at) = add_entry(&old, &head, &rel, func.as_deref());
        let n = old.chars().count();
        // the new buffer takes the profile only on the next frame; the cursor goes past the line's end now
        if self.emacs_keys() {
            self.doc().ed.set_emacs(true);
        }
        self.doc().ed.replace_text(0, n, &text);
        let pos = self.docs[self.cur].ed.buf.to_pos(at);
        self.doc().ed.set_cursor(pos);
        self.doc().touched(0);
    }

    /// C-x v a: the commits since the ChangeLog's newest date, written into it as entries.
    pub(crate) fn changelog_update(&mut self) {
        let Some(root) = self.git_root() else { return self.say("not in a git repository") };
        let log = root.join("ChangeLog");
        let old = std::fs::read_to_string(&log).unwrap_or_default();
        let since = old.lines().next().and_then(|l| l.get(..10)).filter(|d| d.chars().filter(|c| *c == '-').count() == 2).map(str::to_string);
        let mut args = vec!["log".to_string(), "--date=short".into(), "--format=%x1e%ad  %an  <%ae>%x1f%s".into(), "--name-only".into()];
        if let Some(s) = &since {
            args.push(format!("--since={s} 00:00:00"));
        }
        let out = std::process::Command::new("git").arg("-C").arg(&root).args(&args).output().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
        let mut entries = String::new();
        let mut last_head = String::new();
        for block in out.split('\x1e').map(str::trim).filter(|b| !b.is_empty()) {
            let mut lines = block.lines();
            let Some(first) = lines.next() else { continue };
            let mut parts = first.split('\x1f');
            let (head, subject) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
            let files: Vec<&str> = lines.filter(|l| !l.is_empty()).collect();
            // the newest day is read again: what is logged already stays out
            if !subject.is_empty() && old.contains(subject) {
                continue;
            }
            if head != last_head {
                entries.push_str(&format!("{}{head}\n\n", if entries.is_empty() { "" } else { "\n" }));
                last_head = head.to_string();
            }
            entries.push_str(&format!("\t* {}: {subject}\n", if files.is_empty() { "." .to_string() } else { files.join(", ") }));
        }
        if entries.is_empty() {
            return self.say("No commits since the ChangeLog's newest entry");
        }
        self.command(&format!("e {}", log.display()));
        let n = self.docs[self.cur].ed.text().chars().count();
        let sep = if old.is_empty() { "" } else { "\n" };
        self.doc().ed.replace_text(0, n, &format!("{entries}{sep}{old}"));
        self.doc().touched(0);
        self.say("ChangeLog updated from the commits: C-x C-s saves it");
    }
}

#[cfg(test)]
mod tests {
    use super::add_entry;

    #[test]
    fn entries_go_under_todays_heading() {
        let head = "2026-10-23  A  <a@b>";
        let (t, at) = add_entry("", head, "src/x.rs", Some("parse"));
        assert_eq!(t, "2026-10-23  A  <a@b>\n\n\t* src/x.rs (parse): \n\n");
        assert_eq!(t.chars().nth(at - 1), Some(' '));
        // the same day: one heading, the newer entry first
        let (t2, _) = add_entry(&t, head, "y.rs", None);
        assert_eq!(t2, "2026-10-23  A  <a@b>\n\n\t* y.rs: \n\t* src/x.rs (parse): \n\n");
        // another day: a heading of its own on top
        let (t3, _) = add_entry("2026-10-01  A  <a@b>\n\n\t* z: old\n", head, "y.rs", None);
        assert!(t3.starts_with("2026-10-23  A  <a@b>\n\n\t* y.rs: \n\n2026-10-01"));
    }
}

impl App {
    /// The name of the definition the cursor is in (`fn parse(…)` → parse), for a ChangeLog entry.
    fn outline_name_here(&self) -> Option<String> {
        let ed = &self.docs[self.cur].ed;
        let line = ed.cursor().line;
        let s = ed.sections().into_iter().filter(|s| s.start <= line && line <= s.end).max_by_key(|s| s.level)?;
        let head = s.title.split('(').next().unwrap_or("");
        let name = head.split_whitespace().last()?.trim_matches(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.' || c == ':'));
        (!name.is_empty()).then(|| name.to_string())
    }
}
