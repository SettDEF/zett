//! Crash safety: unsaved changes kept on disk while you type (state/recover/), offered back when the file is opened
//! again after a crash (:recover / :recover!); each file reopens where its cursor was (state/places.tsv).
use super::*;
use std::collections::HashMap;

/// What has been kept and looked at in this session.
#[derive(Default)]
pub(crate) struct Keeper {
    /// a file's kept copy: the buffer revision it holds
    kept: HashMap<String, u64>,
    /// files already looked at for a copy and a place (once per session)
    seen: std::collections::HashSet<String>,
    /// where each file's cursor was: path → (line, col), read from places.tsv once
    places: Option<HashMap<String, (usize, usize)>>,
    places_dirty: bool,
    at: Option<Instant>,
    places_at: Option<Instant>,
}

fn dir() -> PathBuf {
    doom::state_dir()
}

/// The kept copy of `path`: <state>/recover/<hash of the path>.txt
fn copy_in(base: &Path, path: &str) -> PathBuf {
    let h = path.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3));
    base.join("recover").join(format!("{h:016x}.txt"))
}

/// A kept copy: (when it was written, unix seconds; the text). Its first lines say what it is and for which file.
fn read_copy(base: &Path, path: &str) -> Option<(u64, String)> {
    let s = std::fs::read_to_string(copy_in(base, path)).ok()?;
    let mut it = s.splitn(4, '\n');
    (it.next()? == "zett-recover 1").then_some(())?;
    (it.next()? == path).then_some(())?;
    let t = it.next()?.parse().ok()?;
    Some((t, it.next().unwrap_or("").to_string()))
}

fn write_copy(base: &Path, path: &str, text: &str) {
    let f = copy_in(base, path);
    let Some(parent) = f.parent() else { return };
    let _ = std::fs::create_dir_all(parent);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    // written whole, then renamed into place: a crash in the middle leaves the old copy, never half of one
    let tmp = f.with_extension("tmp");
    if std::fs::write(&tmp, format!("zett-recover 1\n{path}\n{now}\n{text}")).is_ok() {
        let _ = std::fs::rename(&tmp, &f);
    }
}

fn read_places() -> HashMap<String, (usize, usize)> {
    std::fs::read_to_string(dir().join("places.tsv"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut f = l.split('\t');
            Some((f.next()?.to_string(), (f.next()?.parse().ok()?, f.next()?.parse().ok()?)))
        })
        .collect()
}

fn write_places(p: &HashMap<String, (usize, usize)>) {
    let _ = std::fs::create_dir_all(dir());
    // the 2000 kept: enough for anyone's files, small to read at start
    let mut v: Vec<_> = p.iter().collect();
    v.sort();
    let body: String = v.iter().rev().take(2000).map(|(k, (l, c))| format!("{k}\t{l}\t{c}\n")).collect();
    let f = dir().join("places.tsv");
    let tmp = f.with_extension("tmp");
    if std::fs::write(&tmp, body).is_ok() {
        let _ = std::fs::rename(&tmp, &f);
    }
}

impl App {
    /// From the main loop: copies of unsaved buffers (a second after the last edit), copies of saved ones gone,
    /// a file opened for the first time put back at its place and told about a copy newer than it.
    pub(crate) fn recover_tick(&mut self) {
        if self.recover.at.is_some_and(|t| t.elapsed() < Duration::from_millis(500)) {
            return;
        }
        self.recover.at = Some(Instant::now());
        if self.recover.places.is_none() {
            self.recover.places = Some(read_places());
        }
        let quiet = self.edit_at.elapsed() > Duration::from_secs(1);
        let mut say = None;
        for i in 0..self.docs.len() {
            let Some(path) = self.docs[i].abs_path().map(|p| p.display().to_string()) else { continue };
            // first sight: its place, and a copy left by a session that did not end well
            if self.recover.seen.insert(path.clone()) {
                let c = self.docs[i].ed.cursor();
                if c.line == 0 && c.col == 0 {
                    if let Some(&(l, col)) = self.recover.places.as_ref().and_then(|p| p.get(&path)) {
                        let ed = &mut self.docs[i].ed;
                        let l = l.min(ed.buf.len_lines().saturating_sub(1));
                        let col = col.min(ed.buf.line(l).chars().count());
                        ed.set_cursor(zett_core::Pos { line: l, col });
                    }
                }
                if let Some((t, text)) = read_copy(&dir(), &path) {
                    let disk = std::fs::read_to_string(&path).unwrap_or_default();
                    let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok().and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
                    if text != disk && t >= mtime {
                        say = Some(format!("\"{}\" has unsaved changes from a session that ended without saving: :recover brings them back, :recover! drops them", self.docs[i].name));
                    } else {
                        let _ = std::fs::remove_file(copy_in(&dir(), &path));
                    }
                }
            }
            // its place, as the cursor is now
            let c = self.docs[i].ed.cursor();
            if let Some(p) = self.recover.places.as_mut() {
                if p.get(&path) != Some(&(c.line, c.col)) {
                    p.insert(path.clone(), (c.line, c.col));
                    self.recover.places_dirty = true;
                }
            }
            let d = &self.docs[i];
            if d.dirty() {
                let rev = d.ed.buf.rev();
                if quiet && self.recover.kept.get(&path) != Some(&rev) {
                    write_copy(&dir(), &path, &d.ed.text());
                    self.recover.kept.insert(path, rev);
                }
            } else if self.recover.kept.remove(&path).is_some() {
                // saved (or the changes undone): the copy is not needed
                let _ = std::fs::remove_file(copy_in(&dir(), &path));
            }
        }
        // the places on disk every few seconds, so a crash keeps them too
        if self.recover.places_dirty && self.recover.places_at.is_none_or(|t| t.elapsed() > Duration::from_secs(5)) {
            if let Some(p) = &self.recover.places { write_places(p) }
            self.recover.places_dirty = false;
            self.recover.places_at = Some(Instant::now());
        }
        if let Some(m) = say {
            self.say(m);
        }
    }

