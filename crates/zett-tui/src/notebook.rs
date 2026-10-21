//! Notebook blocks, as Org's babel: C-c C-c in a fenced code block (```sh, python, js, lua, ruby,
//! fish, zsh) runs it in the note's folder, in the background with a 30 s limit, and its output is
//! drawn under the block; C-c C-v writes that output into the note as an ```output block under it
//! (one undo step; run again and write again replaces it). Output belongs to the code that made
//! it: change the code and it goes until the block runs again.

use super::*;
use std::sync::mpsc;

const LIMIT: Duration = Duration::from_secs(30);
const SHOWN: usize = 20;

/// The command a language runs its code with (the code comes last).
pub(crate) fn runner(lang: &str) -> Option<Vec<&'static str>> {
    Some(match lang {
        "sh" | "bash" | "shell" => vec!["bash", "-c"],
        "zsh" => vec!["zsh", "-c"],
        "fish" => vec!["fish", "-c"],
        "python" | "py" | "python3" => vec!["python3", "-c"],
        "js" | "javascript" | "node" => vec!["node", "-e"],
        "lua" => vec!["lua", "-e"],
        "ruby" | "rb" => vec!["ruby", "-e"],
        _ => return None,
    })
}

/// The fenced block a line is in (fences included): (open, close, language, code).
pub(crate) fn block_at(lines: &[String], at: usize) -> Option<(usize, usize, String, String)> {
    let mut open: Option<(usize, String)> = None;
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim_start();
        if !t.starts_with("```") {
            continue;
        }
        match open.take() {
            None => open = Some((i, t.trim_start_matches('`').trim().split_whitespace().next().unwrap_or("").to_lowercase())),
            Some((a, lang)) => {
                if (a..=i).contains(&at) {
                    return Some((a, i, lang, lines[a + 1..i].join("\n")));
                }
                if i > at {
                    return None;
                }
            }
        }
    }
    None
}

/// Every block with a language that runs: (open, close, language, code).
fn blocks(lines: &[String]) -> Vec<(usize, usize, String, String)> {
    let mut out = Vec::new();
    let mut open: Option<(usize, String)> = None;
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim_start();
        if !t.starts_with("```") {
            continue;
        }
        match open.take() {
            None => open = Some((i, t.trim_start_matches('`').trim().split_whitespace().next().unwrap_or("").to_lowercase())),
            Some((a, lang)) => {
                if runner(&lang).is_some() {
                    out.push((a, i, lang, lines[a + 1..i].join("\n")));
                }
            }
        }
    }
    out
}

#[derive(Clone, Debug)]
pub(crate) struct Output {
    pub lines: Vec<String>,
    pub code: Option<i32>,
    pub secs: f32,
}

/// Runs code to its end or the limit; stdout and stderr together, in order of arrival per stream.
pub(crate) fn run(lang: &str, code: &str, dir: &Path) -> Output {
    let started = Instant::now();
    let Some(cmd) = runner(lang) else {
        return Output { lines: vec![format!("{lang} does not run here")], code: None, secs: 0.0 };
    };
    let child = std::process::Command::new(cmd[0])
        .args(&cmd[1..])
        .arg(code)
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();
    let mut child = match child {
        Ok(c) => c,
        Err(e) => return Output { lines: vec![format!("{}: {e}", cmd[0])], code: None, secs: 0.0 },
    };
    let (mut out, mut err) = (child.stdout.take().unwrap(), child.stderr.take().unwrap());
    let ro = std::thread::spawn(move || { let mut s = String::new(); let _ = std::io::Read::read_to_string(&mut out, &mut s); s });
    let re = std::thread::spawn(move || { let mut s = String::new(); let _ = std::io::Read::read_to_string(&mut err, &mut s); s });
    let code = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st.code(),
            Ok(None) if started.elapsed() < LIMIT => std::thread::sleep(Duration::from_millis(20)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };
    let mut text = ro.join().unwrap_or_default();
    let e = re.join().unwrap_or_default();
    if !e.is_empty() {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&e);
    }
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    if code.is_none() && started.elapsed() >= LIMIT {
        lines.push(format!("(stopped after {} s)", LIMIT.as_secs()));
    }
    Output { lines, code, secs: started.elapsed().as_secs_f32() }
}

/// Runs in flight and what finished ones printed, by (buffer, language + code).
#[derive(Default)]
pub(crate) struct Notebook {
    pub done: std::collections::HashMap<(String, String), Output>,
    running: Vec<((String, String), mpsc::Receiver<Output>, Instant)>,
}

impl Notebook {
    pub fn busy(&self) -> bool {
        !self.running.is_empty()
    }

