//! Structure, the same for notes and code: Markdown headings, or code blocks by indentation and
//! brackets (as Emacs's outline and defun commands find them, with no grammar needed). One set of
//! commands works on both: move between sections, move, promote and demote a whole subtree, fold,
//! narrow. Folds and the narrowing remember their heading, so they follow it through any edit.

use crate::Editor;

/// How a buffer's structure is read.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OutlineKind {
    /// `#` headings (outside fenced code)
    Markdown,
    /// a line whose next lines are indented further starts a block (closing bracket included)
    #[default]
    Code,
}

impl OutlineKind {
    /// By the highlighter's name for the file type.
    pub fn for_syntax(name: &str) -> OutlineKind {
        if name.eq_ignore_ascii_case("markdown") || name.eq_ignore_ascii_case("multimarkdown") { OutlineKind::Markdown } else { OutlineKind::Code }
    }
}

/// A heading and everything under it: lines `start..end`.
#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    /// 1 = top level
    pub level: usize,
    pub start: usize,
    pub end: usize,
    pub title: String,
}

/// The outline commands (M-x, C-h f); keys point at them in the keymaps.
pub const COMMANDS: &[(&str, &str)] = &[
    ("outline-next-heading", "Move to the next heading (or block)"),
    ("outline-previous-heading", "Move to the previous heading (or block)"),
    ("outline-forward-same-level", "Move to the next heading at the same level"),
    ("outline-backward-same-level", "Move to the previous heading at the same level"),
    ("outline-up-heading", "Move to the heading this one is under"),
    ("outline-cycle", "Fold or unfold the section at the cursor"),
    ("outline-cycle-buffer", "Fold every top section, or unfold all"),
    ("outline-show-all", "Unfold everything"),
    ("outline-move-subtree-up", "Move the section, with what is under it, above the one before"),
    ("outline-move-subtree-down", "Move the section, with what is under it, below the one after"),
    ("outline-promote", "One level up: fewer #, or one indent less, for the whole section"),
    ("outline-demote", "One level down: more #, or one indent more, for the whole section"),
    ("outline-mark-subtree", "Select the section at the cursor"),
    ("narrow-to-subtree", "Show only the section at the cursor"),
    ("narrow-to-region", "Show only the lines of the region"),
    ("widen", "Show the whole buffer again"),
];

pub fn is_command(name: &str) -> bool {
    COMMANDS.iter().any(|(n, _)| *n == name)
}

fn indent(l: &str) -> usize {
    l.chars().take_while(|c| *c == ' ' || *c == '\t').map(|c| if c == '\t' { 4 } else { 1 }).sum()
}

fn blank(l: &str) -> bool {
    l.trim().is_empty()
}

/// A line that closes the block above it at its own indent: `}`, `end`, `fi`, `</div>` …
fn closer(l: &str) -> bool {
    let t = l.trim_start();
    t.starts_with(['}', ')', ']']) || t.starts_with("</") || ["end", "fi", "done", "esac", "endif", "endfunction"].iter().any(|w| t == *w || t.starts_with(&format!("{w} ")) || t.starts_with(&format!("{w};")) || t.starts_with(&format!("{w})")))
}

/// The heading level of a Markdown line (`## x` → 2).
fn md_level(l: &str) -> Option<usize> {
    let n = l.chars().take_while(|c| *c == '#').count();
    (1..=6).contains(&n).then_some(n).filter(|_| l[n..].starts_with([' ', '\t']) || l.len() == n)
}

/// Every section of the text, in order of their first lines.
pub fn sections(lines: &[String], kind: OutlineKind) -> Vec<Section> {
    match kind {
        OutlineKind::Markdown => markdown(lines),
        OutlineKind::Code => code(lines),
    }
}

fn markdown(lines: &[String]) -> Vec<Section> {
    let mut heads = Vec::new();
    let mut fence = false;
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            fence = !fence;
            continue;
        }
        if !fence {
            if let Some(n) = md_level(l) {
                heads.push((i, n));
            }
        }
    }
    heads
        .iter()
        .enumerate()
        .map(|(k, &(start, level))| {
            let end = heads[k + 1..].iter().find(|(_, l)| *l <= level).map_or(lines.len(), |(s, _)| *s);
            Section { level, start, end, title: lines[start].trim().to_string() }
        })
        .collect()
}

