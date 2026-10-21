//! Vim's diff mode: :diffthis :diffsplit :diffoff :diffupdate :diffget :diffput :diffpatch, and
//! do dp ]c [c. Two windows in it are compared line by line (Myers' diff; 'diffopt' icase iwhite
//! iwhiteall filler vertical), drawn with DiffAdd DiffChange DiffText and DiffDelete filler lines,
//! and scrolled together through the diff.

use super::*;
use std::collections::HashMap;

const NAMES: &[(&str, &str)] = &[
    ("diffthis", "difft"), ("diffoff", "diffo"), ("diffsplit", "diffs"), ("diffupdate", "dif"), ("diffget", "diffg"),
    ("diffput", "diffpu"), ("diffpatch", "diffp"),
];

/// A typed name as one of these commands (its full name).
pub(crate) fn diff_name(typed: &str) -> Option<&'static str> {
    NAMES.iter().find(|(full, short)| typed.len() >= short.len() && full.starts_with(typed)).map(|(f, _)| *f)
}

/// A change: lines `a.0..a.1` here stand where `b.0..b.1` are in the other buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Block {
    pub a: (usize, usize),
    pub b: (usize, usize),
}

/// What equal means: 'diffopt' icase, iwhite (the amount of blanks), iwhiteall (any blanks).
fn key(line: &str, opt: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let has = |f: &str| opt.split(',').any(|o| o.trim() == f);
    let mut s: String = if has("iwhiteall") {
        line.chars().filter(|c| !c.is_whitespace()).collect()
    } else if has("iwhite") {
        line.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        line.to_string()
    };
    if has("icase") {
        s = s.to_lowercase();
    }
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

/// The changes between two lists of lines (Myers' shortest edit; past a few thousand differences the
/// middle is one change, as a guard against huge, unrelated files).
pub(crate) fn diff(a: &[String], b: &[String], opt: &str) -> Vec<Block> {
    let ka: Vec<u64> = a.iter().map(|l| key(l, opt)).collect();
    let kb: Vec<u64> = b.iter().map(|l| key(l, opt)).collect();
    // the same first and last lines first: most diffs are small
    let pre = ka.iter().zip(&kb).take_while(|(x, y)| x == y).count();
    let suf = ka[pre..].iter().rev().zip(kb[pre..].iter().rev()).take_while(|(x, y)| x == y).count();
    let (ma, mb) = (&ka[pre..ka.len() - suf], &kb[pre..kb.len() - suf]);
    let pairs = myers(ma, mb).unwrap_or_default();
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    for (x, y) in pairs.into_iter().chain(std::iter::once((ma.len(), mb.len()))) {
        if x > i || y > j {
            out.push(Block { a: (pre + i, pre + x), b: (pre + j, pre + y) });
        }
        (i, j) = (x + 1, y + 1);
    }
    out
}

/// The equal pairs of a shortest edit, in order; None past the limit (then all of it is one change).
fn myers(a: &[u64], b: &[u64]) -> Option<Vec<(usize, usize)>> {
    let (n, m) = (a.len() as isize, b.len() as isize);
    let max = n + m;
    if max == 0 {
        return Some(Vec::new());
    }
    let off = max as usize;
    let mut v = vec![0isize; 2 * off + 2];
    let mut trace: Vec<Vec<isize>> = Vec::new();
    let mut found = None;
    'outer: for d in 0..=max.min(4000) {
        // v for k in -d..=d as it was when step d began
        trace.push(v[off - d as usize..=off + d as usize].to_vec());
        let mut k = -d;
        while k <= d {
            let ki = (k + off as isize) as usize;
            let mut x = if k == -d || (k != d && v[ki - 1] < v[ki + 1]) { v[ki + 1] } else { v[ki - 1] + 1 };
            let mut y = x - k;
            while x < n && y < m && a[x as usize] == b[y as usize] {
                x += 1;
                y += 1;
            }
            v[ki] = x;
            if x >= n && y >= m {
                found = Some(d);
                break 'outer;
            }
            k += 2;
        }
    }
    let dmax = found?;
    let mut pairs = Vec::new();
    let (mut x, mut y) = (n, m);
    for d in (1..=dmax).rev() {
        let vs = &trace[d as usize];
        let at = |k: isize| vs[(k + d) as usize];
        let k = x - y;
        let pk = if k == -d || (k != d && at(k - 1) < at(k + 1)) { k + 1 } else { k - 1 };
        let px = at(pk);
        let py = px - pk;
        while x > px && y > py && x > 0 && y > 0 {
            pairs.push(((x - 1) as usize, (y - 1) as usize));
            x -= 1;
            y -= 1;
        }
        (x, y) = (px, py);
    }
    while x > 0 && y > 0 {
        pairs.push(((x - 1) as usize, (y - 1) as usize));
        x -= 1;
        y -= 1;
    }
    pairs.reverse();
    Some(pairs)
}

