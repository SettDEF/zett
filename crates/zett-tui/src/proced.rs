//! Emacs's proced: the processes as a buffer (M-x proced, :Proced), busiest first, new every 2 s. With Vitals
//! installed it is Vitals' view (vitals top: CPU, GPU, memory, power per app, and `vitals why` in the header) and its
//! controls: f freezes / thaws, l limits (Vitals' background preset), L takes the limit off. Without it: ps.
//! m marks, u unmarks, U unmarks all, x twice sends SIGTERM (the marked ones, or the one under the cursor), s sorts,
//! g reads again, Enter says the whole command line, q closes. Reading and acting run off the editor's thread
//! (vitals measures for a second).
use super::*;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver};

pub(crate) const NAME: &str = "*proced*";

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Proc {
    pub pid: u32,
    pub user: String,
    pub cpu: f32,
    /// % of memory (ps) — Vitals gives MB instead
    pub mem: f32,
    pub mem_mb: Option<u64>,
    pub gpu: Option<f32>,
    pub power: Option<f32>,
    pub time: String,
    pub cmd: String,
}

#[derive(Default)]
pub(crate) struct Proced {
    pub rows: Vec<Proc>,
    pub marked: HashSet<u32>,
    /// Vitals: CPU, memory, GPU, power · ps: CPU, memory, pid
    pub sort: u8,
    /// x pressed once: the pids it would signal (a second x sends)
    pub confirm: Option<Vec<u32>>,
    pub at: Option<Instant>,
    /// what was sent or asked of Vitals (tests read this; nothing is run there)
    pub sent: Vec<String>,
    /// Vitals is installed (looked for once)
    pub vitals: Option<bool>,
    /// its sentence on what is making the laptop slow or hot, and when it was asked
    pub why: String,
    why_at: Option<Instant>,
    /// apps frozen or limited from here (shown with ❄ / ↓)
    pub frozen: HashSet<u32>,
    pub limited: HashSet<u32>,
    /// a reading under way: its result comes here
    pending: Option<Receiver<(Vec<Proc>, Option<String>)>>,
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
            ..Default::default()
        })
    }).collect()
}

/// `vitals top --json`: its apps as processes.
pub(crate) fn parse_vitals(json: &str) -> Vec<Proc> {
    let Ok(serde_json::Value::Array(a)) = serde_json::from_str::<serde_json::Value>(json) else { return Vec::new() };
    a.iter().filter_map(|v| {
        Some(Proc {
            pid: v["pid"].as_u64()? as u32,
            cpu: v["cpu_pct"].as_f64().unwrap_or(0.0) as f32,
            mem_mb: v["mem_mb"].as_u64(),
            gpu: v["gpu_pct"].as_f64().map(|x| x as f32),
            power: v["power_w"].as_f64().map(|x| x as f32),
            cmd: v["name"].as_str().unwrap_or("").to_string(),
            ..Default::default()
        })
    }).collect()
}

fn read_ps() -> Vec<Proc> {
    let o = std::process::Command::new("ps").args(["-eo", "pid,user,%cpu,%mem,etime,args", "--no-headers"]).output();
    // --no-headers: a header put back for parse
    o.map(|o| parse(&format!("PID\n{}", String::from_utf8_lossy(&o.stdout)))).unwrap_or_default()
}

fn has_vitals() -> bool {
    std::process::Command::new("sh").args(["-c", "command -v vitals"]).stdout(std::process::Stdio::null()).status().is_ok_and(|s| s.success())
}

/// The whole command line of a process (Vitals names only the program).
fn cmdline(pid: u32) -> String {
    std::fs::read(format!("/proc/{pid}/cmdline")).map(|b| b.split(|c| *c == 0).filter(|s| !s.is_empty()).map(|s| String::from_utf8_lossy(s).into_owned()).collect::<Vec<_>>().join(" ")).unwrap_or_default()
}

impl Proced {
    fn sorted(&mut self) {
        let vitals = self.vitals == Some(true);
        match (vitals, self.sort) {
            (true, 1) => self.rows.sort_by_key(|p| std::cmp::Reverse(p.mem_mb.unwrap_or(0))),
            (true, 2) => self.rows.sort_by(|a, b| b.gpu.unwrap_or(0.0).total_cmp(&a.gpu.unwrap_or(0.0))),
            (true, 3) => self.rows.sort_by(|a, b| b.power.unwrap_or(0.0).total_cmp(&a.power.unwrap_or(0.0))),
            (false, 1) => self.rows.sort_by(|a, b| b.mem.total_cmp(&a.mem)),
            (false, 2) => self.rows.sort_by_key(|p| p.pid),
            _ => self.rows.sort_by(|a, b| b.cpu.total_cmp(&a.cpu)),
        }
    }

