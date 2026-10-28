//! CodeDoc: the code editor's own core (no Qt TextEdit): a rope, any number of cursors, undo, folds by indent,
//! and syntect colours per line from parse states kept every few lines. CodeView.qml draws only the rows on screen.
#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, path)]
        /// rows drawn: lines minus the folded ones
        #[qproperty(i32, rows)]
        #[qproperty(i32, lines)]
        /// bumps on every change: QML bindings read it to redraw
        #[qproperty(i32, version)]
        #[qproperty(bool, dirty)]
        /// the cursors as JSON: [[line, col, anchor line, anchor col], …], the main one first
        #[qproperty(QString, cursors)]
        type CodeDoc = super::CodeDocRust;

        #[qinvokable]
        fn open(self: Pin<&mut CodeDoc>, path: &QString) -> bool;
        #[qinvokable]
        fn save(self: Pin<&mut CodeDoc>) -> bool;
        #[qinvokable]
        fn text(self: &CodeDoc) -> QString;
        /// row → line and back, with folds
        #[qinvokable]
        #[cxx_name = "rowLine"]
        fn row_line(self: &CodeDoc, row: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "lineRow"]
        fn line_row(self: &CodeDoc, line: i32) -> i32;
        /// a line as rich text (StyledText), tabs as four spaces
        #[qinvokable]
        #[cxx_name = "lineHtml"]
        fn line_html(self: Pin<&mut CodeDoc>, line: i32) -> QString;
        /// 0: nothing to fold, 1: foldable, 2: folded
        #[qinvokable]
        #[cxx_name = "foldAt"]
        fn fold_at(self: &CodeDoc, line: i32) -> i32;
        #[qinvokable]
        #[cxx_name = "toggleFold"]
        fn toggle_fold(self: Pin<&mut CodeDoc>, line: i32);
        /// a key: Qt's key code, its text, its modifiers; true when the editor took it
        #[qinvokable]
        fn key(self: Pin<&mut CodeDoc>, key: i32, text: &QString, mods: i32) -> bool;
        /// a click at a line and column: shift extends, alt adds a cursor; drag extends the main one
        #[qinvokable]
        fn click(self: Pin<&mut CodeDoc>, line: i32, col: i32, shift: bool, alt: bool);
        #[qinvokable]
        fn drag(self: Pin<&mut CodeDoc>, line: i32, col: i32);
        #[qinvokable]
        fn insert(self: Pin<&mut CodeDoc>, text: &QString);
        #[qinvokable]
        #[cxx_name = "selectedText"]
        fn selected_text(self: &CodeDoc) -> QString;
    }
}

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use ropey::Rope;
use syntect::highlighting::{HighlightState, Highlighter, RangedHighlightIterator};
use syntect::parsing::{ParseState, ScopeStack};

const KEEP: usize = 32; // a parse state kept every KEEP lines

#[derive(Clone, Copy, Debug, PartialEq)]
struct Caret {
    head: usize,
    anchor: usize,
    goal: Option<usize>,
}

impl Caret {
    fn at(p: usize) -> Self {
        Caret { head: p, anchor: p, goal: None }
    }
    fn range(&self) -> (usize, usize) {
        (self.head.min(self.anchor), self.head.max(self.anchor))
    }
}

#[derive(Clone)]
struct Snap {
    rope: Rope,
    carets: Vec<Caret>,
}

pub struct CodeDocRust {
    path: QString,
    rows: i32,
    lines: i32,
    version: i32,
    dirty: bool,
    cursors: QString,
    rope: Rope,
    saved: Rope,
    carets: Vec<Caret>,
    undo: Vec<Snap>,
    redo: Vec<Snap>,
    /// the last edit's kind, so typing a word is one undo step
    last: &'static str,
    ext: String,
    states: Vec<Option<(ParseState, HighlightState)>>,
    folded: std::collections::BTreeSet<usize>,
    visible: Vec<usize>,
}