fn code(lines: &[String]) -> Vec<Section> {
    let next_text = |from: usize| (from..lines.len()).find(|&j| !blank(&lines[j]));
    let mut out: Vec<Section> = Vec::new();
    let mut open: Vec<usize> = Vec::new(); // indices into out of the blocks still open
    for (i, l) in lines.iter().enumerate() {
        if blank(l) {
            continue;
        }
        let k = indent(l);
        // blocks this line is not inside any more
        while let Some(&o) = open.last() {
            let ok = indent(&lines[out[o].start]);
            if k > ok {
                break;
            }
            open.pop();
            // a closer at the block's own indent belongs to it
            out[o].end = if k == ok && closer(l) { i + 1 } else { last_text(lines, i) + 1 };
        }
        if closer(l) && open.is_empty() && out.last().is_some_and(|s| s.end == i + 1) {
            continue;
        }
        if let Some(j) = next_text(i + 1) {
            if indent(&lines[j]) > k {
                out.push(Section { level: open.len() + 1, start: i, end: lines.len(), title: l.trim().to_string() });
                open.push(out.len() - 1);
            }
        }
    }
    let end = last_text(lines, lines.len()) + 1;
    for o in open {
        out[o].end = end;
    }
    out
}

/// The last non-blank line before `before`.
fn last_text(lines: &[String], before: usize) -> usize {
    (0..before).rev().find(|&j| !blank(&lines[j])).unwrap_or(0)
}

/// The innermost section `line` is in.
pub fn at(secs: &[Section], line: usize) -> Option<usize> {
    secs.iter().enumerate().filter(|(_, s)| s.start <= line && line < s.end).max_by_key(|(_, s)| s.level).map(|(i, _)| i)
}

/// A remembered heading: where it was and what it said, to find it again after edits.
#[derive(Clone, Debug, PartialEq)]
pub struct Mark {
    pub title: String,
    pub line: usize,
}

/// The section of `secs` that is `m` now: the same title, nearest to where it was.
fn find(secs: &[Section], m: &Mark) -> Option<usize> {
    secs.iter().enumerate().filter(|(_, s)| s.title == m.title).min_by_key(|(_, s)| s.start.abs_diff(m.line)).map(|(i, _)| i)
}

/// Folds and the narrowing of one buffer, kept in step with its text.
#[derive(Clone, Debug, Default)]
pub struct View {
    pub kind: OutlineKind,
    folds: Vec<Mark>,
    narrow: Option<(Mark, Option<Mark>)>,
    /// for the text at this revision: hidden line ranges and the narrowed range
    hidden: Vec<(usize, usize)>,
    range: Option<(usize, usize)>,
    rev: Option<u64>,
}

impl View {
    pub fn is_empty(&self) -> bool {
        self.folds.is_empty() && self.narrow.is_none()
    }
}

impl Editor {
    fn all_lines(&self) -> Vec<String> {
        (0..self.buf.len_lines()).map(|i| self.buf.line(i).trim_end_matches('\n').to_string()).collect()
    }

    pub fn sections(&self) -> Vec<Section> {
        sections(&self.all_lines(), self.view.kind)
    }

    pub fn view_kind(&self) -> OutlineKind {
        self.view.kind
    }

    /// Nothing folded, not narrowed.
    pub fn view_is_empty(&self) -> bool {
        self.view.is_empty()
    }

    pub fn set_outline_kind(&mut self, kind: OutlineKind) {
        if self.view.kind != kind {
            self.view.kind = kind;
            self.view.rev = None;
        }
    }

