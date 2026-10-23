//! Time travel: scrub a buffer through every state it has been in (its undo tree, kept across
//! restarts), see the whole text as it was, and take back all of it, one section, or a copy of a
//! region. Nothing is lost on the way: going back is a step in the tree, not a rewrite.

use super::*;
use zett_edit::outline;

/// `undodir` when a config set it (empty: zett's own state folder), kept here for the undo files.
pub(crate) static UNDODIR: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

/// Where a file's history is kept: by `undodir` as Neovim reads it (its first folder that exists,
/// "." beside the file, one ending in "//" naming it by its whole path), else beside zett's other state.
fn history_file(path: &Path) -> PathBuf {
    let p = std::fs::canonicalize(path).unwrap_or(path.to_path_buf());
    if let Some(f) = undodir_file(&UNDODIR.lock().unwrap(), &p) {
        return f;
    }
    zett_history_file(&p)
}

/// zett's own place for it (a hash of the path, in its state folder).
fn zett_history_file(p: &Path) -> PathBuf {
    let h = p.display().to_string().bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3));
    doom::state_dir().join("history").join("undo").join(format!("{h:016x}"))
}

impl Doc {
    /// The history saved with this file, if it still fits the text (else a fresh one).
    pub(crate) fn load_history(&mut self) {
        let Some(p) = self.ed.buf.path().map(Path::to_path_buf) else { return };
        // by undodir, else where zett kept it before undodir was set
        if let Ok(bytes) = std::fs::read(history_file(&p)).or_else(|_| std::fs::read(zett_history_file(&std::fs::canonicalize(&p).unwrap_or(p.clone())))) {
            self.ed.history_load(&bytes);
        }
    }

    /// Keeps the history with the file, as the file is on disk now (after a save).
    pub(crate) fn keep_history(&mut self) {
        let Some(p) = self.ed.buf.path().map(Path::to_path_buf) else { return };
        if self.dirty() {
            return;
        }
        let f = history_file(&p);
        if let Some(dir) = f.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&f, self.ed.history_save());
        // and which file it is, for search everything (the name is only a hash)
        if let Some(h) = f.file_name() {
            everything::remember(&h.to_string_lossy(), &std::fs::canonicalize(&p).unwrap_or(p));
        }
    }
}

/// The view while travelling: which buffer, its states, the one shown, and that text as a
/// buffer of its own (drawn in place of the real one, moved through, copied from).
pub(crate) struct Travel {
    pub doc: usize,
    states: Vec<(usize, u64)>,
    now: usize,
    pub at: usize,
    pub preview: Doc,
}

/// "just now", "4 min ago", "3 h ago", "yesterday", "12 Oct", "12 Oct 2025".
pub(crate) fn when(t: u64, now: u64) -> String {
    let d = now.saturating_sub(t);
    match d {
        0..=59 => "just now".into(),
        60..=3599 => format!("{} min ago", d / 60),
        3600..=86_399 => format!("{} h ago", d / 3600),
        86_400..=172_799 => "yesterday".into(),
        _ => {
            let (y, m, day) = civil(t as i64 / 86_400);
            let (ny, _, _) = civil(now as i64 / 86_400);
            const M: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
            if y == ny { format!("{day} {}", M[m as usize - 1]) } else { format!("{day} {} {y}", M[m as usize - 1]) }
        }
    }
}

/// Days since 1970-01-01 to (year, month, day) — Howard Hinnant's civil_from_days.
fn civil(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { yoe + era * 400 + 1 } else { yoe + era * 400 }, m, d)
}

fn unix_now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

impl App {
    /// Starts time travel on the current buffer, at the state it is in.
    pub(crate) fn travel_start(&mut self) {
        let i = self.cur;
        let states = self.docs[i].ed.undo_states();
        if states.len() < 2 {
            return self.say("no history yet: this buffer has not changed (history is kept with each save)");
        }
        let now = self.docs[i].ed.undo_current();
        let preview = self.travel_preview(i, now);
        self.travel = Some(Travel { doc: i, states, now, at: now, preview });
    }