/// The changed part of two lines (their common start and end left out): its char range in `a`.
pub(crate) fn changed_part(a: &str, b: &str) -> (usize, usize) {
    let (ca, cb): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let pre = ca.iter().zip(&cb).take_while(|(x, y)| x == y).count();
    let suf = ca[pre..].iter().rev().zip(cb[pre..].iter().rev()).take_while(|(x, y)| x == y).count();
    (pre, ca.len() - suf)
}

/// The other side's line for line `l` here: through the changes, the same distance into an equal part.
pub(crate) fn map_line(blocks: &[Block], l: usize) -> usize {
    let mut shift: isize = 0;
    for bl in blocks {
        if l < bl.a.0 {
            break;
        }
        if l < bl.a.1 {
            // inside a change: as far into the other side's part as it goes
            return (bl.b.0 + (l - bl.a.0)).min(bl.b.1.saturating_sub(1).max(bl.b.0));
        }
        shift = bl.b.1 as isize - bl.a.1 as isize;
    }
    (l as isize + shift).max(0) as usize
}

/// How a window in diff mode is drawn: filler lines before a line, and each changed line's group
/// and changed part (chars).
#[derive(Default)]
pub(crate) struct View {
    pub fill: HashMap<usize, usize>,
    pub lines: HashMap<usize, (&'static str, Option<(usize, usize)>)>,
}

fn lines_of(ed: &zett_edit::Editor) -> Vec<String> {
    let n = ed.buf.len_lines();
    let mut v: Vec<String> = (0..n).map(|l| ed.buf.line(l).trim_end_matches('\n').to_string()).collect();
    // a final newline makes no line of its own
    if v.len() > 1 && v.last().is_some_and(|l| l.is_empty()) && ed.buf.text().ends_with('\n') {
        v.pop();
    }
    v
}

impl App {
    /// The buffer a window shows (the focused one's is the current buffer, its window learns it later).
    pub(crate) fn win_doc(&self, w: usize) -> usize {
        if w == self.focus { self.cur } else { self.wins.get(w).and_then(|w| w.as_ref()).map_or(self.cur, |w| w.doc.min(self.docs.len() - 1)) }
    }

    /// The window this one is compared with: the first other one in diff mode.
    pub(crate) fn diff_partner(&self, win: usize) -> Option<usize> {
        if !self.wins.get(win).and_then(|w| w.as_ref()).is_some_and(|w| w.diff) {
            return None;
        }
        self.tree.ids().into_iter().find(|&w| w != win && self.wins.get(w).and_then(|w| w.as_ref()).is_some_and(|w| w.diff))
    }

    /// The changes between two buffers, from `da`'s side; kept until either changes (:diffupdate drops them).
    pub(crate) fn diff_blocks(&mut self, da: usize, db: usize) -> Vec<Block> {
        let opt = self.setting(da, "diffopt");
        let revs = (self.docs[da].ed.buf.rev(), self.docs[db].ed.buf.rev());
        if let Some((r, o, b)) = self.diff_cache.get(&(da, db)) {
            if *r == revs && *o == opt {
                return b.clone();
            }
        }
        let blocks = diff(&lines_of(&self.docs[da].ed), &lines_of(&self.docs[db].ed), &opt);
        self.diff_cache.insert((da, db), (revs, opt, blocks.clone()));
        blocks
    }