    /// Brings folds and the narrowing up to date with the text (after any edit, however made).
    pub fn refresh_view(&mut self) {
        if self.view.rev == Some(self.buf.rev()) {
            return;
        }
        self.view.rev = Some(self.buf.rev());
        if self.view.is_empty() && self.vim.folds.is_empty() {
            self.view.hidden.clear();
            self.view.range = None;
            return;
        }
        let secs = self.sections();
        let mut kept = Vec::new();
        let mut hidden = Vec::new();
        for m in std::mem::take(&mut self.view.folds) {
            if let Some(i) = find(&secs, &m) {
                let s = &secs[i];
                if s.end > s.start + 1 {
                    hidden.push((s.start + 1, s.end));
                    kept.push(Mark { title: m.title, line: s.start });
                }
            }
        }
        self.view.folds = kept;
        // Vim's manual folds (vim/folds.rs) hide lines too
        hidden.extend(self.manual_hidden());
        hidden.sort();
        self.view.hidden = hidden;
        self.view.range = match self.view.narrow.take() {
            None => None,
            // a section: follows its heading
            Some((m, None)) => find(&secs, &m).map(|i| {
                self.view.narrow = Some((Mark { title: m.title, line: secs[i].start }, None));
                (secs[i].start, secs[i].end)
            }),
            // a region: from one line to another, by their text
            Some((a, Some(b))) => {
                let lines = self.all_lines();
                let near = |m: &Mark| (0..lines.len()).filter(|&i| lines[i].trim() == m.title).min_by_key(|i| i.abs_diff(m.line));
                match (near(&a), near(&b)) {
                    (Some(x), Some(y)) if y >= x => {
                        self.view.narrow = Some((Mark { line: x, ..a }, Some(Mark { line: y, ..b })));
                        Some((x, y + 1))
                    }
                    _ => None,
                }
            }
        };
    }

    /// Folds changed outside the outline (Vim's zf zo zc …): the hidden lines are worked out again.
    pub(crate) fn view_touch(&mut self) {
        self.view.rev = None;
        self.refresh_view();
    }

    /// Lines folded away or outside the narrowing are not shown and not moved through.
    pub fn is_hidden(&self, line: usize) -> bool {
        if let Some((a, b)) = self.view.range {
            if line < a || line >= b {
                return true;
            }
        }
        self.view.hidden.iter().any(|&(a, b)| line >= a && line < b)
    }

    /// The line starts a folded section (drawn with a … after it).
    pub fn is_fold_start(&self, line: usize) -> bool {
        self.view.hidden.iter().any(|&(a, _)| a == line + 1)
    }

    /// The narrowed lines, if narrowed.
    pub fn narrowed(&self) -> Option<(usize, usize)> {
        self.view.range
    }

    /// From `line`, `n` shown lines on (or back), skipping hidden ones.
    pub fn step_shown(&self, line: usize, n: usize, back: bool) -> usize {
        let last = self.buf.len_lines().saturating_sub(1);
        let (mut l, mut left) = (line, n);
        while left > 0 {
            let next = if back { l.checked_sub(1) } else { (l < last).then_some(l + 1) };
            let Some(mut x) = next else { break };
            while self.is_hidden(x) {
                match if back { x.checked_sub(1) } else { (x < last).then_some(x + 1) } {
                    Some(y) => x = y,
                    None => return l,
                }
            }
            l = x;
            left -= 1;
        }
        l
    }

    /// The cursor never rests on a hidden line: unfold what hides it (as isearch reveals).
    pub fn reveal(&mut self) {
        self.refresh_view();
        let line = self.cursor().line;
        if !self.is_hidden(line) {
            return;
        }
        if let Some((a, b)) = self.view.range {
            if line < a || line >= b {
                self.view.narrow = None;
            }
        }
        let secs = self.sections();
        self.view.folds.retain(|m| find(&secs, m).is_none_or(|i| !(secs[i].start < line && line < secs[i].end)));
        self.view.rev = None;
        self.refresh_view();
    }

    fn goto_line_start(&mut self, line: usize) {
        self.cursor = self.line_start(line);
    }

