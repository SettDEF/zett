//! Suggestions: an agent's proposed text, shown beside the real text and taken or dropped one
//! hunk at a time (as suggestions in a shared document). The text is not changed until one is
//! taken; taking one is its own undo step. Each remembers the lines it replaces, so it follows
//! them through any edit, and goes away when they are gone.

use crate::Editor;

/// The suggestion commands (M-x, C-h f); keys point at them in the keymaps.
pub const COMMANDS: &[(&str, &str)] = &[
    ("agent-accept", "Take the suggestion at the cursor (or the next one)"),
    ("agent-reject", "Drop the suggestion at the cursor (or the next one)"),
    ("agent-accept-all", "Take every suggestion"),
    ("agent-reject-all", "Drop every suggestion"),
    ("agent-next-suggestion", "Move to the next suggestion"),
    ("agent-previous-suggestion", "Move to the previous suggestion"),
];

pub fn is_command(name: &str) -> bool {
    COMMANDS.iter().any(|(n, _)| *n == name)
}

/// One proposed change: lines `at..at + old.len()` become `new`.
#[derive(Clone, Debug, PartialEq)]
pub struct Hunk {
    pub at: usize,
    pub old: Vec<String>,
    pub new: Vec<String>,
    /// for an insertion (no old lines): the line it follows, to find its place again
    pub after: Option<String>,
    pub author: String,
}

impl Hunk {
    /// The lines it covers now (an insertion covers none, before `at`).
    pub fn end(&self) -> usize {
        self.at + self.old.len()
    }
}

/// The hunks that turn `old` into `new`, as line numbers from `base`. Longest common
/// subsequence over lines; past a size where that costs too much, one hunk for the lot.
pub fn diff(old: &[String], new: &[String], base: usize, author: &str) -> Vec<Hunk> {
    let (n, m) = (old.len(), new.len());
    let mut out = Vec::new();
    // equal lines at both ends need no table
    let pre = old.iter().zip(new).take_while(|(a, b)| a == b).count();
    let suf = old[pre..].iter().rev().zip(new[pre..].iter().rev()).take_while(|(a, b)| a == b).count();
    let (o, w) = (&old[pre..n - suf], &new[pre..m - suf]);
    if o.is_empty() && w.is_empty() {
        return out;
    }
    let hunk = |at: usize, old: &[String], new: &[String], all: &[String]| Hunk {
        at: base + at,
        old: old.to_vec(),
        new: new.to_vec(),
        after: if old.is_empty() && at > 0 { all.get(at - 1).cloned() } else { None },
        author: author.to_string(),
    };
    if o.len() * w.len() > 4_000_000 {
        out.push(hunk(pre, o, w, old));
        return out;
    }
    // lcs[i][j]: common lines of o[i..] and w[j..]
    let mut lcs = vec![vec![0u32; w.len() + 1]; o.len() + 1];
    for i in (0..o.len()).rev() {
        for j in (0..w.len()).rev() {
            lcs[i][j] = if o[i] == w[j] { lcs[i + 1][j + 1] + 1 } else { lcs[i + 1][j].max(lcs[i][j + 1]) };
        }
    }
    let (mut i, mut j) = (0, 0);
    let (mut hi, mut hj) = (0, 0);
    let flush = |i: usize, j: usize, hi: usize, hj: usize, out: &mut Vec<Hunk>| {
        if hi < i || hj < j {
            out.push(hunk(pre + hi, &o[hi..i], &w[hj..j], old));
        }
    };
    while i < o.len() || j < w.len() {
        if i < o.len() && j < w.len() && o[i] == w[j] {
            flush(i, j, hi, hj, &mut out);
            i += 1;
            j += 1;
            hi = i;
            hj = j;
        } else if j < w.len() && (i == o.len() || lcs[i][j + 1] >= lcs[i + 1][j]) {
            j += 1;
        } else {
            i += 1;
        }
    }
    flush(i, j, hi, hj, &mut out);
    out
}