    /// zett is quitting as asked (:q!, :qa!): the places kept; copies of changes left unsaved on purpose dropped.
    pub(crate) fn recover_quit(&mut self) {
        self.recover.at = None;
        self.recover.places_at = None;
        self.recover_tick();
        if let Some(p) = &self.recover.places { write_places(p) }
        for d in &self.docs {
            if let Some(path) = d.abs_path() {
                let _ = std::fs::remove_file(copy_in(&dir(), &path.display().to_string()));
            }
        }
    }

    /// :recover — the kept unsaved changes back into the buffer (undo takes them out); :recover! drops them.
    pub(crate) fn recover_cmd(&mut self, bang: bool) {
        let i = self.cur;
        let Some(path) = self.docs[i].abs_path().map(|p| p.display().to_string()) else { return self.say("this buffer has no file") };
        let Some((t, text)) = read_copy(&dir(), &path) else { return self.say("no unsaved changes kept for this file") };
        if bang {
            let _ = std::fs::remove_file(copy_in(&dir(), &path));
            return self.say("the kept changes are dropped");
        }
        let n = self.docs[i].ed.buf.len_chars();
        self.docs[i].ed.replace_text(0, n, &text);
        self.docs[i].touched(0);
        let ago = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(t).saturating_sub(t);
        self.say(format!("changes from {} brought back: :w keeps them, u takes them out again", crate::travel::when(t, t + ago)));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_copy_names_its_file_and_is_read_back() {
        let dir = std::env::temp_dir().join(format!("zett-recover-{}", std::process::id()));
        let path = "/tmp/some file.rs";
        super::write_copy(&dir, path, "fn main() {}\nsecond\n");
        let (t, text) = super::read_copy(&dir, path).unwrap();
        assert!(t > 0);
        assert_eq!(text, "fn main() {}\nsecond\n");
        assert!(super::read_copy(&dir, "/tmp/other.rs").is_none(), "another file's copy is not this one");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod crash {
    // edit without saving, "crash" (no clean quit), open again: told, :recover brings the text back, the cursor
    // goes back to its place. Sets XDG_STATE_HOME itself: run alone (--ignored)
    #[test]
    #[ignore]
    fn unsaved_changes_survive_a_crash() {
        let dir = std::env::temp_dir().join(format!("zett-crash-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        unsafe { std::env::set_var("XDG_STATE_HOME", dir.join("state")) }
        let f = dir.join("a.txt");
        std::fs::write(&f, "one\ntwo\nthree\n").unwrap();
        {
            let mut app = crate::App::new(vec![f.clone()]);
            app.goto_line(3);
            app.command("normal! Aextra");
            app.edit_at = std::time::Instant::now() - std::time::Duration::from_secs(5);
            app.recover_tick();
            app.recover.places_at = None;
            app.recover.at = None;
            app.recover_tick();
            // dropped without recover_quit: the crash
        }
        let mut app = crate::App::new(vec![f.clone()]);
        app.recover_tick();
        let msg = app.msg.as_ref().map(|m| m.0.clone()).unwrap_or_default();
        assert!(msg.contains(":recover"), "told about it: {msg}");
        assert_eq!(app.docs[app.cur].ed.cursor().line, 2, "back at its place");
        app.command("recover");
        assert_eq!(app.docs[app.cur].ed.text(), "one\ntwo\nthreeextra\n");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// For the window (zett --gui): the same copies as the terminal's, so either recovers the other's.
pub fn keep_unsaved(path: &str, text: &str) {
    write_copy(&dir(), path, text)
}
/// A kept copy newer than the file and different from it: (unix seconds, the text).
pub fn kept_unsaved(path: &str) -> Option<(u64, String)> {
    let (t, text) = read_copy(&dir(), path)?;
    let disk = std::fs::read_to_string(path).unwrap_or_default();
    let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok().and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
    (text != disk && t >= mtime).then_some((t, text))
}
pub fn drop_unsaved(path: &str) {
    let _ = std::fs::remove_file(copy_in(&dir(), path));
}