    /// The buffer as it was at state `n`, the cursor on the first line that differs from now.
    fn travel_preview(&mut self, i: usize, n: usize) -> Doc {
        let then = self.docs[i].ed.text_at(n);
        let cur = self.docs[i].ed.text();
        let line = then.lines().zip(cur.lines()).position(|(a, b)| a != b).unwrap_or_else(|| then.lines().count().min(cur.lines().count()));
        let mut ed = Editor::from_str(&then);
        ed.set_emacs(true);
        ed.use_kill_ring(&self.kill_ring);
        let mut d = Doc::new(ed, self.docs[i].name.clone(), self.docs[i].syntax.clone());
        d.ed.set_outline_kind(outline::OutlineKind::for_syntax(&d.syntax));
        let last = d.ed.buf.len_lines().saturating_sub(1);
        d.ed.set_cursor(Pos { line: line.min(last), col: 0 });
        d
    }

    /// Time travel shows state `to` (search everything opens a past hit there).
    pub(crate) fn travel_to(&mut self, to: usize) {
        self.travel_go(to);
    }

    fn travel_go(&mut self, to: usize) {
        let Some(t) = self.travel.as_ref() else { return };
        let to = to.min(t.states.len() - 1);
        if to == t.at {
            return;
        }
        let doc = t.doc;
        let preview = self.travel_preview(doc, to);
        if let Some(t) = self.travel.as_mut() {
            t.at = to;
            t.preview = preview;
        }
    }

    pub(crate) fn travel_key(&mut self, k: KeyEvent) {
        let Some(t) = self.travel.as_mut() else { return };
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let alt = k.modifiers.contains(KeyModifiers::ALT);
        let shift = k.modifiers.contains(KeyModifiers::SHIFT);
        let (at, last) = (t.at, t.states.len() - 1);
        let step = if shift { 10 } else { 1 };
        match k.code {
            KeyCode::Esc | KeyCode::Char('q') => self.travel = None,
            KeyCode::Char('g') if ctrl => self.travel = None,
            // through time
            KeyCode::Left | KeyCode::Char('[') | KeyCode::Char('h') => self.travel_go(at.saturating_sub(step)),
            KeyCode::Right | KeyCode::Char(']') | KeyCode::Char('l') => self.travel_go(at + step),
            KeyCode::Home => self.travel_go(0),
            KeyCode::End => self.travel_go(last),
            KeyCode::PageUp => self.travel_go(at.saturating_sub(10)),
            KeyCode::PageDown => self.travel_go(at + 10),
            // through the text as it was
            KeyCode::Up | KeyCode::Char('k') => {
                t.preview.ed.exec("previous-line", 1);
            }
            KeyCode::Char('p') if ctrl => {
                t.preview.ed.exec("previous-line", 1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                t.preview.ed.exec("next-line", 1);
            }
            KeyCode::Char('n') if ctrl => {
                t.preview.ed.exec("next-line", 1);
            }
            KeyCode::Char('v') if ctrl => {
                t.preview.ed.exec("next-line", 20);
            }
            KeyCode::Char('v') if alt => {
                t.preview.ed.exec("previous-line", 20);
            }
            KeyCode::Char(' ') if ctrl => {
                t.preview.ed.exec("set-mark-command", 1);
            }
            KeyCode::Char('v') => {
                t.preview.ed.exec("set-mark-command", 1);
            }
            // take something back
            KeyCode::Char('w') | KeyCode::Char('y') if alt || !ctrl => {
                let n = t.at;
                let had = t.preview.ed.kill_count();
                t.preview.ed.exec("copy-region-as-kill", 1);
                if t.preview.ed.kill_count() == had {
                    return self.say("mark a region first (v or C-SPC), then y / M-w copies it");
                }
                if let Some(text) = t.preview.ed.last_kill() {
                    clipboard::copy(&text);
                }
                let ago = when(t.states[n].1, unix_now());
                self.say(format!("copied from {ago}: C-y puts it in"));
            }
            KeyCode::Enter => {
                let (doc, n) = (t.doc, t.at);
                self.travel = None;
                let before = self.docs[doc].ed.cursor().line;
                self.docs[doc].ed.undo_goto(n);
                self.docs[doc].touched(before);
                self.say(format!("back at state {n}; the later ones are kept (C-c t, or :later)"));
            }
            KeyCode::Char('b') => {
                let (doc, n) = (t.doc, t.at);
                let line = t.preview.ed.cursor().line;
                let secs = t.preview.ed.sections();
                let Some(title) = outline::at(&secs, line).map(|i| secs[i].title.clone()) else {
                    return self.say("the cursor is not in a section (a heading or a block)");
                };
                let before = self.docs[doc].ed.cursor().line;
                if self.docs[doc].ed.restore_section(n, &title) {
                    self.docs[doc].touched(before);
                    self.travel = None;
                    self.say(format!("{title} is back as it was; everything else stays (C-/ undoes)"));
                } else {
                    self.say(format!("{title} is not in the buffer now"));
                }
            }
            _ => {}
        }
    }

    /// The echo area while travelling: the slider, when, and the keys.
    pub(crate) fn draw_travel(&self, f: &mut Frame, a: Rect) {
        let Some(t) = &self.travel else { return };
        let last = t.states.len() - 1;
        let ago = when(t.states[t.at].1, unix_now());
        let label = format!(" {}/{last} {ago} ", t.at);
        let keys = " ←→ time · Enter all · b section · v y copy · q ";
        let room = (a.width as usize).saturating_sub(label.chars().count() + keys.chars().count() + 2).clamp(8, 60);
        let pos = if last == 0 { 0 } else { t.at * (room - 1) / last };
        let now = if last == 0 { 0 } else { t.now * (room - 1) / last };
        let bar: String = (0..room).map(|i| if i == pos { '●' } else if i == now { '┃' } else if i < pos { '━' } else { '─' }).collect();
        crate::layer(f, a);
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(" ⏲ ", hl("Prompt")), Span::styled(bar, hl("PickerPrompt")), Span::styled(label, hl("Prompt")), Span::styled(keys, hl("PickerDir"))])),
            a,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_read_like_people_say_them() {
        let now = 1_792_454_400; // 2026-10-20 00:00 UTC
        assert_eq!(when(now - 10, now), "just now");
        assert_eq!(when(now - 600, now), "10 min ago");
        assert_eq!(when(now - 7200, now), "2 h ago");
        assert_eq!(when(now - 90_000, now), "yesterday");
        assert_eq!(when(now - 6 * 86_400, now), "14 Oct");
        assert_eq!(when(1_735_689_600, now), "1 Jan 2025");
    }
}