/// A buffer's suggestions, kept in step with its text.
#[derive(Clone, Debug, Default)]
pub struct Pending {
    hunks: Vec<Hunk>,
    rev: Option<u64>,
}

impl Editor {
    fn lines_now(&self) -> Vec<String> {
        (0..self.buf.len_lines()).map(|i| self.buf.line(i).trim_end_matches('\n').to_string()).collect()
    }

    /// Proposes `new` for lines `start..end` (exclusive): the differences become suggestions.
    /// Returns how many.
    pub fn propose(&mut self, start: usize, end: usize, new: &str, author: &str) -> usize {
        let lines = self.lines_now();
        let end = end.min(lines.len());
        let new: Vec<String> = new.lines().map(String::from).collect();
        let hunks = diff(&lines[start..end], &new, start, author);
        let n = hunks.len();
        // a new answer for the same lines replaces what was still pending there
        self.pending.hunks.retain(|h| h.end() <= start || h.at >= end);
        self.pending.hunks.extend(hunks);
        self.pending.hunks.sort_by_key(|h| h.at);
        self.pending.rev = Some(self.buf.rev());
        n
    }

    /// The suggestions, placed on the text as it is now.
    pub fn suggestions(&mut self) -> &[Hunk] {
        if self.pending.rev != Some(self.buf.rev()) && !self.pending.hunks.is_empty() {
            let lines = self.lines_now();
            let found = |h: &Hunk| -> Option<usize> {
                let fits = |at: usize| {
                    if h.old.is_empty() {
                        at <= lines.len() && h.after.as_ref().is_none_or(|a| at > 0 && lines[at - 1] == *a)
                    } else {
                        at + h.old.len() <= lines.len() && lines[at..at + h.old.len()] == h.old[..]
                    }
                };
                (0..=lines.len()).filter(|&a| fits(a)).min_by_key(|a| a.abs_diff(h.at))
            };
            let mut kept: Vec<Hunk> = Vec::new();
            for mut h in std::mem::take(&mut self.pending.hunks) {
                if let Some(at) = found(&h) {
                    h.at = at;
                    kept.push(h);
                }
            }
            kept.sort_by_key(|h| h.at);
            self.pending.hunks = kept;
        }
        self.pending.rev = Some(self.buf.rev());
        &self.pending.hunks
    }

    /// The suggestion the cursor is on, else the next one after it, else the last.
    fn hunk_here(&mut self) -> Option<usize> {
        let line = self.cursor().line;
        let hs = self.suggestions();
        hs.iter().position(|h| (h.at <= line && line < h.end()) || h.at == line).or_else(|| hs.iter().position(|h| h.at > line)).or_else(|| hs.len().checked_sub(1))
    }

    fn take(&mut self, i: usize) {
        let h = self.pending.hunks.remove(i);
        // past the last line is the end of the text (line_start would clamp onto the last line)
        let a = if h.at >= self.buf.len_lines() { self.buf.len_chars() } else { self.line_start(h.at) };
        let b = if h.end() >= self.buf.len_lines() { self.buf.len_chars() } else { self.line_start(h.end()) };
        let at_end = b == self.buf.len_chars() && !self.buf.slice(a, b).ends_with('\n') && h.end() > h.at;
        let mut text: String = h.new.iter().map(|l| format!("{l}\n")).collect();
        // an insertion at the very end, after a last line without a break, needs one before it
        if h.old.is_empty() && a == self.buf.len_chars() && a > 0 && !self.buf.slice(a - 1, a).ends_with('\n') {
            text.insert(0, '\n');
            text.pop();
        } else if at_end {
            text.pop();
        }
        self.begin();
        self.rem(a, b);
        self.ins(a, &text);
        self.close();
        self.cursor = a.min(self.buf.len_chars());
    }