    /// How window `win` (showing `di`) is drawn in diff mode; None when it is not in it.
    pub(crate) fn diff_view(&mut self, win: usize, di: usize) -> Option<View> {
        let other = self.diff_partner(win)?;
        let db = self.win_doc(other);
        let blocks = self.diff_blocks(di, db);
        let filler = self.setting(di, "diffopt").split(',').any(|o| o.trim() == "filler");
        let mut v = View::default();
        for bl in &blocks {
            let (na, nb) = (bl.a.1 - bl.a.0, bl.b.1 - bl.b.0);
            for (k, l) in (bl.a.0..bl.a.1).enumerate() {
                if k < nb {
                    let here = self.docs[di].ed.buf.line(l);
                    let there = self.docs[db].ed.buf.line(bl.b.0 + k);
                    v.lines.insert(l, ("DiffChange", Some(changed_part(here.trim_end_matches('\n'), there.trim_end_matches('\n')))));
                } else {
                    v.lines.insert(l, ("DiffAdd", None));
                }
            }
            if nb > na && filler {
                *v.fill.entry(bl.a.0 + na).or_default() += nb - na;
            }
        }
        Some(v)
    }

    /// The change at or after line `l` (the change a deleted part stands before counts at its line).
    fn diff_block_at(&mut self, l: usize, end: usize) -> Vec<Block> {
        let Some(other) = self.diff_partner(self.focus) else { return Vec::new() };
        let db = self.win_doc(other);
        let blocks = self.diff_blocks(self.cur, db);
        blocks.into_iter().filter(|b| (b.a.0 <= end && b.a.1 > l) || (b.a.0 == b.a.1 && (b.a.0 == l || b.a.0 == l + 1))).collect()
    }

    /// ]c [c in diff mode: to the start of the next / previous change; false when not in diff mode.
    pub(crate) fn diff_jump(&mut self, dir: isize) -> bool {
        let Some(other) = self.diff_partner(self.focus) else { return false };
        let db = self.win_doc(other);
        let blocks = self.diff_blocks(self.cur, db);
        let l = self.docs[self.cur].ed.cursor().line;
        let target = if dir > 0 { blocks.iter().map(|b| b.a.0).find(|&s| s > l) } else { blocks.iter().map(|b| b.a.0).filter(|&s| s < l).last() };
        match target {
            Some(t) => self.goto_line(t.min(self.last_line()) + 1),
            None => self.say(if dir > 0 { "no more changes below" } else { "no more changes above" }),
        }
        true
    }

    fn set_diff(&mut self, win: usize, on: bool) {
        let Some(Some(w)) = self.wins.get_mut(win) else { return };
        w.diff = on;
        let di = self.win_doc(win);
        for (name, value) in [("diff", on), ("scrollbind", on), ("cursorbind", on)] {
            self.docs[di].local.insert(name.into(), value.to_string());
        }
        self.diff_cache.clear();
    }

    /// Before each frame: :set diff and :set nodiff turn the current window's diff mode on and off.
    pub(crate) fn sync_diff(&mut self) {
        let on = self.flag(self.cur, "diff");
        if self.wins.get(self.focus).and_then(|w| w.as_ref()).is_some_and(|w| w.diff != on) {
            self.set_diff(self.focus, on);
        }
    }

