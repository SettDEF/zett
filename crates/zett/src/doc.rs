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
        /// one more cursor on the line above (-1) or below (+1) the last one, at its column
        #[qinvokable]
        #[cxx_name = "addCursorLine"]
        fn add_cursor_line(self: Pin<&mut CodeDoc>, dir: i32);
        /// the bracket at the main cursor and its partner: [l, c, l2, c2], or "" for none
        #[qinvokable]
        #[cxx_name = "bracketPair"]
        fn bracket_pair(self: &CodeDoc) -> QString;
        /// the lines that open the scope around a line (less indented, ending in { ( [ : or a declaration), 3 at most
        #[qinvokable]
        fn scope(self: &CodeDoc, line: i32) -> QString;
        /// double click: the word there; triple: the line
        #[qinvokable]
        #[cxx_name = "selectUnit"]
        fn select_unit(self: Pin<&mut CodeDoc>, line: i32, col: i32, whole_line: bool);
        /// Shift+Alt drag: a cursor on each line from l0 to l1 at column col
        #[qinvokable]
        fn columns(self: Pin<&mut CodeDoc>, l0: i32, l1: i32, col: i32);
        /// the file's own words starting with `prefix` (not the prefix itself), nearest the cursor first, 30 at most
        #[qinvokable]
        fn words(self: &CodeDoc, prefix: &QString) -> QString;
        /// completion without a server: this file's words, the language's keywords, the other open files' words (paths as JSON)
        #[qinvokable]
        fn candidates(self: &CodeDoc, prefix: &QString, ext: &QString, others: &QString) -> QString;
        /// the newest ycmd answer's number (the window asks for the list again when it grows)
        #[qinvokable]
        #[cxx_name = "ycmSeq"]
        fn ycm_seq(self: &CodeDoc) -> i32;
        /// the text replaced, the file on disk kept as it is (so it shows as unsaved): edits kept while away
        #[qinvokable]
        fn restore(self: Pin<&mut CodeDoc>, text: &QString);
        #[qinvokable]
        #[cxx_name = "lineText"]
        fn line_text(self: &CodeDoc, line: i32) -> QString;
        #[qinvokable]
        #[cxx_name = "selectedText"]
        fn selected_text(self: &CodeDoc) -> QString;
        /// line editing: "comment://", "up", "down", "copyup", "copydown", "delete"
        #[qinvokable]
        #[cxx_name = "lineOp"]
        fn line_op(self: Pin<&mut CodeDoc>, op: &QString);
        /// every match of a search (opts JSON {case, regex, word}): [[line, col, len], …], 5000 at most
        #[qinvokable]
        fn find(self: &CodeDoc, query: &QString, opts: &QString) -> QString;
        /// the main cursor on a range (a match, a definition): selects it
        #[qinvokable]
        fn select(self: Pin<&mut CodeDoc>, line: i32, col: i32, len: i32);
        /// every match replaced; how many
        #[qinvokable]
        #[cxx_name = "replaceAll"]
        fn replace_all_matches(self: Pin<&mut CodeDoc>, query: &QString, with: &QString, opts: &QString) -> i32;
        /// edits from a server (format, rename, code action): [[l0, c0, l1, c1, text], …], applied from the last
        #[qinvokable]
        #[cxx_name = "applyEdits"]
        fn apply_edits(self: Pin<&mut CodeDoc>, edits: &QString);
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
    /// states far from the dense ones (a big file opened deep in): by line, every KEEP lines, from where it was entered
    far: std::collections::BTreeMap<usize, (ParseState, HighlightState)>,
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
            far: Default::default(),
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
            let cut = keep * KEEP;
            r.far.retain(|l, _| *l <= cut);
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
            r.far.clear();
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
        // far from any kept state (a big file opened at its end): start 200 lines above, fresh, keeping nothing;
        // right for nearly all code, and the kept states catch up as the view moves down
        let near = r.states.get(k).cloned().flatten().filter(|_| l - k * KEEP < 300);
        let approx = near.is_none() && l >= 300;
        let (mut ps, mut hs, mut from) = match near {
            Some((p, h)) => (p, h, k * KEEP),
            // the nearest far state above within 300 lines, else fresh 200 lines above (on a KEEP line, so it is kept)
            None if approx => match r.far.range(l.saturating_sub(300)..=l).next_back().map(|(k, v)| (*k, v.clone())) {
                Some((at, (p, h))) => (p, h, at),
                None => (ParseState::new(syn), HighlightState::new(&hl, ScopeStack::new()), (l - 200) / KEEP * KEEP),
            },
            None => (ParseState::new(syn), HighlightState::new(&hl, ScopeStack::new()), 0),
        };
        if r.states.is_empty() && !approx {
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
            if i % KEEP == 0 {
                if !approx && r.states.len() == i / KEEP { r.states.push(Some((ps.clone(), hs.clone()))); }
                else if approx && !r.far.contains_key(&i) { r.far.insert(i, (ps.clone(), hs.clone())); }
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
            0x0100_0001 | 0x0100_0002 if !ctrl => {
                // Tab: four spaces, or with a selection over lines (or Shift+Tab) the lines in or out by four
                let multi = { let r = self.rust(); let (a, b) = r.carets[0].range(); r.line_col(a).0 != r.line_col(b).0 };
                if key == 0x0100_0001 && !shift && !multi { self.replace_all("    ", ""); return true }
                let (a, b) = { let r = self.rust(); let (x, y) = r.carets[0].range(); let (la, _) = r.line_col(x); let (mut lb, cb) = r.line_col(y); if lb > la && cb == 0 { lb -= 1 } (la, lb) };
                let out = key == 0x0100_0002 || shift;
                self.as_mut().checkpoint("");
                {
                    let mut r = self.as_mut().rust_mut();
                    for l in (a..=b).rev() {
                        let s = r.rope.line_to_char(l);
                        if out { let n = r.line_str(l).chars().take(4).take_while(|c| *c == ' ').count(); r.rope.remove(s..s + n); } else if !r.line_str(l).is_empty() { r.rope.insert(s, "    "); }
                    }
                    let (sa, sb) = (r.rope.line_to_char(a), r.pos(b, usize::MAX));
                    r.carets = vec![Caret { head: sb, anchor: sa, goal: None }];
                }
                self.changed(a);
                true
            }
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

fn pattern(q: &str, opts: &str) -> Option<regex::Regex> {
    let o: serde_json::Value = serde_json::from_str(opts).unwrap_or_default();
    if q.is_empty() {
        return None;
    }
    let src = if o["regex"] == true { q.to_string() } else { regex::escape(q) };
    let src = if o["word"] == true { format!(r"\b{src}\b") } else { src };
    regex::RegexBuilder::new(&src).case_insensitive(o["case"] != true).multi_line(o["multiline"] == true).build().ok()
}

impl qobject::CodeDoc {
    pub fn line_op(mut self: Pin<&mut Self>, op: &QString) {
        let op = op.to_string();
        // the lines the cursors cover, each cursor's own range of lines
        let (a, b) = {
            let r = self.rust();
            let c = r.carets[0];
            let (s, e) = c.range();
            let (la, _) = r.line_col(s);
            let (mut lb, cb) = r.line_col(e);
            if lb > la && cb == 0 { lb -= 1; }
            (la, lb)
        };
        let n = self.rust().rope.len_lines();
        let rows: Vec<String> = (a..=b).map(|l| self.rust().line_str(l)).collect();
        let (new_rows, from, to, shift): (Vec<String>, usize, usize, i32) = if let Some(mark) = op.strip_prefix("comment:") {
            let all = rows.iter().filter(|s| !s.trim().is_empty()).all(|s| s.trim_start().starts_with(mark));
            let ind = rows.iter().filter(|s| !s.trim().is_empty()).map(|s| s.len() - s.trim_start().len()).min().unwrap_or(0);
            (rows.iter().map(|s| {
                if s.trim().is_empty() { return s.clone() }
                if all { let t = s.trim_start(); let lead = &s[..s.len() - t.len()]; let rest = t[mark.len()..].strip_prefix(' ').unwrap_or(&t[mark.len()..]); format!("{lead}{rest}") }
                else { format!("{}{mark} {}", &s[..ind], &s[ind..]) }
            }).collect(), a, b, 0)
        } else {
            match op.as_str() {
                "up" if a > 0 => { let mut v = rows.clone(); v.push(self.rust().line_str(a - 1)); (v, a - 1, b, -1) }
                "down" if b + 1 < n && !(b + 2 == n && self.rust().line_str(b + 1).is_empty()) => { let mut v = vec![self.rust().line_str(b + 1)]; v.extend(rows.clone()); (v, a, b + 1, 1) }
                "copyup" | "copydown" => { let mut v = rows.clone(); v.extend(rows.clone()); (v, a, b, if op == "copydown" { (b - a + 1) as i32 } else { 0 }) }
                "delete" => (Vec::new(), a, b, 0),
                _ => return,
            }
        };
        self.as_mut().checkpoint("");
        {
            let mut r = self.as_mut().rust_mut();
            let (_, col) = r.line_col(r.carets[0].head);
            let start = r.rope.line_to_char(from);
            let end = if to + 1 < r.rope.len_lines() { r.rope.line_to_char(to + 1) } else { r.rope.len_chars() };
            let tail = if to + 1 < r.rope.len_lines() { "\n" } else { "" };
            r.rope.remove(start..end);
            if !new_rows.is_empty() {
                r.rope.insert(start, &(new_rows.join("\n") + tail));
            }
            let nl = ((a as i32 + shift).max(0) as usize).min(r.rope.len_lines().saturating_sub(1));
            let p = r.pos(nl, col);
            r.carets = vec![Caret::at(p)];
        }
        self.changed(from);
    }
    pub fn find(&self, query: &QString, opts: &QString) -> QString {
        let r = self.rust();
        let Some(re) = pattern(&query.to_string(), &opts.to_string()) else { return QString::from("[]") };
        let mut out = Vec::new();
        for (l, line) in r.rope.lines().enumerate() {
            let s = line.to_string();
            for m in re.find_iter(s.trim_end_matches(['\n', '\r'])) {
                out.push(format!("[{l},{},{}]", s[..m.start()].chars().count(), m.as_str().chars().count()));
                if out.len() >= 5000 { return QString::from(&format!("[{}]", out.join(","))) }
            }
        }
        QString::from(&format!("[{}]", out.join(",")))
    }
    pub fn select(mut self: Pin<&mut Self>, line: i32, col: i32, len: i32) {
        {
            let mut r = self.as_mut().rust_mut();
            let a = r.pos(line.max(0) as usize, col.max(0) as usize);
            // a folded line opens to show it
            let l = line.max(0) as usize;
            let opened: Vec<usize> = r.folded.iter().copied().filter(|f| *f < l && r.fold_end(*f).is_some_and(|e| e >= l)).collect();
            for f in opened { r.folded.remove(&f); }
            r.relayout();
            r.carets = vec![Caret { head: a + len.max(0) as usize, anchor: a, goal: None }];
        }
        let rows = self.rust().visible.len() as i32;
        self.as_mut().set_rows(rows);
        self.publish();
    }
    pub fn replace_all_matches(mut self: Pin<&mut Self>, query: &QString, with: &QString, opts: &QString) -> i32 {
        let o = opts.to_string();
        let Some(re) = pattern(&query.to_string(), &o) else { return 0 };
        let text = self.rust().rope.to_string();
        let n = re.find_iter(&text).count();
        if n == 0 { return 0 }
        let regex = serde_json::from_str::<serde_json::Value>(&o).unwrap_or_default()["regex"] == true;
        let w = with.to_string();
        let out = if regex { re.replace_all(&text, w.as_str()).to_string() } else { re.replace_all(&text, regex::NoExpand(&w)).to_string() };
        self.as_mut().checkpoint("");
        {
            let mut r = self.as_mut().rust_mut();
            let head = r.carets[0].head.min(out.chars().count());
            r.rope = Rope::from_str(&out);
            r.carets = vec![Caret::at(head)];
        }
        self.changed(0);
        n as i32
    }
    pub fn apply_edits(mut self: Pin<&mut Self>, edits: &QString) {
        let v: Vec<(usize, usize, usize, usize, String)> = serde_json::from_str(&edits.to_string()).unwrap_or_default();
        if v.is_empty() { return }
        self.as_mut().checkpoint("");
        let mut low = usize::MAX;
        {
            let mut r = self.as_mut().rust_mut();
            let mut v = v;
            v.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));
            for (l0, c0, l1, c1, t) in v {
                let a = r.pos(l0, c0);
                let b = r.pos(l1, c1).max(a);
                r.rope.remove(a..b);
                r.rope.insert(a, &t);
                low = low.min(l0);
            }
            let len = r.rope.len_chars();
            for c in r.carets.iter_mut() { c.head = c.head.min(len); c.anchor = c.anchor.min(len); }
        }
        let n = self.rust().rope.len_lines();
        self.changed(low.min(n));
    }
}