impl Default for CodeDocRust {
    fn default() -> Self {
        CodeDocRust {
            path: QString::default(),
            rows: 1,
            lines: 1,
            version: 0,
            dirty: false,
            cursors: QString::from("[[0,0,0,0]]"),
            rope: Rope::new(),
            saved: Rope::new(),
            carets: vec![Caret::at(0)],
            undo: Vec::new(),
            redo: Vec::new(),
            last: "",
            ext: String::new(),
            states: Vec::new(),
            folded: Default::default(),
            visible: vec![0],
        }
    }
}

fn indent(s: &str) -> usize {
    s.chars().take_while(|c| *c == ' ' || *c == '\t').map(|c| if c == '\t' { 4 } else { 1 }).sum()
}

impl CodeDocRust {
    fn line_str(&self, l: usize) -> String {
        if l >= self.rope.len_lines() {
            return String::new();
        }
        let s = self.rope.line(l).to_string();
        s.trim_end_matches(['\n', '\r']).to_string()
    }
    fn line_len(&self, l: usize) -> usize {
        self.line_str(l).chars().count()
    }
    fn pos(&self, line: usize, col: usize) -> usize {
        let line = line.min(self.rope.len_lines().saturating_sub(1));
        self.rope.line_to_char(line) + col.min(self.line_len(line))
    }
    fn line_col(&self, p: usize) -> (usize, usize) {
        let p = p.min(self.rope.len_chars());
        let l = self.rope.char_to_line(p);
        (l, p - self.rope.line_to_char(l))
    }
    /// the folded range of a start line: the lines after it indented deeper (blank lines inside belong to it)
    fn fold_end(&self, start: usize) -> Option<usize> {
        let n = self.rope.len_lines();
        let base = self.line_str(start);
        if base.trim().is_empty() {
            return None;
        }
        let ind = indent(&base);
        let mut end = start;
        for l in start + 1..n {
            let s = self.line_str(l);
            if s.trim().is_empty() {
                continue;
            }
            if indent(&s) > ind {
                end = l;
            } else {
                break;
            }
        }
        (end > start).then_some(end)
    }
    fn relayout(&mut self) {
        let n = self.rope.len_lines().max(1);
        let mut vis = Vec::with_capacity(n);
        let mut l = 0;
        while l < n {
            vis.push(l);
            if self.folded.contains(&l) {
                if let Some(e) = self.fold_end(l) {
                    l = e + 1;
                    continue;
                }
            }
            l += 1;
        }
        self.visible = vis;
    }
}