    /// One of these commands.
    pub(crate) fn diff_command(&mut self, name: &str, arg: &str, bang: bool, range: Option<(usize, usize)>) {
        let Some(cmd) = diff_name(name) else { return };
        match cmd {
            "diffthis" => self.set_diff(self.focus, true),
            "diffoff" => {
                let wins: Vec<usize> = if bang { self.tree.ids() } else { vec![self.focus] };
                for w in wins {
                    self.set_diff(w, false);
                }
            }
            "diffupdate" => self.diff_cache.clear(),
            "diffsplit" | "diffpatch" => {
                let arg = arg.trim();
                if arg.is_empty() {
                    return self.say(format!("E471: Argument required: :{cmd} {{file}}"));
                }
                let new = if cmd == "diffpatch" {
                    // the patched file next to the original, by patch(1), as Vim makes it
                    let Some(orig) = self.docs[self.cur].abs_path() else { return self.say("E37: :diffpatch needs a file on disk") };
                    let out = std::env::temp_dir().join(format!("zett-patched-{}-{}", std::process::id(), orig.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default()));
                    let patch = self.root.join(arg);
                    let run = std::process::Command::new("patch").arg("-o").arg(&out).arg(&orig).arg(&patch).output();
                    match run {
                        Ok(o) if o.status.success() => out.display().to_string(),
                        Ok(o) => return self.say(format!("E816: patch failed: {}", String::from_utf8_lossy(&o.stdout).lines().last().unwrap_or(""))),
                        Err(e) => return self.say(format!("E816: no patch program: {e}")),
                    }
                } else {
                    arg.to_string()
                };
                self.set_diff(self.focus, true);
                let vertical = self.setting(self.cur, "diffopt").split(',').any(|o| o.trim() == "vertical");
                self.doom_ui("split", if vertical { "right" } else { "below" });
                self.command(&format!("e {new}"));
                self.set_diff(self.focus, true);
            }
            "diffget" | "diffput" => {
                let Some(other) = self.diff_partner(self.focus) else { return self.say("E99: Current buffer is not in diff mode") };
                let cur = self.docs[self.cur].ed.cursor().line;
                let (l, end) = range.unwrap_or((cur, cur));
                let blocks = self.diff_block_at(l, end);
                if blocks.is_empty() {
                    return self.say("no change here");
                }
                let (da, db) = (self.cur, self.win_doc(other));
                let text_a = lines_of(&self.docs[da].ed);
                let text_b = lines_of(&self.docs[db].ed);
                // from the last change up, so the line numbers of the earlier ones hold
                for bl in blocks.iter().rev() {
                    if cmd == "diffget" {
                        let new: Vec<String> = text_b[bl.b.0..bl.b.1].to_vec();
                        self.replace_doc_lines(da, bl.a.0, bl.a.1, new);
                    } else {
                        let new: Vec<String> = text_a[bl.a.0..bl.a.1].to_vec();
                        self.replace_doc_lines(db, bl.b.0, bl.b.1, new);
                    }
                }
                self.diff_cache.clear();
            }
            _ => {}
        }
    }

