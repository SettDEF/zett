//! Emacs's proced: the processes as a buffer (M-x proced, :Proced), busiest first, new every 2 s. m marks, u unmarks,
//! U unmarks all, x twice sends SIGTERM to the marked ones (or the one under the cursor), s sorts by CPU / memory /
//! pid, g reads them again, Enter says the whole command line, q closes. Read from ps; signals through kill.
use super::*;
use std::collections::HashSet;

pub(crate) const NAME: &str = "*proced*";

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Proc {
    pub pid: u32,
    pub user: String,
    pub cpu: f32,
    pub mem: f32,
    pub time: String,
    pub cmd: String,
}

#[derive(Default)]
pub(crate) struct Proced {
    pub rows: Vec<Proc>,
    pub marked: HashSet<u32>,
    /// 0 CPU, 1 memory, 2 pid
    pub sort: u8,
    /// x pressed once: the pids it would signal (a second x sends)
    pub confirm: Option<Vec<u32>>,
    pub at: Option<Instant>,
    /// signals sent (tests read this; nothing is sent there)
    pub sent: Vec<u32>,
}

/// ps's lines (pid user %cpu %mem etime args) as processes; the header and broken lines skipped.
pub(crate) fn parse(ps: &str) -> Vec<Proc> {
    ps.lines().skip(1).filter_map(|l| {
        let mut f = l.split_whitespace();
        Some(Proc {
            pid: f.next()?.parse().ok()?,
            user: f.next()?.to_string(),
            cpu: f.next()?.parse().ok()?,
            mem: f.next()?.parse().ok()?,
            time: f.next()?.to_string(),
            cmd: f.collect::<Vec<_>>().join(" "),
        })
    }).collect()
}

fn read() -> Vec<Proc> {
    let o = std::process::Command::new("ps").args(["-eo", "pid,user,%cpu,%mem,etime,args", "--no-headers"]).output();
    // --no-headers: a header put back for parse
    o.map(|o| parse(&format!("PID\n{}", String::from_utf8_lossy(&o.stdout)))).unwrap_or_default()
}

impl Proced {
    fn sorted(&mut self) {
        match self.sort {
            1 => self.rows.sort_by(|a, b| b.mem.total_cmp(&a.mem)),
            2 => self.rows.sort_by_key(|p| p.pid),
            _ => self.rows.sort_by(|a, b| b.cpu.total_cmp(&a.cpu)),
        }
    }

    /// The buffer's text: a header, then a row per process (* before a marked one).
    pub(crate) fn text(&self, width: usize) -> String {
        let by = ["CPU", "memory", "pid"][self.sort as usize % 3];
        let mut out = format!("  {:>7} {:<10} {:>5} {:>5} {:>11}  COMMAND   (by {by} · m mark · x x kill · s sort · g · q)\n", "PID", "USER", "%CPU", "%MEM", "TIME");
        for p in &self.rows {
            let mark = if self.marked.contains(&p.pid) { "* " } else { "  " };
            let line = format!("{mark}{:>7} {:<10} {:>5.1} {:>5.1} {:>11}  {}", p.pid, p.user.chars().take(10).collect::<String>(), p.cpu, p.mem, p.time, p.cmd);
            out.push_str(&line.chars().take(width.max(40)).collect::<String>());
            out.push('\n');
        }
        out
    }
}

impl App {
    /// M-x proced: the list (made, or shown again).
    pub(crate) fn proced_open(&mut self) {
        if self.proced.is_none() {
            self.proced = Some(Proced::default());
        }
        let i = match self.docs.iter().position(|d| d.name == NAME) {
            Some(i) => i,
            None => {
                let mut d = Doc::new(Editor::from_str(""), NAME.into(), "Plain Text".into());
                d.local.insert("buftype".into(), "nofile".into());
                d.local.insert("modifiable".into(), "false".into());
                d.local.insert("wrap".into(), "false".into());
                self.docs.push(d);
                self.docs.len() - 1
            }
        };
        self.show_doc(i);
        self.proced_refresh();
    }

    /// The processes read again, the buffer rewritten (the cursor stays on its process when it is still there).
    pub(crate) fn proced_refresh(&mut self) {
        let Some(i) = self.docs.iter().position(|d| d.name == NAME) else { self.proced = None; return };
        let Some(p) = self.proced.as_mut() else { return };
        let line = self.docs[i].ed.cursor().line;
        let at_pid = line.checked_sub(1).and_then(|r| p.rows.get(r)).map(|r| r.pid);
        p.rows = if cfg!(test) && !p.rows.is_empty() { p.rows.clone() } else { read() };
        p.sorted();
        p.marked.retain(|m| p.rows.iter().any(|r| r.pid == *m));
        p.at = Some(Instant::now());
        let text = p.text(self.area.width.max(80) as usize);
        let new_line = at_pid.and_then(|pid| p.rows.iter().position(|r| r.pid == pid)).map(|r| r + 1).unwrap_or(line);
        let ed = &mut self.docs[i].ed;
        ed.set_opt("modifiable", "true");
        let n = ed.buf.len_chars();
        ed.replace_text(0, n, &text);
        ed.set_opt("modifiable", "false");
        // never on the header: the first process at the least
        let new_line = new_line.max(1).min(ed.buf.len_lines().saturating_sub(2).max(1));
        ed.set_cursor(zett_core::Pos { line: new_line, col: 0 });
        self.docs[i].touched(0);
    }