impl qobject::CodeDoc {
    fn changed(mut self: Pin<&mut Self>, from_line: usize) {
        let keep = from_line / KEEP;
        {
            let mut r = self.as_mut().rust_mut();
            r.states.truncate(keep + 1);
            let n = r.rope.len_lines();
            r.folded.retain(|l| *l < n);
            r.relayout();
        }
        let (rows, lines, dirty) = {
            let r = self.rust();
            (r.visible.len() as i32, r.rope.len_lines() as i32, r.rope != r.saved)
        };
        self.as_mut().set_rows(rows);
        self.as_mut().set_lines(lines);
        self.as_mut().set_dirty(dirty);
        self.as_mut().publish();
    }
    fn publish(mut self: Pin<&mut Self>) {
        let js = {
            let r = self.rust();
            let parts: Vec<String> = r.carets.iter().map(|c| {
                let (l, col) = r.line_col(c.head);
                let (al, ac) = r.line_col(c.anchor);
                format!("[{l},{col},{al},{ac}]")
            }).collect();
            format!("[{}]", parts.join(","))
        };
        self.as_mut().set_cursors(QString::from(&js));
        let v = self.version + 1;
        self.as_mut().set_version(v);
    }
    /// a snapshot for undo; typing in a row of the same kind joins the step before
    fn checkpoint(mut self: Pin<&mut Self>, kind: &'static str) {
        let mut r = self.as_mut().rust_mut();
        if kind.is_empty() || kind != r.last {
            let s = Snap { rope: r.rope.clone(), carets: r.carets.clone() };
            r.undo.push(s);
            if r.undo.len() > 500 {
                r.undo.remove(0);
            }
        }
        r.redo.clear();
        r.last = kind;
    }
    /// every cursor's selection replaced by `text` (deleted when empty), from the last so earlier ones stay right
    fn replace_all(mut self: Pin<&mut Self>, text: &str, kind: &'static str) {
        self.as_mut().checkpoint(kind);
        let first_line;
        {
            let mut r = self.as_mut().rust_mut();
            let mut order: Vec<usize> = (0..r.carets.len()).collect();
            order.sort_by_key(|i| std::cmp::Reverse(r.carets[*i].range().0));
            let add = text.chars().count();
            let mut low = usize::MAX;
            for i in order {
                let (a, b) = r.carets[i].range();
                r.rope.remove(a..b);
                r.rope.insert(a, text);
                let shift = add as isize - (b - a) as isize;
                for j in 0..r.carets.len() {
                    if j != i && r.carets[j].range().0 > a {
                        r.carets[j].head = (r.carets[j].head as isize + shift).max(0) as usize;
                        r.carets[j].anchor = (r.carets[j].anchor as isize + shift).max(0) as usize;
                    }
                }
                r.carets[i] = Caret::at(a + add);
                low = low.min(a);
            }
            first_line = r.rope.char_to_line(low.min(r.rope.len_chars()));
        }
        self.changed(first_line);
    }
    /// Backspace (-1) or Delete (+1) at every cursor without a selection; selections are deleted
    fn delete_dir(mut self: Pin<&mut Self>, dir: i32) {
        {
            let mut r = self.as_mut().rust_mut();
            let len = r.rope.len_chars();
            for c in r.carets.iter_mut() {
                if c.head == c.anchor {
                    if dir < 0 && c.head > 0 {
                        c.anchor = c.head - 1;
                    } else if dir > 0 && c.head < len {
                        c.anchor = c.head + 1;
                    }
                }
            }
        }
        self.replace_all("", if dir < 0 { "back" } else { "del" });
    }
    fn motion(mut self: Pin<&mut Self>, f: impl Fn(&CodeDocRust, &Caret) -> (usize, Option<usize>), extend: bool) {
        {
            let mut r = self.as_mut().rust_mut();
            let next: Vec<(usize, Option<usize>)> = r.carets.iter().map(|c| f(&r, c)).collect();
            for (c, (p, goal)) in r.carets.iter_mut().zip(next) {
                c.head = p;
                c.goal = goal;
                if !extend {
                    c.anchor = p;
                }
            }
            r.last = "";
            // cursors that met become one
            r.carets.dedup_by(|a, b| a.head == b.head);
        }
        self.publish();
    }
    fn word_edge(r: &CodeDocRust, p: usize, dir: i32) -> usize {
        let len = r.rope.len_chars();
        let w = |c: char| c.is_alphanumeric() || c == '_';
        let mut p = p;
        if dir < 0 {
            while p > 0 && !w(r.rope.char(p - 1)) { p -= 1; }
            while p > 0 && w(r.rope.char(p - 1)) { p -= 1; }
        } else {
            while p < len && !w(r.rope.char(p)) { p += 1; }
            while p < len && w(r.rope.char(p)) { p += 1; }
        }
        p
    }