    /// Runs a suggestion command. False: not one.
    pub(crate) fn suggest_exec(&mut self, name: &str) -> bool {
        if !is_command(name) {
            return false;
        }
        match name {
            "agent-accept" => {
                if let Some(i) = self.hunk_here() {
                    self.take(i);
                }
            }
            "agent-reject" => {
                if let Some(i) = self.hunk_here() {
                    let at = self.pending.hunks.remove(i).at;
                    self.cursor = self.line_start(at.min(self.buf.len_lines().saturating_sub(1)));
                }
            }
            "agent-accept-all" => {
                // from the bottom, so the line numbers above stay right
                while !self.suggestions().is_empty() {
                    let last = self.pending.hunks.len() - 1;
                    self.take(last);
                }
            }
            "agent-reject-all" => self.pending.hunks.clear(),
            "agent-next-suggestion" | "agent-previous-suggestion" => {
                let line = self.cursor().line;
                let hs = self.suggestions();
                let to = if name == "agent-next-suggestion" { hs.iter().find(|h| h.at > line) } else { hs.iter().rev().find(|h| h.at < line) }.map(|h| h.at);
                if let Some(l) = to {
                    self.cursor = self.line_start(l.min(self.buf.len_lines().saturating_sub(1)));
                }
            }
            _ => {}
        }
        self.want_col = self.cursor().col;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ls(s: &str) -> Vec<String> {
        s.lines().map(String::from).collect()
    }

    #[test]
    fn diff_splits_into_separate_hunks() {
        let h = diff(&ls("a\nb\nc\nd\ne"), &ls("a\nB\nc\nd\ne\nf"), 10, "Haiku");
        assert_eq!(h.len(), 2, "{h:?}");
        assert_eq!((h[0].at, h[0].old.clone(), h[0].new.clone()), (11, ls("b"), ls("B")));
        assert_eq!((h[1].at, h[1].old.len(), h[1].new.clone(), h[1].after.clone()), (15, 0, ls("f"), Some("e".into())));
        assert!(diff(&ls("same"), &ls("same"), 0, "x").is_empty());
    }

    fn ed(t: &str) -> Editor {
        let mut e = Editor::from_str(t);
        e.set_emacs(true);
        e
    }

    #[test]
    fn nothing_changes_until_one_is_taken() {
        let mut e = ed("one\ntwo\nthree\n");
        assert_eq!(e.propose(0, 3, "one\n2\nthree\nfour", "Haiku"), 2);
        assert_eq!(e.text(), "one\ntwo\nthree\n");
        e.set_cursor(crate::Pos { line: 1, col: 0 });
        e.exec("agent-accept", 1);
        assert_eq!(e.text(), "one\n2\nthree\n");
        assert_eq!(e.suggestions().len(), 1);
        e.exec("agent-reject", 1);
        assert!(e.suggestions().is_empty());
        assert_eq!(e.text(), "one\n2\nthree\n");
        e.exec("undo", 1);
        assert_eq!(e.text(), "one\ntwo\nthree\n", "taking one is one undo step");
    }

    #[test]
    fn suggestions_follow_their_lines_through_edits() {
        let mut e = ed("a\nb\nc\n");
        e.propose(0, 3, "a\nB\nc", "Haiku");
        // a line typed above moves the suggestion down with b
        e.set_cursor(crate::Pos { line: 0, col: 0 });
        e.exec("open-line", 1);
        assert_eq!(e.suggestions()[0].at, 2);
        e.exec("agent-accept-all", 1);
        assert_eq!(e.text(), "\na\nB\nc\n");
        // the lines it replaced are gone: so is the suggestion
        let mut g = ed("a\nb\n");
        g.propose(0, 2, "a\nB", "Haiku");
        g.set_cursor(crate::Pos { line: 1, col: 0 });
        g.exec("kill-line", 1);
        assert!(g.suggestions().is_empty());
    }

    #[test]
    fn accept_all_and_an_insertion_at_the_end() {
        let mut e = ed("x\ny");
        e.propose(0, 2, "X\ny\nz", "Haiku");
        e.exec("agent-accept-all", 1);
        assert_eq!(e.text(), "X\ny\nz");
    }
}