    /// The rows a buffer's blocks draw after their closing fence.
    pub fn rows(&self, doc: &str, lines: &[String]) -> Vec<(usize, Vec<Vec<(String, &'static str)>>)> {
        let mut out = Vec::new();
        for (_, close, lang, code) in blocks(lines) {
            let key = (doc.to_string(), format!("{lang}\n{code}"));
            if let Some((_, _, at)) = self.running.iter().find(|(k, _, _)| *k == key) {
                out.push((close, vec![vec![("┊ ".into(), "QueryHead"), (format!("⧗ running {lang} · {:.0} s · C-g stops waiting", at.elapsed().as_secs_f32()), "QueryHead")]]));
                continue;
            }
            let Some(o) = self.done.get(&key) else { continue };
            let status = match o.code { Some(0) => "exit 0".to_string(), Some(c) => format!("exit {c}"), None => "stopped".into() };
            let mut r = vec![vec![("┊ ".into(), "QueryHead"), (format!("▶ {lang} · {:.2} s · {status}", o.secs), if o.code == Some(0) { "NoteOk" } else { "Error" }), ("  C-c C-v / SPC m i writes it in".into(), "QueryHead")]];
            for l in o.lines.iter().take(SHOWN) {
                r.push(vec![("┊ ".into(), "QueryHead"), (l.replace('\t', "    "), "NoteOut")]);
            }
            if o.lines.len() > SHOWN {
                r.push(vec![("┊ ".into(), "QueryHead"), (format!("… {} more lines", o.lines.len() - SHOWN), "QueryHead")]);
            }
            if o.lines.is_empty() {
                r.push(vec![("┊ ".into(), "QueryHead"), ("(no output)".into(), "QueryHead")]);
            }
            out.push((close, r));
        }
        out
    }
}

impl App {
    fn block_here(&self) -> Option<(usize, usize, String, String)> {
        let d = &self.docs[self.cur];
        let lines: Vec<String> = (0..d.ed.buf.len_lines()).map(|i| d.ed.buf.line(i).trim_end_matches('\n').to_string()).collect();
        block_at(&lines, d.ed.cursor().line)
    }

    /// C-c C-c: runs the code block at the cursor.
    pub(crate) fn notebook_run(&mut self) {
        let Some((_, _, lang, code)) = self.block_here() else { return self.say("not in a code block") };
        if runner(&lang).is_none() {
            return self.say(format!("```{lang} does not run: sh, bash, zsh, fish, python, js, lua, ruby"));
        }
        let d = &self.docs[self.cur];
        let dir = std::fs::canonicalize(&d.name).ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or(self.root.clone());
        let key = (d.name.clone(), format!("{lang}\n{code}"));
        if self.notebook.running.iter().any(|(k, _, _)| *k == key) {
            return self.say("it is running already");
        }
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(run(&lang, &code, &dir));
        });
        self.notebook.running.push((key, rx, Instant::now()));
    }

    /// Every turn of the loop: takes the runs that finished.
    pub(crate) fn notebook_poll(&mut self) {
        let mut i = 0;
        while i < self.notebook.running.len() {
            match self.notebook.running[i].1.try_recv() {
                Ok(o) => {
                    let (key, _, _) = self.notebook.running.remove(i);
                    self.notebook.done.insert(key, o);
                }
                Err(mpsc::TryRecvError::Empty) => i += 1,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.notebook.running.remove(i);
                }
            }
        }
    }

    /// C-g: stops waiting for the runs (they finish or hit the limit on their own).
    pub(crate) fn notebook_drop(&mut self) -> bool {
        let had = self.notebook.busy();
        self.notebook.running.clear();
        had
    }

    /// C-c C-v: the block's output into the note, as an ```output block under it (replacing the one there).
    pub(crate) fn notebook_insert(&mut self) {
        let Some((_, close, lang, code)) = self.block_here() else { return self.say("not in a code block") };
        let d = &self.docs[self.cur];
        let Some(o) = self.notebook.done.get(&(d.name.clone(), format!("{lang}\n{code}"))).cloned() else {
            return self.say("run it first: C-c C-c");
        };
        let lines: Vec<String> = (0..d.ed.buf.len_lines()).map(|i| d.ed.buf.line(i).trim_end_matches('\n').to_string()).collect();
        let body = format!("```output\n{}{}```", o.lines.join("\n"), if o.lines.is_empty() { "" } else { "\n" });
        // an ```output block right under it (a blank line between allowed) is the last result: replaced
        let mut at = close + 1;
        while at < lines.len() && lines[at].trim().is_empty() {
            at += 1;
        }
        let old = (at < lines.len() && lines[at].trim_start().starts_with("```output")).then(|| (at, (at + 1..lines.len()).find(|&j| lines[j].trim_start().starts_with("```")))).and_then(|(a, b)| b.map(|b| (a, b)));
        let ed = &mut self.doc().ed;
        match old {
            Some((a, b)) => {
                let end = Pos { line: b, col: lines[b].chars().count() };
                ed.apply_edits(vec![(Pos { line: a, col: 0 }, end, body)]);
            }
            None => {
                let end = Pos { line: close, col: lines[close].chars().count() };
                ed.apply_edits(vec![(end, end, format!("\n\n{body}"))]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn finds_the_block_at_a_line() {
        let t = l(&["# x", "```sh", "echo hi", "```", "text", "```python extra", "print(1)", "```"]);
        assert_eq!(block_at(&t, 2), Some((1, 3, "sh".into(), "echo hi".into())));
        assert_eq!(block_at(&t, 1).map(|b| b.0), Some(1), "on its fence");
        assert_eq!(block_at(&t, 4), None, "between blocks");
        assert_eq!(block_at(&t, 6).map(|b| b.2), Some("python".into()));
        assert_eq!(blocks(&l(&["```query", "tasks", "```", "```sh", "ls", "```"])).len(), 1, "a query block does not run");
    }

    #[test]
    fn runs_code_and_keeps_its_output_and_exit() {
        let d = std::env::temp_dir();
        let o = run("sh", "echo one; echo two >&2; exit 3", &d);
        assert_eq!(o.lines, ["one", "two"]);
        assert_eq!(o.code, Some(3));
        assert_eq!(run("bash", "pwd", &d).lines, [d.canonicalize().unwrap().display().to_string()], "in the note's folder");
        assert!(run("cobol", "x", &d).lines[0].contains("does not run"));
    }
}