    pub fn open(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let p = path.to_string();
        let Ok(text) = std::fs::read_to_string(&p) else { return false };
        {
            let mut r = self.as_mut().rust_mut();
            r.rope = Rope::from_str(&text);
            r.saved = r.rope.clone();
            r.carets = vec![Caret::at(0)];
            r.undo.clear();
            r.redo.clear();
            r.states.clear();   // parse states from before (another language, or none yet) are no use
            r.folded.clear();
            r.ext = std::path::Path::new(&p).extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
        }
        self.as_mut().set_path(path.clone());
        self.changed(0);
        true
    }
    pub fn save(mut self: Pin<&mut Self>) -> bool {
        let p = self.path.to_string();
        let text = self.rust().rope.to_string();
        if std::fs::write(&p, text).is_err() {
            return false;
        }
        {
            let mut r = self.as_mut().rust_mut();
            r.saved = r.rope.clone();
        }
        self.as_mut().set_dirty(false);
        true
    }
    pub fn text(&self) -> QString {
        QString::from(&self.rust().rope.to_string())
    }
    pub fn row_line(&self, row: i32) -> i32 {
        self.rust().visible.get(row.max(0) as usize).copied().unwrap_or(0) as i32
    }
    pub fn line_row(&self, line: i32) -> i32 {
        let v = &self.rust().visible;
        match v.binary_search(&(line.max(0) as usize)) { Ok(i) => i as i32, Err(i) => i.saturating_sub(1) as i32 }
    }
    pub fn fold_at(&self, line: i32) -> i32 {
        let r = self.rust();
        let l = line.max(0) as usize;
        if r.fold_end(l).is_none() { 0 } else if r.folded.contains(&l) { 2 } else { 1 }
    }
    pub fn toggle_fold(mut self: Pin<&mut Self>, line: i32) {
        {
            let mut r = self.as_mut().rust_mut();
            let l = line.max(0) as usize;
            if !r.folded.remove(&l) && r.fold_end(l).is_some() {
                r.folded.insert(l);
                // cursors inside go to the fold's line
                let end = r.fold_end(l).unwrap_or(l);
                let (a, b) = (r.rope.line_to_char(l + 1), r.rope.line_to_char((end + 1).min(r.rope.len_lines())));
                let at = r.pos(l, 0);
                for c in r.carets.iter_mut() {
                    if c.head >= a && c.head < b { *c = Caret::at(at); }
                }
            }
            r.relayout();
        }
        let rows = self.rust().visible.len() as i32;
        self.as_mut().set_rows(rows);
        self.publish();
    }

    pub fn line_html(self: Pin<&mut Self>, line: i32) -> QString {
        let l = line.max(0) as usize;
        let (ss, th) = crate::window::code_syntax();
        let this = self.rust_mut();
        let r = this.get_mut();
        if l >= r.rope.len_lines() {
            return QString::default();
        }
        let syn = ss.find_syntax_by_extension(&r.ext).unwrap_or_else(|| ss.find_syntax_plain_text());
        let hl = Highlighter::new(th);
        // from the nearest kept state up to this line, keeping one every KEEP lines
        let k = (l / KEEP).min(r.states.len().saturating_sub(1));
        let (mut ps, mut hs, mut from) = match r.states.get(k).cloned().flatten() {
            Some((p, h)) => (p, h, k * KEEP),
            None => (ParseState::new(syn), HighlightState::new(&hl, ScopeStack::new()), 0),
        };
        if r.states.is_empty() {
            r.states.push(Some((ps.clone(), hs.clone())));
            from = 0;
        }
        let mut out = String::new();
        let mut i = from;
        loop {
            let s = r.rope.line(i).to_string();
            let ops = ps.parse_line(&s, ss).unwrap_or_default();
            if i == l {
                for (st, piece, _) in RangedHighlightIterator::new(&mut hs, &ops, &s, &hl) {
                    let t = piece.trim_end_matches(['\n', '\r']).replace('\t', "    ");
                    if t.is_empty() { continue; }
                    let e = t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace(' ', "&nbsp;");
                    let f = st.foreground;
                    let bold = st.font_style.contains(syntect::highlighting::FontStyle::BOLD);
                    let it = st.font_style.contains(syntect::highlighting::FontStyle::ITALIC);
                    out += &format!("<font color=\"#{:02x}{:02x}{:02x}\">{}{}{}{}{}</font>", f.r, f.g, f.b, if bold { "<b>" } else { "" }, if it { "<i>" } else { "" }, e, if it { "</i>" } else { "" }, if bold { "</b>" } else { "" });
                }
                break;
            }
            for _ in RangedHighlightIterator::new(&mut hs, &ops, &s, &hl) {}
            i += 1;
            if i % KEEP == 0 && r.states.len() == i / KEEP {
                r.states.push(Some((ps.clone(), hs.clone())));
            }
        }
        QString::from(&out)
    }