/// The undo file for `p` (a canonical path) by `undodir` (Neovim's rules; zett's own format, so
/// ".zett" ends its name and Neovim's own files beside it are left alone).
fn undodir_file(undodir: &str, p: &Path) -> Option<PathBuf> {
    let name = p.file_name()?.to_string_lossy().to_string();
    undodir.split(',').map(str::trim).filter(|d| !d.is_empty()).find_map(|d| {
        if d == "." {
            return Some(p.parent()?.join(format!(".{name}.zett-undo")));
        }
        let dir = PathBuf::from(shellexpand_home(d.trim_end_matches('/')));
        if !dir.is_dir() {
            return None;
        }
        Some(if d.ends_with("//") { dir.join(format!("{}.zett", p.display().to_string().replace('/', "%"))) } else { dir.join(format!("{name}.zett")) })
    })
}

fn shellexpand_home(d: &str) -> String {
    match (d.strip_prefix("~"), std::env::var("HOME")) {
        (Some(rest), Ok(h)) => format!("{h}{rest}"),
        _ => d.to_string(),
    }
}

impl App {
    /// `undodir` as the config set it, for the undo files (unset: zett's own state folder; not
    /// Neovim's default, whose undo files are another format).
    pub(crate) fn sync_undodir(&self) {
        let v = self.engine.opt("undodir").unwrap_or_default();
        let mut u = UNDODIR.lock().unwrap();
        if *u != v {
            *u = v;
        }
    }
}

#[cfg(test)]
mod undodir_tests {
    use super::*;

    #[test]
    fn undodir_places_like_neovim() {
        let tmp = std::env::temp_dir().join(format!("zett-udir-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let p = Path::new("/home/u/notes/a.md");
        assert_eq!(undodir_file(".", p), Some(PathBuf::from("/home/u/notes/.a.md.zett-undo")));
        let d = tmp.display().to_string();
        assert_eq!(undodir_file(&format!("{d}//"), p), Some(tmp.join("%home%u%notes%a.md.zett")));
        assert_eq!(undodir_file(&d, p), Some(tmp.join("a.md.zett")));
        // the first folder that exists
        assert_eq!(undodir_file(&format!("/nope/x,{d}"), p), Some(tmp.join("a.md.zett")));
        assert_eq!(undodir_file("", p), None);
        let _ = std::fs::remove_dir(&tmp);
    }
}
