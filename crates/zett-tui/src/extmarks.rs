//! Neovim's extmarks as zett draws them: a plugin's highlights over text, its signs in the gutter,
//! a line's colour and virtual text after a line (vim.lua keeps them; read here when they change).

use super::*;

/// One mark as drawn (rows 0-based, columns in bytes; end_col -1: a mark at one place).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ExtMark {
    pub row: usize,
    pub col: usize,
    pub end_row: usize,
    pub end_col: i64,
    pub priority: i64,
    pub hl: String,
    pub line_hl: String,
    pub sign: String,
    pub sign_hl: String,
    pub num_hl: String,
    pub virt_pos: String,
    pub virt: Vec<(String, String)>,
    /// conceal: the text hidden (Some(''): nothing in its place, else that character)
    pub conceal: Option<String>,
}

/// vim._marks_for's lines into marks.
pub(crate) fn parse(s: &str) -> Vec<ExtMark> {
    s.split('\u{1e}').filter(|l| !l.is_empty()).filter_map(|l| {
        let f: Vec<&str> = l.split('\t').collect();
        let n = |i: usize| f.get(i).and_then(|v| v.parse::<i64>().ok());
        let t = |i: usize| f.get(i).copied().unwrap_or("").to_string();
        let vt: Vec<&str> = f.get(11).map(|v| v.split('\u{1f}').collect()).unwrap_or_default();
        let virt = vt.chunks(2).filter(|c| !c[0].is_empty()).map(|c| (c[0].to_string(), c.get(1).copied().unwrap_or("").to_string())).collect();
        Some(ExtMark { row: n(0)? as usize, col: n(1)? as usize, end_row: n(2)? as usize, end_col: n(3)?, priority: n(4).unwrap_or(4096), hl: t(5), line_hl: t(6), sign: t(7), sign_hl: t(8), num_hl: t(9), virt_pos: t(10), virt, conceal: f.get(12).and_then(|c| c.strip_prefix('=')).map(str::to_string) })
    }).collect()
}

/// A byte column of `line` as a character index.
fn char_at(line: &str, byte: usize) -> usize {
    line.char_indices().take_while(|(b, _)| *b < byte).count()
}

/// The highlighted ranges on line `ln` (text `line`): (first char, past the last char, group,
/// priority), low priority first so the higher ones are laid over them.
pub(crate) fn line_ranges(marks: &[ExtMark], ln: usize, line: &str) -> Vec<(usize, usize, String, i64)> {
    let len = line.trim_end_matches('\n').len();
    let mut out: Vec<(usize, usize, String, i64)> = marks
        .iter()
        .filter(|m| !m.hl.is_empty() && m.end_col >= 0 && m.row <= ln && ln <= m.end_row)
        .filter_map(|m| {
            let a = if ln == m.row { m.col } else { 0 };
            let b = if ln == m.end_row { (m.end_col as usize).min(len) } else { len };
            (b > a || (m.end_row > ln)).then(|| (char_at(line, a), char_at(line, b.max(a)), m.hl.clone(), m.priority))
        })
        .collect();
    out.sort_by_key(|r| r.3);
    out
}

impl App {
    /// Buffer `di`'s marks on rows `from..to`, read again only when marks changed or the rows did.
    pub(crate) fn ext_marks(&mut self, di: usize, from: usize, to: usize) {
        let seq = self.with_lua(|e| e.eval("return vim._marks_seq()")).map(|r| r.0).unwrap_or_default();
        let key = format!("{seq}|{from}|{to}");
        if self.extmarks.get(&di).is_some_and(|(k, _)| *k == key) {
            return;
        }
        if seq.trim() == "0" {
            self.extmarks.insert(di, (key, Vec::new()));
            return;
        }
        let out = self.with_lua(|e| e.eval(&format!("return vim._marks_for({}, {from}, {to})", di + 1))).map(|r| r.0).unwrap_or_default();
        self.extmarks.insert(di, (key, parse(&out)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marks_read_and_ranges_on_a_line() {
        let m = parse("0\t1\t0\t5\t4096\tSearch\t\t\t\t\teol\t\x1e1\t0\t1\t-1\t10\t\t\tE>\tError\t\teol\tnote\x1fComment\x1ftwo\x1f");
        assert_eq!(m.len(), 2);
        assert_eq!((m[1].sign.as_str(), m[1].virt.len()), ("E>", 2));
        assert_eq!(m[1].virt[0], ("note".into(), "Comment".into()));
        // bytes 1..5 of "héllo" (é is two bytes): chars 1..4, "éll"
        let r = line_ranges(&m, 0, "héllo\n");
        assert_eq!(r, vec![(1, 4, "Search".to_string(), 4096)]);
        assert!(line_ranges(&m, 1, "x\n").is_empty(), "a mark at one place colours nothing");
    }
}

/// The concealed ranges on line `ln` (chars): (first, past the last, what shows instead).
pub(crate) fn conceal_ranges(marks: &[ExtMark], ln: usize, line: &str) -> Vec<(usize, usize, String)> {
    let len = line.trim_end_matches('\n').len();
    marks
        .iter()
        .filter(|m| m.conceal.is_some() && m.end_col >= 0 && m.row <= ln && ln <= m.end_row)
        .map(|m| {
            let a = if ln == m.row { m.col } else { 0 };
            let b = if ln == m.end_row { (m.end_col as usize).min(len) } else { len };
            (char_at(line, a), char_at(line, b.max(a)), m.conceal.clone().unwrap_or_default())
        })
        .filter(|r| r.1 > r.0)
        .collect()
}