    pub fn key(mut self: Pin<&mut Self>, key: i32, text: &QString, mods: i32) -> bool {
        const SHIFT: i32 = 0x0200_0000;
        const CTRL: i32 = 0x0400_0000;
        const ALT: i32 = 0x0800_0000;
        let (shift, ctrl, alt) = (mods & SHIFT != 0, mods & CTRL != 0, mods & ALT != 0);
        let t = text.to_string();
        match key {
            0x0100_0012 | 0x0100_0014 => {
                // Left / Right; Ctrl by words; a selection collapses to its side
                let d = if key == 0x0100_0012 { -1 } else { 1 };
                self.motion(move |r, c| {
                    if !shift && c.head != c.anchor && !ctrl {
                        let (a, b) = c.range();
                        return (if d < 0 { a } else { b }, None);
                    }
                    let p = if ctrl { Self::word_edge(r, c.head, d) } else if d < 0 { c.head.saturating_sub(1) } else { (c.head + 1).min(r.rope.len_chars()) };
                    (p, None)
                }, shift);
                true
            }
            0x0100_0013 | 0x0100_0015 if !alt => {
                // Up / Down, over folds, keeping the column it started in
                let d: i32 = if key == 0x0100_0013 { -1 } else { 1 };
                self.motion(move |r, c| {
                    let (l, col) = r.line_col(c.head);
                    let goal = c.goal.unwrap_or(col);
                    let row = match r.visible.binary_search(&l) { Ok(i) => i, Err(i) => i.saturating_sub(1) } as i32 + d;
                    if row < 0 { return (0, Some(0)); }
                    let Some(&nl) = r.visible.get(row as usize) else { return (r.rope.len_chars(), Some(goal)) };
                    (r.pos(nl, goal), Some(goal))
                }, shift);
                true
            }
            0x0100_0010 | 0x0100_0011 => {
                // Home (the first non-blank, then column 0) / End; Ctrl: the file's start / end
                let home = key == 0x0100_0010;
                self.motion(move |r, c| {
                    if ctrl { return (if home { 0 } else { r.rope.len_chars() }, None); }
                    let (l, col) = r.line_col(c.head);
                    if !home { return (r.pos(l, usize::MAX), None); }
                    let first = indent(&r.line_str(l)).min(r.line_len(l));
                    (r.pos(l, if col == first { 0 } else { first }), None)
                }, shift);
                true
            }
            0x0100_0016 | 0x0100_0017 => {
                let d: i32 = if key == 0x0100_0016 { -30 } else { 30 };
                self.motion(move |r, c| {
                    let (l, col) = r.line_col(c.head);
                    let nl = (l as i32 + d).clamp(0, r.rope.len_lines() as i32 - 1) as usize;
                    (r.pos(nl, col), None)
                }, shift);
                true
            }
            0x0100_0003 => { self.delete_dir(-1); true }
            0x0100_0007 => { self.delete_dir(1); true }
            0x0100_0004 | 0x0100_0005 => {
                // Enter keeps the line's indent, and one more after { ( [ :
                let (l, col) = { let r = self.rust(); r.line_col(r.carets[0].head) };
                let s = self.rust().line_str(l);
                let mut ind: String = s.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
                let before: String = s.chars().take(col).collect();
                if before.trim_end().ends_with(['{', '(', '[', ':']) { ind += "    "; }
                self.replace_all(&format!("\n{ind}"), "");
                true
            }
            0x0100_0001 if !ctrl => { self.replace_all("    ", ""); true }
            0x0100_0000 => {
                // Esc: back to one cursor, nothing selected
                {
                    let mut r = self.as_mut().rust_mut();
                    let c = r.carets[0];
                    r.carets = vec![Caret::at(c.head)];
                }
                self.publish();
                true
            }
            _ if ctrl && !alt && (key == 0x5a || key == 0x59) => {
                // Ctrl+Z undo; Ctrl+Shift+Z / Ctrl+Y redo
                let redo = key == 0x59 || shift;
                let line = {
                    let mut r = self.as_mut().rust_mut();
                    let snap = if redo { r.redo.pop() } else { r.undo.pop() };
                    let Some(s) = snap else { return true };
                    let now = Snap { rope: r.rope.clone(), carets: r.carets.clone() };
                    if redo { r.undo.push(now) } else { r.redo.push(now) }
                    r.rope = s.rope;
                    r.carets = s.carets;
                    r.last = "";
                    0
                };
                self.changed(line);
                true
            }
            _ if ctrl && !alt && key == 0x41 => {
                {
                    let mut r = self.as_mut().rust_mut();
                    let len = r.rope.len_chars();
                    r.carets = vec![Caret { head: len, anchor: 0, goal: None }];
                }
                self.publish();
                true
            }
            _ if ctrl && !alt && key == 0x44 => {
                // Ctrl+D: the word at the cursor, then its next occurrence as one more cursor
                {
                    let mut r = self.as_mut().rust_mut();
                    let last = *r.carets.last().unwrap();
                    if last.head == last.anchor {
                        let a = Self::word_edge(&r, last.head + 1, -1).min(last.head);
                        let b = Self::word_edge(&r, a, 1);
                        let n = r.carets.len();
                        r.carets[n - 1] = Caret { head: b, anchor: a, goal: None };
                    } else {
                        let (a, b) = last.range();
                        let word = r.rope.slice(a..b).to_string();
                        let all = r.rope.to_string();
                        let from = all.char_indices().nth(b).map(|(i, _)| i).unwrap_or(all.len());
                        let hit = all[from..].find(&word).map(|i| from + i).or_else(|| all.find(&word));
                        if let Some(bi) = hit {
                            let ca = all[..bi].chars().count();
                            let c = Caret { head: ca + (b - a), anchor: ca, goal: None };
                            if !r.carets.contains(&c) { r.carets.push(c); }
                        }
                    }
                }
                self.publish();
                true
            }
            _ if !t.is_empty() && !ctrl && t.chars().all(|c| !c.is_control()) => {
                // typing; an opening bracket or quote gets its pair
                let pair = match t.as_str() { "(" => Some(")"), "[" => Some("]"), "{" => Some("}"), "\"" => Some("\""), _ => None };
                self.as_mut().replace_all(&t, "type");
                if let Some(p) = pair {
                    let mut r = self.as_mut().rust_mut();
                    // the closing one after each cursor, from the last; each cursor then moves by the pairs put before it
                    let mut order: Vec<usize> = (0..r.carets.len()).collect();
                    order.sort_by_key(|i| r.carets[*i].head);
                    for &i in order.iter().rev() {
                        let h = r.carets[i].head;
                        r.rope.insert(h, p);
                    }
                    for (k, &i) in order.iter().enumerate() {
                        r.carets[i].head += k;
                        r.carets[i].anchor = r.carets[i].head;
                    }
                    drop(r);
                    let line = { let r = self.rust(); r.line_col(r.carets[0].head).0 };
                    self.changed(line);
                }
                true
            }
            _ => false,
        }
    }