impl qobject::CodeDoc {
    pub fn line_text(&self, line: i32) -> QString {
        QString::from(&self.rust().line_str(line.max(0) as usize))
    }
}

impl qobject::CodeDoc {
    pub fn add_cursor_line(mut self: Pin<&mut Self>, dir: i32) {
        {
            let mut r = self.as_mut().rust_mut();
            let last = *r.carets.last().unwrap();
            let (l, c) = r.line_col(last.head);
            let nl = l as i32 + dir;
            if nl < 0 || nl as usize >= r.rope.len_lines() { return }
            let p = r.pos(nl as usize, c);
            if !r.carets.iter().any(|x| x.head == p) { r.carets.push(Caret::at(p)); }
        }
        self.publish();
    }
    pub fn bracket_pair(&self) -> QString {
        let r = self.rust();
        let p = r.carets[0].head;
        let (open, close) = ("([{", ")]}");
        for at in [p, p.wrapping_sub(1)] {
            if at >= r.rope.len_chars() { continue }
            let c = r.rope.char(at);
            let (k, dir) = match (open.find(c), close.find(c)) { (Some(k), _) => (k, 1i64), (_, Some(k)) => (k, -1), _ => continue };
            let (me, other) = if dir > 0 { (c, close.as_bytes()[k] as char) } else { (c, open.as_bytes()[k] as char) };
            let mut depth = 0i32;
            let mut i = at as i64;
            let mut steps = 0;
            while i >= 0 && (i as usize) < r.rope.len_chars() && steps < 50_000 {
                let ch = r.rope.char(i as usize);
                if ch == me { depth += 1 } else if ch == other { depth -= 1; if depth == 0 {
                    let (l1, c1) = r.line_col(at);
                    let (l2, c2) = r.line_col(i as usize);
                    return QString::from(&format!("[{l1},{c1},{l2},{c2}]"));
                } }
                i += dir;
                steps += 1;
            }
            break;
        }
        QString::default()
    }
    pub fn scope(&self, line: i32) -> QString {
        let r = self.rust();
        let l = (line.max(0) as usize).min(r.rope.len_lines().saturating_sub(1));
        static DECL: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let decl = DECL.get_or_init(|| regex::Regex::new(r"^\s*(pub(\([^)]*\))?\s+|export\s+|async\s+)*(fn|impl|struct|enum|trait|mod|class|def|function|interface)\b").unwrap());
        let mut min = indent(&r.line_str(l));
        let mut out = Vec::new();
        let mut i = l;
        while i > 0 && min > 0 && out.len() < 3 && l - i < 2000 {
            i -= 1;
            let s = r.line_str(i);
            if s.trim().is_empty() { continue }
            let ind = indent(&s);
            if ind < min && (s.trim_end().ends_with(['{', '(', '[', ':']) || decl.is_match(&s)) {
                out.push(i);
                min = ind;
            }
        }
        out.reverse();
        QString::from(&format!("[{}]", out.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")))
    }
}

impl qobject::CodeDoc {
    pub fn select_unit(mut self: Pin<&mut Self>, line: i32, col: i32, whole_line: bool) {
        {
            let mut r = self.as_mut().rust_mut();
            let l = (line.max(0) as usize).min(r.rope.len_lines().saturating_sub(1));
            let c = if whole_line {
                let a = r.rope.line_to_char(l);
                let b = if l + 1 < r.rope.len_lines() { r.rope.line_to_char(l + 1) } else { r.rope.len_chars() };
                Caret { head: b, anchor: a, goal: None }
            } else {
                let p = r.pos(l, col.max(0) as usize);
                let a = Self::word_edge(&r, (p + 1).min(r.rope.len_chars()), -1).min(p);
                let b = Self::word_edge(&r, a, 1).max(a);
                Caret { head: b, anchor: a, goal: None }
            };
            r.carets = vec![c];
        }
        self.publish();
    }
    pub fn columns(mut self: Pin<&mut Self>, l0: i32, l1: i32, col: i32) {
        {
            let mut r = self.as_mut().rust_mut();
            let (a, b) = (l0.min(l1).max(0) as usize, l0.max(l1).max(0) as usize);
            let b = b.min(r.rope.len_lines().saturating_sub(1));
            let cs: Vec<Caret> = (a..=b).map(|l| Caret::at(r.pos(l, col.max(0) as usize))).collect();
            if !cs.is_empty() { r.carets = cs; }
        }
        self.publish();
    }
}

impl qobject::CodeDoc {
    // the terminal editor's keyword completion (zett_tui::keyword_candidates, as i_CTRL-N): this file, nearest first
    pub fn words(&self, prefix: &QString) -> QString {
        let r = self.rust();
        let p = prefix.to_string();
        if p.is_empty() { return QString::from("[]") }
        let found = zett_tui::keyword_candidates(&p, &r.rope.to_string(), r.carets[0].head, false, &[], true);
        QString::from(&serde_json::Value::Array(found.into_iter().take(30).map(|(w, _)| serde_json::Value::String(w)).collect()).to_string())
    }

    pub fn ycm_seq(&self) -> i32 {
        zett_ycm::answered() as i32
    }

    pub fn candidates(&self, prefix: &QString, ext: &QString, others: &QString) -> QString {
        let r = self.rust();
        let p = prefix.to_string();
        if p.is_empty() { return QString::from("[]") }
        let lower = p.to_lowercase();
        // the other open files, as on disk (a few, not huge ones)
        // each: a path (read from disk) or { path, text } (an unsaved tab, as it is now)
        let list: Vec<serde_json::Value> = serde_json::from_str(&others.to_string()).unwrap_or_default();
        let texts: Vec<(String, String)> = list.iter().take(12).filter_map(|v| {
            let (f, text) = match v { serde_json::Value::String(p) => (p.clone(), None), o => (o["path"].as_str()?.to_string(), o["text"].as_str().map(String::from)) };
            let name = std::path::Path::new(&f).file_name()?.to_string_lossy().into_owned();
            let text = match text { Some(t) => t, None => { if std::fs::metadata(&f).map_or(true, |m| m.len() >= 2_000_000) { return None } std::fs::read_to_string(&f).ok()? } };
            Some((name, text))
        }).collect();
        let mut out: Vec<serde_json::Value> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        // YouCompleteMe's engine first (ycmd, when built): fuzzy (gfi → getFileInfo), ranked; `ext` is the file's path
        let path = ext.to_string();
        if path.contains('/') {
            let head = r.carets[0].head.min(r.rope.len_chars());
            let line = r.rope.char_to_line(head);
            let col = r.rope.line(line).slice(..head - r.rope.line_to_char(line)).len_bytes();
            let state = std::path::PathBuf::from(std::env::var("XDG_STATE_HOME").unwrap_or_else(|_| format!("{}/.local/state", std::env::var("HOME").unwrap_or_default()))).join("zett/ycmd");
            let text = r.rope.to_string();
            // asked without waiting; the newest answer for this file and line is used (the window looks again when one comes)
            zett_ycm::ask(&state, &path, &text, line, col);
            let a = zett_ycm::answer();
            if a.seq > 0 && a.path == path && a.line == line {
                // an answer for fewer letters: still only what fits the word now, fuzzily (its letters in order)
                let fits = |w: &str| { let mut it = w.chars().flat_map(char::to_lowercase); lower.chars().all(|q| it.any(|c| c == q)) };
                for i in a.items.into_iter().filter(|i| i.text != p && fits(&i.text)).take(40) {
                    if !seen.insert(i.text.clone()) { continue }
                    let detail = if i.kind == "[ID]" || i.kind.is_empty() { "ycm".to_string() } else if i.menu.is_empty() { i.kind.to_lowercase() } else { i.menu.clone() };
                    out.push(serde_json::json!({ "label": i.text, "insert": i.text, "detail": detail }));
                }
            }
        }
        let ext = QString::from(std::path::Path::new(&path).extension().and_then(|e| e.to_str()).unwrap_or(&path));
        for (w, from) in zett_tui::keyword_candidates(&p, &r.rope.to_string(), r.carets[0].head, false, &texts, true).into_iter().take(30) {
            if seen.contains(&w) { continue }
            let detail = if from.is_empty() { "in this file".to_string() } else { format!("in {from}") };
            seen.insert(w.clone());
            out.push(serde_json::json!({ "label": w, "insert": w, "detail": detail }));
        }
        // keywords after the words that are really there
        for k in zett_tui::language_keywords(&ext.to_string()) {
            if k.len() > p.len() && k.to_lowercase().starts_with(&lower) && seen.insert(k.to_string()) {
                out.push(serde_json::json!({ "label": k, "insert": k, "detail": "keyword" }));
            }
        }
        QString::from(&serde_json::Value::Array(out).to_string())
    }
}

impl qobject::CodeDoc {
    pub fn restore(mut self: Pin<&mut Self>, text: &QString) {
        {
            let mut r = self.as_mut().rust_mut();
            let s = text.to_string();
            r.rope = Rope::from_str(&s);
            r.states.clear();
            r.far.clear();
            r.carets = vec![Caret::at(0)];
        }
        self.changed(0);
    }
}