    fn sorts(&self) -> u8 {
        if self.vitals == Some(true) { 4 } else { 3 }
    }

    /// The buffer's text: Vitals' sentence, a header, then a row per process (* marked, ❄ frozen, ↓ limited).
    pub(crate) fn text(&self, width: usize) -> String {
        let vitals = self.vitals == Some(true);
        let by = if vitals { ["CPU", "memory", "GPU", "power"] } else { ["CPU", "memory", "pid", "pid"] }[self.sort as usize % 4];
        let keys = if vitals { "m mark · f freeze · l limit · L free · x x kill · s sort · g · q" } else { "m mark · x x kill · s sort · g · q" };
        let mut out = String::new();
        if vitals {
            out.push_str(&format!("  {}\n", if self.why.is_empty() { "Vitals is looking …" } else { &self.why }));
            out.push_str(&format!("  {:>7} {:>5} {:>5} {:>7} {:>6}  APP   (by {by} · {keys})\n", "PID", "%CPU", "%GPU", "MEM MB", "W"));
        } else {
            out.push_str(&format!("  {:>7} {:<10} {:>5} {:>5} {:>11}  COMMAND   (by {by} · {keys})\n", "PID", "USER", "%CPU", "%MEM", "TIME"));
        }
        for p in &self.rows {
            let mark = if self.marked.contains(&p.pid) { '*' } else if self.frozen.contains(&p.pid) { '❄' } else if self.limited.contains(&p.pid) { '↓' } else { ' ' };
            let line = if vitals {
                format!("{mark} {:>7} {:>5.0} {:>5.0} {:>7} {:>6.1}  {}", p.pid, p.cpu, p.gpu.unwrap_or(0.0), p.mem_mb.unwrap_or(0), p.power.unwrap_or(0.0), p.cmd)
            } else {
                format!("{mark} {:>7} {:<10} {:>5.1} {:>5.1} {:>11}  {}", p.pid, p.user.chars().take(10).collect::<String>(), p.cpu, p.mem, p.time, p.cmd)
            };
            out.push_str(&line.chars().take(width.max(40)).collect::<String>());
            out.push('\n');
        }
        out
    }

    /// The rows before the first process (the sentence, the header).
    fn head(&self) -> usize {
        if self.vitals == Some(true) { 2 } else { 1 }
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
        if cfg!(test) {
            if let Some(p) = self.proced.as_mut() { p.vitals.get_or_insert(false); }
        }
        self.proced_read();
        self.proced_show();
    }

    /// A reading started off the editor's thread (none while one is under way).
    fn proced_read(&mut self) {
        let Some(p) = self.proced.as_mut() else { return };
        if p.pending.is_some() {
            return;
        }
        let vitals = *p.vitals.get_or_insert_with(has_vitals);
        p.at = Some(Instant::now());
        if cfg!(test) {
            return;
        }
        let what = ["cpu", "mem", "gpu", "power"][p.sort as usize % 4].to_string();
        // the sentence every 10 s at most (it takes a second and a half)
        let ask_why = vitals && p.why_at.is_none_or(|t| t.elapsed() > Duration::from_secs(10));
        if ask_why {
            p.why_at = Some(Instant::now());
        }
        let (tx, rx) = channel();
        p.pending = Some(rx);
        std::thread::spawn(move || {
            let rows = if vitals {
                std::process::Command::new("vitals").args(["top", &what, "-n", "80", "--json"]).output().map(|o| parse_vitals(&String::from_utf8_lossy(&o.stdout))).unwrap_or_default()
            } else {
                read_ps()
            };
            let why = if ask_why { std::process::Command::new("vitals").arg("why").output().ok().map(|o| String::from_utf8_lossy(&o.stdout).lines().next().unwrap_or("").trim_start_matches(['●', ' ']).to_string()) } else { None };
            let _ = tx.send((rows, why));
        });
    }

    /// The buffer rewritten from the rows (the cursor stays on its process when it is still there).
    pub(crate) fn proced_show(&mut self) {
        let Some(i) = self.docs.iter().position(|d| d.name == NAME) else { self.proced = None; return };
        let Some(p) = self.proced.as_mut() else { return };
        let line = self.docs[i].ed.cursor().line;
        let head = p.head();
        let at_pid = line.checked_sub(head).and_then(|r| p.rows.get(r)).map(|r| r.pid);
        p.sorted();
        p.marked.retain(|m| p.rows.iter().any(|r| r.pid == *m));
        let text = p.text(self.area.width.max(80) as usize);
        let new_line = at_pid.and_then(|pid| p.rows.iter().position(|r| r.pid == pid)).map(|r| r + head).unwrap_or(line);
        let ed = &mut self.docs[i].ed;
        ed.set_opt("modifiable", "true");
        let n = ed.buf.len_chars();
        ed.replace_text(0, n, &text);
        ed.set_opt("modifiable", "false");
        // the list is not a change to save (no "modified" mark)
        ed.buf.mark_saved();
        // never on the header: the first process at the least
        let new_line = new_line.max(head).min(ed.buf.len_lines().saturating_sub(2).max(head));
        ed.set_cursor(zett_core::Pos { line: new_line, col: 0 });
        self.docs[i].touched(0);
    }