    /// Lines `a..b` of buffer `di` become `new` (a == b: put before line a), as one undo step.
    fn replace_doc_lines(&mut self, di: usize, a: usize, b: usize, new: Vec<String>) {
        let ed = &mut self.docs[di].ed;
        let real = lines_of(ed).len();
        let end_nl = ed.buf.text().ends_with('\n');
        let at = |k: usize| if k < real { ed.buf.to_char(Pos { line: k, col: 0 }) } else { ed.buf.len_chars() };
        let (from, to) = (at(a), at(b));
        let mut ins: String = new.iter().map(|l| format!("{l}\n")).collect();
        if !end_nl && !ins.is_empty() {
            // no newline at the end of the file: none after its new last line either
            if b >= real && a < real {
                ins.pop();
            } else if a >= real {
                ins = format!("\n{}", ins.trim_end_matches('\n'));
            }
        }
        let (p, q) = (ed.buf.to_pos(from), ed.buf.to_pos(to));
        ed.apply_edits(vec![(p, q, ins)]);
        self.docs[di].states.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::{diff, map_line, Block};

    fn ls(s: &str) -> Vec<String> {
        s.lines().map(str::to_string).collect()
    }

    #[test]
    fn changes_between_two_texts() {
        assert_eq!(diff(&ls("a\nb\nc"), &ls("a\nb\nc"), ""), vec![]);
        assert_eq!(diff(&ls("a\nb\nc"), &ls("a\nx\nc"), ""), vec![Block { a: (1, 2), b: (1, 2) }], "a changed line");
        assert_eq!(diff(&ls("a\nc"), &ls("a\nb\nc"), ""), vec![Block { a: (1, 1), b: (1, 2) }], "a line only there");
        assert_eq!(diff(&ls("a\nb\nc"), &ls("a\nc"), ""), vec![Block { a: (1, 2), b: (1, 1) }], "a line only here");
        assert_eq!(diff(&ls("x\na\nb\ny"), &ls("a\nz\nb"), ""), vec![Block { a: (0, 1), b: (0, 0) }, Block { a: (2, 2), b: (1, 2) }, Block { a: (3, 4), b: (3, 3) }]);
        assert_eq!(diff(&ls("A  b"), &ls("a b"), "icase,iwhite"), vec![], "'diffopt' icase iwhite");
        assert_eq!(diff(&ls("ab"), &ls("a b"), "iwhiteall"), vec![]);
        // a longer one stays a shortest edit: every equal line kept
        let a: Vec<String> = (0..300).map(|i| format!("l{i}")).collect();
        let mut b = a.clone();
        b.remove(10);
        b.insert(200, "new".into());
        assert_eq!(diff(&a, &b, ""), vec![Block { a: (10, 11), b: (10, 10) }, Block { a: (201, 201), b: (200, 201) }]);
    }

    #[test]
    fn diff_mode_in_two_windows() {
        let dir = std::env::temp_dir().join(format!("zett-diff-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (a, b) = (dir.join("a.txt"), dir.join("b.txt"));
        std::fs::write(&a, "a\nb\nc\n").unwrap();
        std::fs::write(&b, "a\nx\nc\nd\n").unwrap();
        let mut app = crate::App::new(vec![a.clone()]);
        let wa = app.focus;
        app.command(&format!("diffsplit {}", b.display()));
        let wb = app.focus;
        assert_ne!(wa, wb);
        assert!(app.wins[wa].as_ref().unwrap().diff && app.wins[wb].as_ref().unwrap().diff, "both in diff mode");
        let (da, db) = (app.wins[wa].as_ref().unwrap().doc, app.cur);
        let vb = app.diff_view(wb, db).unwrap();
        assert_eq!(vb.lines.get(&1), Some(&("DiffChange", Some((0, 1)))), "x for b");
        assert_eq!(vb.lines.get(&3), Some(&("DiffAdd", None)), "d only here");
        let va = app.diff_view(wa, da).unwrap();
        assert_eq!(va.fill.get(&3), Some(&1), "a filler where d is on the other side");
        // ]c to the change; do takes the other side's line, dp gives this side's
        let keys = |app: &mut crate::App, k: &str| {
            for k in zett_edit::keys::parse(k) {
                app.key(crate::input::key_event(&k));
            }
        };
        keys(&mut app, "gg]c");
        assert_eq!(app.docs[app.cur].ed.cursor().line, 1);
        keys(&mut app, "do");
        assert_eq!(app.docs[db].ed.buf.text(), "a\nb\nc\nd\n", "do: b from the other side");
        keys(&mut app, "]c");
        assert_eq!(app.docs[app.cur].ed.cursor().line, 3);
        keys(&mut app, "dp");
        assert_eq!(app.docs[da].ed.buf.text(), "a\nb\nc\nd\n", "dp: d put into a");
        assert!(app.diff_view(wb, db).unwrap().lines.is_empty(), "no changes left");
        app.command("diffoff!");
        assert!(!app.wins[wa].as_ref().unwrap().diff && !app.wins[wb].as_ref().unwrap().diff);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn lines_map_across() {
        let blocks = vec![Block { a: (1, 1), b: (1, 3) }, Block { a: (5, 7), b: (7, 7) }];
        assert_eq!(map_line(&blocks, 0), 0);
        assert_eq!(map_line(&blocks, 1), 3, "after two lines there");
        assert_eq!(map_line(&blocks, 4), 6);
        assert_eq!(map_line(&blocks, 6), 7, "inside a part only here: where it stood");
        assert_eq!(map_line(&blocks, 9), 9);
    }
}