    /// Runs an outline command `n` times. False: not one.
    pub(crate) fn outline_exec(&mut self, name: &str, n: usize) -> bool {
        if !is_command(name) {
            return false;
        }
        self.refresh_view();
        for _ in 0..n.max(1) {
            let secs = self.sections();
            let line = self.cursor().line;
            let here = at(&secs, line);
            let shown = |e: &Editor, s: &Section| !e.is_hidden(s.start);
            match name {
                "outline-next-heading" => {
                    if let Some(s) = secs.iter().find(|s| s.start > line && shown(self, s)) {
                        self.goto_line_start(s.start);
                    }
                }
                "outline-previous-heading" => {
                    if let Some(s) = secs.iter().rev().find(|s| s.start < line && shown(self, s)) {
                        self.goto_line_start(s.start);
                    }
                }
                "outline-forward-same-level" | "outline-backward-same-level" => {
                    let Some(h) = here else { continue };
                    let me = &secs[h];
                    let parent = parent(&secs, h);
                    let sib = |s: &Section| s.level == me.level && parent.is_none_or(|p| secs[p].start < s.start && s.start < secs[p].end);
                    let found = if name == "outline-forward-same-level" { secs.iter().find(|s| s.start >= me.end && sib(s)) } else { secs.iter().rev().find(|s| s.end <= me.start && sib(s)) };
                    if let Some(s) = found {
                        self.goto_line_start(s.start);
                    }
                }
                "outline-up-heading" => {
                    let target = match here {
                        // on a heading: its parent; inside a section: its heading
                        Some(h) if secs[h].start == line => parent(&secs, h),
                        h => h,
                    };
                    if let Some(p) = target {
                        self.goto_line_start(secs[p].start);
                    }
                }
                "outline-cycle" => {
                    let Some(h) = here else { continue };
                    let s = &secs[h];
                    let m = Mark { title: s.title.clone(), line: s.start };
                    match self.view.folds.iter().position(|f| find(&secs, f) == Some(h)) {
                        Some(i) => {
                            self.view.folds.remove(i);
                        }
                        None => {
                            self.view.folds.push(m);
                            self.goto_line_start(s.start);
                        }
                    }
                    self.view.rev = None;
                }
                "outline-cycle-buffer" => {
                    if self.view.folds.is_empty() {
                        let top = secs.iter().map(|s| s.level).min().unwrap_or(1);
                        self.view.folds = secs.iter().filter(|s| s.level == top).map(|s| Mark { title: s.title.clone(), line: s.start }).collect();
                        if let Some(h) = secs.iter().filter(|s| s.level == top && s.start <= line).last() {
                            self.goto_line_start(h.start);
                        }
                    } else {
                        self.view.folds.clear();
                    }
                    self.view.rev = None;
                }
                "outline-show-all" => {
                    self.view.folds.clear();
                    self.view.rev = None;
                }
                "outline-move-subtree-up" | "outline-move-subtree-down" => {
                    let Some(h) = here else { continue };
                    let me = secs[h].clone();
                    let parent = parent(&secs, h);
                    let sib = |s: &Section| s.level == me.level && parent.is_none_or(|p| secs[p].start < s.start && s.start < secs[p].end);
                    let other = if name.ends_with("up") { secs.iter().rev().find(|s| s.end <= me.start && sib(s)) } else { secs.iter().find(|s| s.start >= me.end && sib(s)) };
                    let Some(o) = other.cloned() else { continue };
                    let col = self.cursor().col;
                    let offset = line - me.start;
                    let (first, second) = if o.start < me.start { (o, me.clone()) } else { (me.clone(), o) };
                    // swap two neighbours: second's lines, what lies between, first's lines
                    let span = |a: usize, b: usize| (self.line_start(a), if b >= self.buf.len_lines() { self.buf.len_chars() } else { self.line_start(b) });
                    let (fa, _) = span(first.start, first.end);
                    let (_, sb) = span(second.start, second.end);
                    let mut whole = self.buf.slice(fa, sb);
                    let ends_open = !whole.ends_with('\n');
                    if ends_open {
                        whole.push('\n');
                    }
                    let lines: Vec<&str> = whole.split_inclusive('\n').collect();
                    let n1 = first.end - first.start;
                    let gap = second.start - first.end;
                    let (one, rest) = lines.split_at(n1);
                    let (mid, two) = rest.split_at(gap);
                    let mut new: String = [two, mid, one].concat().concat();
                    if ends_open {
                        new.pop();
                    }
                    self.begin();
                    self.rem(fa, sb);
                    self.ins(fa, &new);
                    self.close();
                    // the cursor stays in the section it was in
                    let me_new = if first.start == me.start { first.start + (second.end - second.start) + gap } else { first.start };
                    let l = me_new + offset;
                    self.cursor = (self.line_start(l) + col).min(self.line_end(l));
                    self.view.rev = None;
                }
                "outline-promote" | "outline-demote" => {
                    let Some(h) = here else { continue };
                    let s = secs[h].clone();
                    let up = name == "outline-promote";
                    // measured once: the lines move while it runs
                    let unit = indent_unit(&secs, &s, self);
                    let col = self.cursor().col;
                    self.begin();
                    for l in s.start..s.end {
                        let text = self.buf.line(l);
                        let text = text.trim_end_matches('\n');
                        let ls = self.line_start(l);
                        match self.view.kind {
                            OutlineKind::Markdown => {
                                let Some(lv) = md_level(text) else { continue };
                                if up && lv > 1 {
                                    self.rem(ls, ls + 1);
                                } else if !up && lv < 6 {
                                    self.ins(ls, "#");
                                }
                            }
                            OutlineKind::Code => {
                                if blank(text) {
                                    continue;
                                }
                                if up {
                                    let k = text.chars().take(unit).take_while(|c| *c == ' ' || *c == '\t').count();
                                    let k = if text.starts_with('\t') { 1 } else { k };
                                    if k > 0 {
                                        self.rem(ls, ls + k);
                                    }
                                } else {
                                    let pad = if text.starts_with('\t') { "\t".to_string() } else { " ".repeat(unit) };
                                    self.ins(ls, &pad);
                                }
                            }
                        }
                    }
                    self.close();
                    let l = line;
                    self.cursor = (self.line_start(l) + col).min(self.line_end(l));
                    self.view.rev = None;
                }
                "outline-mark-subtree" => {
                    let Some(h) = here else { continue };
                    let s = &secs[h];
                    let end = if s.end >= self.buf.len_lines() { self.buf.len_chars() } else { self.line_start(s.end) };
                    self.cursor = end;
                    self.set_mark(true);
                    self.goto_line_start(s.start);
                }
                "narrow-to-subtree" => {
                    if let Some(h) = here {
                        self.view.narrow = Some((Mark { title: secs[h].title.clone(), line: secs[h].start }, None));
                        self.view.rev = None;
                    }
                }
                "narrow-to-region" => {
                    if let Some((a, b)) = self.mark_span() {
                        let (la, lb) = (self.buf.to_pos(a).line, self.buf.to_pos(b.saturating_sub(1).max(a)).line);
                        let mark = |l: usize| Mark { title: self.buf.line(l).trim().to_string(), line: l };
                        self.view.narrow = Some((mark(la), Some(mark(lb))));
                        self.mark_active = false;
                        self.view.rev = None;
                    }
                }
                "widen" => {
                    self.view.narrow = None;
                    self.view.rev = None;
                }
                _ => {}
            }
        }
        // a change of structure can put the cursor inside a fold (a demoted heading joins the
        // folded section above it): open what hides it
        self.reveal();
        self.want_col = self.cursor().col;
        true
    }
}