    /// From the main loop: a reading that came is shown; a new one every 2 s while the list is the buffer shown.
    pub(crate) fn proced_tick(&mut self) {
        let Some(p) = self.proced.as_mut() else { return };
        if let Some(rx) = &p.pending {
            match rx.try_recv() {
                Ok((rows, why)) => {
                    p.pending = None;
                    if !rows.is_empty() {
                        p.rows = rows;
                    }
                    if let Some(w) = why {
                        p.why = w;
                    }
                    self.proced_show();
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => p.pending = None,
                Err(_) => {}
            }
            return;
        }
        let shown = self.docs.get(self.cur).is_some_and(|d| d.name == NAME);
        let p = self.proced.as_ref().unwrap();
        if shown && p.confirm.is_none() && p.at.is_none_or(|t| t.elapsed() > Duration::from_secs(2)) {
            self.proced_read();
        }
    }

    /// A Vitals control on processes (freeze, thaw, limit, free), run off the editor's thread; recorded in tests.
    fn proced_vitals(&mut self, args: Vec<String>) {
        if let Some(p) = self.proced.as_mut() {
            p.sent.push(args.join(" "));
        }
        if cfg!(test) {
            return;
        }
        std::thread::spawn(move || {
            let _ = std::process::Command::new("vitals").args(&args).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
        });
    }

    /// Its keys (normal mode, or the Emacs profile's); false: not the list, or not one of them.
    pub(crate) fn proced_key(&mut self, k: &KeyEvent) -> bool {
        if self.docs[self.cur].name != NAME || self.proced.is_none() {
            return false;
        }
        let line = self.docs[self.cur].ed.cursor().line;
        let p = self.proced.as_mut().unwrap();
        let head = p.head();
        let vitals = p.vitals == Some(true);
        let here = line.checked_sub(head).and_then(|r| p.rows.get(r)).cloned();
        let down = |app: &mut App| {
            let ed = &mut app.docs[app.cur].ed;
            let l = (line + 1).min(ed.buf.len_lines().saturating_sub(2));
            ed.set_cursor(zett_core::Pos { line: l, col: 0 });
        };
        // the marked ones, or the one under the cursor
        let chosen: Vec<u32> = if p.marked.is_empty() { here.iter().map(|h| h.pid).collect() } else { p.marked.iter().copied().collect() };
        // a second x sends; anything else forgets the first
        let confirm = p.confirm.take();
        match k.code {
            KeyCode::Char('m') => {
                if let Some(h) = &here {
                    p.marked.insert(h.pid);
                }
                self.proced_show();
                down(self);
            }
            KeyCode::Char('u') => {
                if let Some(h) = &here {
                    p.marked.remove(&h.pid);
                }
                self.proced_show();
                down(self);
            }
            KeyCode::Char('U') => {
                p.marked.clear();
                self.proced_show();
            }
            KeyCode::Char('f') if vitals => {
                // frozen ones thaw, the others freeze
                let (thaw, freeze): (Vec<u32>, Vec<u32>) = chosen.iter().partition(|pid| p.frozen.contains(pid));
                for pid in &freeze {
                    p.frozen.insert(*pid);
                }
                for pid in &thaw {
                    p.frozen.remove(pid);
                }
                p.marked.clear();
                for pid in &freeze {
                    self.proced_vitals(vec!["freeze".into(), pid.to_string()]);
                }
                for pid in &thaw {
                    self.proced_vitals(vec!["thaw".into(), pid.to_string()]);
                }
                self.say(format!("{} frozen, {} thawed (f again thaws)", freeze.len(), thaw.len()));
                self.proced_show();
            }
            KeyCode::Char('l') if vitals => {
                for pid in &chosen {
                    p.limited.insert(*pid);
                }
                p.marked.clear();
                for pid in &chosen {
                    self.proced_vitals(vec!["limit".into(), pid.to_string(), "--preset".into(), "background".into()]);
                }
                self.say(format!("{} limited to Vitals' background preset (L takes it off)", chosen.len()));
                self.proced_show();
            }
            KeyCode::Char('L') if vitals => {
                for pid in &chosen {
                    p.limited.remove(pid);
                }
                p.marked.clear();
                for pid in &chosen {
                    self.proced_vitals(vec!["free".into(), pid.to_string()]);
                }
                self.say(format!("{} free of limits", chosen.len()));
                self.proced_show();
            }
            KeyCode::Char('x') => {
                if chosen.is_empty() {
                    return true;
                }
                if confirm.as_ref() == Some(&chosen) {
                    let mut failed = 0;
                    for pid in &chosen {
                        p.sent.push(format!("kill {pid}"));
                        if !cfg!(test) && !std::process::Command::new("kill").args(["-TERM", &pid.to_string()]).status().is_ok_and(|s| s.success()) {
                            failed += 1;
                        }
                    }
                    p.marked.clear();
                    p.at = None;
                    let n = chosen.len();
                    self.say(if failed == 0 { format!("SIGTERM sent to {n}") } else { format!("SIGTERM sent to {}, {failed} refused (not yours?)", n - failed) });
                    self.proced_show();
                } else {
                    let names: Vec<String> = chosen.iter().filter_map(|pid| p.rows.iter().find(|r| r.pid == *pid)).take(4).map(|r| format!("{} {}", r.pid, r.cmd.split_whitespace().next().unwrap_or("").rsplit('/').next().unwrap_or(""))).collect();
                    p.confirm = Some(chosen.clone());
                    self.say(format!("x again sends SIGTERM to {}: {}", chosen.len(), names.join(", ")));
                }
            }
            KeyCode::Char('s') => {
                p.sort = (p.sort + 1) % p.sorts();
                p.at = None;
                self.proced_show();
                self.proced_read();
            }
            KeyCode::Char('g') => {
                p.at = None;
                self.proced_read();
            }
            KeyCode::Enter => {
                if let Some(h) = here {
                    let full = cmdline(h.pid);
                    self.say(format!("{}: {}", h.pid, if full.is_empty() { h.cmd } else { full }));
                }
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
    const VITALS: &str = r#"[{"cpu_pct":36.0,"gpu_pct":40.4,"mem_mb":98,"name":"loom","pid":6816,"power_w":21.5},{"cpu_pct":16.0,"gpu_pct":0.7,"mem_mb":602,"name":"zen","pid":1021,"power_w":1.3}]"#;

    #[test]
    fn ps_and_vitals_parse_and_sort() {
        let rows = parse(PS);
        assert_eq!(rows.len(), 3);
        assert_eq!((rows[0].pid, rows[0].cmd.as_str()), (10, "/usr/bin/busy --flag"));
        let mut p = Proced { rows, sort: 1, vitals: Some(false), ..Default::default() };
        p.sorted();
        assert_eq!(p.rows[0].pid, 20, "by memory");
        let v = parse_vitals(VITALS);
        assert_eq!((v[0].pid, v[0].gpu, v[1].mem_mb), (6816, Some(40.4), Some(602)));
        let mut p = Proced { rows: v, sort: 1, vitals: Some(true), ..Default::default() };
        p.sorted();
        assert_eq!(p.rows[0].pid, 1021, "by memory (MB)");
        assert!(p.text(100).contains("APP"));
    }

    // x twice signals (recorded only: tests send nothing), once only asks; marks pick which
    #[test]
    fn x_asks_then_sends_to_the_marked() {
        let mut app = crate::App::new(Vec::new());
        app.proced = Some(Proced { rows: parse(PS), vitals: Some(false), ..Default::default() });
        app.proced_open();
        let key = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
        app.docs[app.cur].ed.set_cursor(zett_core::Pos { line: 1, col: 0 });
        app.key(key('m'));
        app.key(key('m'));
        assert_eq!(app.proced.as_ref().unwrap().marked.len(), 2);
        assert!(app.docs[app.cur].ed.text().lines().filter(|l| l.starts_with('*')).count() == 2, "{}", app.docs[app.cur].ed.text());
        app.key(key('x'));
        assert!(app.proced.as_ref().unwrap().sent.is_empty(), "the first x only asks");
        app.key(key('x'));
        let mut sent = app.proced.as_ref().unwrap().sent.clone();
        sent.sort();
        assert_eq!(sent, vec!["kill 10".to_string(), "kill 20".to_string()]);
    }

    // with Vitals: f freezes then thaws, l limits, L frees (asked of Vitals; recorded only here)
    #[test]
    fn vitals_controls() {
        let mut app = crate::App::new(Vec::new());
        app.proced = Some(Proced { rows: parse_vitals(VITALS), vitals: Some(true), ..Default::default() });
        app.proced_open();
        let key = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
        app.docs[app.cur].ed.set_cursor(zett_core::Pos { line: 2, col: 0 });
        app.key(key('f'));
        assert!(app.docs[app.cur].ed.text().lines().any(|l| l.starts_with('❄')), "{}", app.docs[app.cur].ed.text());
        app.key(key('f'));
        app.key(key('l'));
        app.key(key('L'));
        assert_eq!(app.proced.as_ref().unwrap().sent, vec!["freeze 6816", "thaw 6816", "limit 6816 --preset background", "free 6816"]);
    }
}
