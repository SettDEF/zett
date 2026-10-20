//! The agent as a collaborator: ask it something about the region (or the section the cursor is
//! in), it answers with the text as it would write it, and the differences come back as
//! suggestions to take or drop one at a time. Runs `claude -p` (Haiku by default) in the
//! background: no tools, no session, just the text and the instruction.

use super::*;
use std::sync::mpsc;
use zett_edit::outline;

const SYSTEM: &str = "You are an editor working inside a text editor, beside its user. Apply the \
instruction to the text. Reply with only the complete revised text inside <text></text> and \
nothing else. Keep everything you were not asked to change exactly as it is: the same lines, \
indentation and line breaks.";

/// A question on its way.
pub(crate) struct Job {
    rx: mpsc::Receiver<Result<String, String>>,
    doc: usize,
    start: usize,
    end: usize,
    started: Instant,
    pub model: String,
}

impl App {
    /// The model and the command (`zett.opt.agent_model`, `zett.opt.agent_command`).
    fn agent_conf(&self) -> (String, String) {
        let model = self.engine.opt("agent_model").filter(|s| !s.is_empty()).unwrap_or_else(|| "haiku".into());
        let cmd = self.engine.opt("agent_command").filter(|s| !s.is_empty()).unwrap_or_else(|| {
            let local = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/bin/claude");
            if local.exists() { local.display().to_string() } else { "claude".into() }
        });
        (model, cmd)
    }

    /// The name shown on its suggestions: Haiku, Sonnet …
    pub(crate) fn agent_name(&self) -> String {
        let (model, _) = self.agent_conf();
        let m = model.trim_start_matches("claude-");
        let first = m.split(['-', ' ']).next().unwrap_or(m);
        let mut c = first.chars();
        c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or(model)
    }

    /// `C-c a`: ask in the echo area.
    pub(crate) fn agent_ask(&mut self) {
        if self.agent.is_some() {
            return self.say(format!("{} is still answering (C-g drops it)", self.agent_name()));
        }
        self.prompt = Some(format!("Ask {}: ", self.agent_name()));
        self.asking = true;
        self.cmdline = Some(String::new());
    }

    /// The lines the question is about: the region, else the section at the cursor, else all.
    fn agent_scope(&mut self) -> (usize, usize) {
        let ed = &mut self.docs[self.cur].ed;
        if let Some((a, b)) = ed.selection_range().filter(|(a, b)| b > a) {
            let (la, lb) = (ed.buf.to_pos(a).line, ed.buf.to_pos(b.saturating_sub(1)).line);
            return (la, lb + 1);
        }
        let secs = ed.sections();
        match outline::at(&secs, ed.cursor().line) {
            Some(i) => (secs[i].start, secs[i].end),
            None => (0, ed.buf.len_lines()),
        }
    }

    pub(crate) fn agent_start(&mut self, ask: &str) {
        let ask = ask.trim();
        if ask.is_empty() {
            return;
        }
        let (start, end) = self.agent_scope();
        let d = &self.docs[self.cur];
        // one line each, with its break (buf.line may or may not end in one)
        let text = (start..end.min(d.ed.buf.len_lines())).map(|l| d.ed.buf.line(l).trim_end_matches('\n').to_string()).collect::<Vec<_>>().join("\n");
        let file = Path::new(&d.name).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let prompt = format!("File: {file} ({})\nInstruction: {ask}\n\n<text>\n{}\n</text>\n", d.syntax, text.trim_end_matches('\n'));
        let (model, cmd) = self.agent_conf();
        let (tx, rx) = mpsc::channel();
        let m = model.clone();
        std::thread::spawn(move || {
            let _ = tx.send(run(&cmd, &m, &prompt));
        });
        self.agent = Some(Job { rx, doc: self.cur, start, end, started: Instant::now(), model });
    }

    /// Called every turn of the loop: takes the answer when it has come.
    pub(crate) fn agent_poll(&mut self) {
        let Some(job) = &self.agent else { return };
        let got = match job.rx.try_recv() {
            Ok(r) => r,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => Err("the agent stopped without an answer".into()),
        };
        let Some(job) = self.agent.take() else { return };
        let name = self.agent_name();
        match got.and_then(|out| answer(&out).ok_or_else(|| format!("{name} did not answer with text: {}", out.chars().take(120).collect::<String>()))) {
            Err(e) => self.say(e),
            Ok(new) if job.doc < self.docs.len() => {
                let n = self.docs[job.doc].ed.propose(job.start, job.end, &new, &name);
                let s = job.started.elapsed().as_secs_f32();
                self.say(match n {
                    0 => format!("{name} would change nothing ({s:.1} s)"),
                    n => format!("{name}: {n} suggestion{} ({s:.1} s) · C-c y take · C-c n drop · C-c ] next · C-c Y all", if n == 1 { "" } else { "s" }),
                });
            }
            Ok(_) => {}
        }
    }

    /// While it thinks: the echo area says so.
    pub(crate) fn agent_status(&self) -> Option<String> {
        let j = self.agent.as_ref()?;
        let dots = ["", ".", "..", "..."][(j.started.elapsed().as_millis() / 400 % 4) as usize];
        Some(format!("{} ({}) is thinking{dots} {:.0} s · C-g drops it", self.agent_name(), j.model, j.started.elapsed().as_secs_f32()))
    }
}

/// Runs the agent once: the prompt on stdin, the answer on stdout.
fn run(cmd: &str, model: &str, prompt: &str) -> Result<String, String> {
    use std::io::Write;
    let mut child = std::process::Command::new(cmd)
        .args(["-p", "--model", model, "--tools", "", "--strict-mcp-config", "--no-session-persistence", "--setting-sources", "", "--output-format", "text", "--system-prompt", SYSTEM])
        .current_dir(std::env::temp_dir())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("{cmd}: {e} (zett.opt.agent_command names another)"))?;
    if let Some(mut i) = child.stdin.take() {
        let _ = i.write_all(prompt.as_bytes());
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("{cmd}: {}", err.lines().last().unwrap_or("failed")));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// The text between <text> and </text>, without the line breaks right inside the tags.
fn answer(out: &str) -> Option<String> {
    let a = out.find("<text>")? + "<text>".len();
    let b = out.rfind("</text>")?;
    let inner = out.get(a..b)?;
    let inner = inner.strip_prefix('\n').unwrap_or(inner);
    Some(inner.strip_suffix('\n').unwrap_or(inner).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_answer_is_what_is_inside_the_tags() {
        assert_eq!(answer("<text>\nThe quick fox.\n</text>").as_deref(), Some("The quick fox."));
        assert_eq!(answer("Sure!\n<text>\na\n\nb\n</text>\n").as_deref(), Some("a\n\nb"));
        assert_eq!(answer("no tags"), None);
    }
}