/// The section `h` is directly under.
fn parent(secs: &[Section], h: usize) -> Option<usize> {
    let me = &secs[h];
    secs.iter().enumerate().filter(|(_, s)| s.level < me.level && s.start < me.start && me.start < s.end).max_by_key(|(_, s)| s.level).map(|(i, _)| i)
}

/// One step of indentation in this code: how much the section's first child is indented
/// past it, else 4.
fn indent_unit(secs: &[Section], s: &Section, e: &Editor) -> usize {
    let base = indent(&e.buf.line(s.start));
    (s.start + 1..s.end).map(|l| e.buf.line(l)).find(|l| !blank(l)).map(|l| indent(&l).saturating_sub(base)).filter(|u| *u > 0).or_else(|| secs.first().map(|_| 4)).unwrap_or(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<String> {
        s.lines().map(String::from).collect()
    }

    #[test]
    fn markdown_headings_nest_and_skip_code() {
        let t = lines("# A\ntext\n## A1\n```\n# not a heading\n```\n## A2\n# B\n");
        let s = sections(&t, OutlineKind::Markdown);
        let got: Vec<(usize, usize, usize)> = s.iter().map(|s| (s.level, s.start, s.end)).collect();
        assert_eq!(got, [(1, 0, 7), (2, 2, 6), (2, 6, 7), (1, 7, 8)]);
    }

    #[test]
    fn code_blocks_by_indent_with_their_closer() {
        let t = lines("fn a() {\n    if x {\n        y();\n    }\n}\n\nfn b() {\n    z();\n}\n");
        let s = sections(&t, OutlineKind::Code);
        let got: Vec<(usize, usize, usize)> = s.iter().map(|s| (s.level, s.start, s.end)).collect();
        assert_eq!(got, [(1, 0, 5), (2, 1, 4), (1, 6, 9)]);
    }

    #[test]
    fn python_blocks_end_at_their_last_line() {
        let t = lines("def a():\n    x = 1\n\n    return x\n\ndef b():\n    pass\n");
        let s = sections(&t, OutlineKind::Code);
        let got: Vec<(usize, usize)> = s.iter().map(|s| (s.start, s.end)).collect();
        assert_eq!(got, [(0, 4), (5, 7)]);
    }

    fn ed(text: &str, kind: OutlineKind) -> Editor {
        let mut e = Editor::from_str(text);
        e.set_emacs(true);
        e.set_outline_kind(kind);
        e
    }

    #[test]
    fn moving_between_headings() {
        let mut e = ed("# A\na\n## A1\nx\n## A2\ny\n# B\n", OutlineKind::Markdown);
        e.exec("outline-next-heading", 1);
        assert_eq!(e.cursor().line, 2);
        e.exec("outline-forward-same-level", 1);
        assert_eq!(e.cursor().line, 4, "A1 to A2");
        e.exec("outline-up-heading", 1);
        assert_eq!(e.cursor().line, 0);
        e.exec("outline-forward-same-level", 1);
        assert_eq!(e.cursor().line, 6, "A to B, past A's children");
    }

    #[test]
    fn a_subtree_moves_whole() {
        let mut e = ed("# A\na\n# B\nb\n## B1\nc\n", OutlineKind::Markdown);
        e.set_cursor(crate::Pos { line: 3, col: 0 });
        e.exec("outline-move-subtree-up", 1);
        assert_eq!(e.text(), "# B\nb\n## B1\nc\n# A\na\n");
        assert_eq!(e.cursor().line, 1, "the cursor stays on its line");
        e.exec("undo", 1);
        assert_eq!(e.text(), "# A\na\n# B\nb\n## B1\nc\n", "one undo step");
    }

    #[test]
    fn code_moves_by_block_too() {
        let mut e = ed("fn a() {\n    1\n}\nfn b() {\n    2\n}\n", OutlineKind::Code);
        e.exec("outline-move-subtree-down", 1);
        assert_eq!(e.text(), "fn b() {\n    2\n}\nfn a() {\n    1\n}\n");
    }

    #[test]
    fn promote_and_demote_the_whole_subtree() {
        let mut e = ed("## A\nx\n### A1\n", OutlineKind::Markdown);
        e.exec("outline-promote", 1);
        assert_eq!(e.text(), "# A\nx\n## A1\n");
        e.exec("outline-demote", 1);
        assert_eq!(e.text(), "## A\nx\n### A1\n");
        let mut c = ed("if x:\n  y\n  z\n", OutlineKind::Code);
        c.exec("outline-demote", 1);
        assert_eq!(c.text(), "  if x:\n    y\n    z\n", "the child's indent step");
    }

    #[test]
    fn folds_hide_and_follow_their_heading() {
        let mut e = ed("# A\na1\na2\n# B\nb\n", OutlineKind::Markdown);
        e.exec("outline-cycle", 1);
        assert!(e.is_hidden(1) && e.is_hidden(2) && !e.is_hidden(3));
        assert!(e.is_fold_start(0));
        // a line typed above moves the fold with A
        e.set_cursor(crate::Pos { line: 0, col: 0 });
        e.exec("open-line", 1);
        e.refresh_view();
        assert!(!e.is_hidden(1) && e.is_hidden(2) && e.is_hidden(3) && !e.is_hidden(4), "{:?}", e.text());
        // next-line skips the folded lines
        e.set_cursor(crate::Pos { line: 1, col: 0 });
        e.exec("next-line", 1);
        assert_eq!(e.cursor().line, 4);
    }

    #[test]
    fn the_cursor_never_ends_up_hidden() {
        let mut e = ed("# A\na\n# B\nb\n", OutlineKind::Markdown);
        e.exec("outline-cycle-buffer", 1);
        e.exec("outline-next-heading", 1);
        assert_eq!(e.cursor().line, 2);
        // B becomes a child of the folded A: A opens rather than hiding the cursor
        e.exec("outline-demote", 1);
        assert_eq!(e.text(), "# A\na\n## B\nb\n");
        e.refresh_view(); // as the screen does before it draws
        assert!(!e.is_hidden(e.cursor().line));
    }

    #[test]
    fn narrowing_shows_one_section() {
        let mut e = ed("# A\na\n# B\nb\n", OutlineKind::Markdown);
        e.set_cursor(crate::Pos { line: 3, col: 0 });
        e.exec("narrow-to-subtree", 1);
        assert_eq!(e.narrowed(), Some((2, 4)));
        assert!(e.is_hidden(0) && !e.is_hidden(2));
        e.exec("widen", 1);
        assert_eq!(e.narrowed(), None);
    }
}