    /// From the main loop: the list new every 2 s while it is the buffer shown.
    pub(crate) fn proced_tick(&mut self) {
        let shown = self.docs.get(self.cur).is_some_and(|d| d.name == NAME);
        if shown && self.proced.as_ref().is_some_and(|p| p.confirm.is_none() && p.at.is_none_or(|t| t.elapsed() > Duration::from_secs(2))) {
            self.proced_refresh();
        }
    }

    /// Its keys (normal mode, or the Emacs profile's); false: not the list, or not one of them.
    pub(crate) fn proced_key(&mut self, k: &KeyEvent) -> bool {
        if self.docs[self.cur].name != NAME || self.proced.is_none() {
            return false;
        }
        let line = self.docs[self.cur].ed.cursor().line;
        let p = self.proced.as_mut().unwrap();
        let here = line.checked_sub(1).and_then(|r| p.rows.get(r)).cloned();
        let down = |app: &mut App| {
            let ed = &mut app.docs[app.cur].ed;
            let l = (line + 1).min(ed.buf.len_lines().saturating_sub(2));
            ed.set_cursor(zett_core::Pos { line: l, col: 0 });
        };
        // a second x sends; anything else forgets the first
        let confirm = p.confirm.take();
        match k.code {
            KeyCode::Char('m') => {
                if let Some(h) = &here { p.marked.insert(h.pid); }
                self.proced_refresh();
                down(self);
            }
            KeyCode::Char('u') => {
                if let Some(h) = &here { p.marked.remove(&h.pid); }
                self.proced_refresh();
                down(self);
            }
            KeyCode::Char('U') => {
                p.marked.clear();
                self.proced_refresh();
            }
            KeyCode::Char('x') => {
                let want: Vec<u32> = if p.marked.is_empty() { here.iter().map(|h| h.pid).collect() } else { p.marked.iter().copied().collect() };
                if want.is_empty() {
                    return true;
                }
                if confirm.as_ref() == Some(&want) {
                    let mut failed = 0;
                    for pid in &want {
                        p.sent.push(*pid);
                        if !cfg!(test) && !std::process::Command::new("kill").args(["-TERM", &pid.to_string()]).status().is_ok_and(|s| s.success()) {
                            failed += 1;
                        }
                    }
                    p.marked.clear();
                    p.at = None;
                    let n = want.len();
                    self.say(if failed == 0 { format!("SIGTERM sent to {n}") } else { format!("SIGTERM sent to {}, {failed} refused (not yours?)", n - failed) });
                    self.proced_refresh();
                } else {
                    let names: Vec<String> = want.iter().filter_map(|pid| p.rows.iter().find(|r| r.pid == *pid)).take(4).map(|r| format!("{} {}", r.pid, r.cmd.split_whitespace().next().unwrap_or("").rsplit('/').next().unwrap_or(""))).collect();
                    p.confirm = Some(want.clone());
                    self.say(format!("x again sends SIGTERM to {}: {}", want.len(), names.join(", ")));
                }
            }
            KeyCode::Char('s') => {
                p.sort = (p.sort + 1) % 3;
                self.proced_refresh();
            }
            KeyCode::Char('g') => {
                p.at = None;
                self.proced_refresh();
            }
            KeyCode::Enter => {
                if let Some(h) = here { self.say(format!("{}: {}", h.pid, h.cmd)) }
            }
            KeyCode::Char('q') => {
                self.proced = None;
                self.command("bdelete!");
            }
            _ => {
                if confirm.is_some() {
                    self.say("not sent");
                }
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    const PS: &str = "PID USER %CPU %MEM ELAPSED COMMAND\n  10 cux 50.0 1.0 01:00 /usr/bin/busy --flag\n  20 cux 2.0 30.0 02:00 big\n  30 root 0.0 0.1 03:00 init\n";

    #[test]
    fn ps_lines_parse_and_sort() {
        let rows = parse(PS);
        assert_eq!(rows.len(), 3);
        assert_eq!((rows[0].pid, rows[0].cmd.as_str()), (10, "/usr/bin/busy --flag"));
        let mut p = Proced { rows, sort: 1, ..Default::default() };
        p.sorted();
        assert_eq!(p.rows[0].pid, 20, "by memory");
    }

    // x twice signals (recorded only: tests send nothing), once only asks; marks pick which
    #[test]
    fn x_asks_then_sends_to_the_marked() {
        let mut app = crate::App::new(Vec::new());
        app.proced = Some(Proced { rows: parse(PS), ..Default::default() });
        app.proced_open();
        let key = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
        app.docs[app.cur].ed.set_cursor(zett_core::Pos { line: 1, col: 0 });
        app.key(key('m'));
        app.key(key('m'));
        assert_eq!(app.proced.as_ref().unwrap().marked.len(), 2);
        assert!(app.docs[app.cur].ed.text().lines().filter(|l| l.starts_with("* ")).count() == 2, "{}", app.docs[app.cur].ed.text());
        app.key(key('x'));
        assert!(app.proced.as_ref().unwrap().sent.is_empty(), "the first x only asks");
        app.key(key('x'));
        let mut sent = app.proced.as_ref().unwrap().sent.clone();
        sent.sort();
        assert_eq!(sent, vec![10, 20]);
    }
}