    pub fn click(mut self: Pin<&mut Self>, line: i32, col: i32, shift: bool, alt: bool) {
        {
            let mut r = self.as_mut().rust_mut();
            let p = r.pos(line.max(0) as usize, col.max(0) as usize);
            if alt {
                if !r.carets.iter().any(|c| c.head == p) { r.carets.push(Caret::at(p)); }
            } else if shift {
                r.carets[0].head = p;
                r.carets.truncate(1);
            } else {
                r.carets = vec![Caret::at(p)];
            }
            r.last = "";
        }
        self.publish();
    }
    pub fn drag(mut self: Pin<&mut Self>, line: i32, col: i32) {
        {
            let mut r = self.as_mut().rust_mut();
            let p = r.pos(line.max(0) as usize, col.max(0) as usize);
            r.carets[0].head = p;
        }
        self.publish();
    }
    pub fn insert(self: Pin<&mut Self>, text: &QString) {
        self.replace_all(&text.to_string().replace("\r\n", "\n"), "");
    }
    pub fn selected_text(&self) -> QString {
        let r = self.rust();
        let parts: Vec<String> = r.carets.iter().filter(|c| c.head != c.anchor).map(|c| { let (a, b) = c.range(); r.rope.slice(a..b).to_string() }).collect();
        QString::from(&parts.join("\n"))
    }
}
