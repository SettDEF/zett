//! Vim's quickfix and location lists: errorformat turns a program's output into places, a stack of
//! lists keeps the older ones (:colder :cnewer, chistory). The commands are in qfcmd.rs.

use fancy_regex::Regex;

/// One place in a list: a file, line and column (1-based, 0: none), the kind (E W I N, or ' '), its
/// number, the message; `valid` false for a line that matched nothing (kept, shown as || text).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct QfItem {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub kind: char,
    pub nr: usize,
    pub text: String,
    pub valid: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct QfList {
    pub title: String,
    pub items: Vec<QfItem>,
    /// the item last gone to
    pub idx: usize,
}

impl QfList {
    pub(crate) fn valid(&self) -> Vec<usize> {
        (0..self.items.len()).filter(|&i| self.items[i].valid).collect()
    }
}

/// The lists made so far (newest last) and the one in use.
#[derive(Clone, Debug, Default)]
pub(crate) struct QfStack {
    pub lists: Vec<QfList>,
    pub cur: usize,
}

impl QfStack {
    /// A new list after the one in use (newer ones go, as in Vim); at most `max` kept.
    pub(crate) fn push(&mut self, l: QfList, max: usize) {
        if !self.lists.is_empty() {
            self.lists.truncate(self.cur + 1);
        }
        self.lists.push(l);
        while self.lists.len() > max.max(1) {
            self.lists.remove(0);
        }
        self.cur = self.lists.len() - 1;
    }
    pub(crate) fn current(&self) -> Option<&QfList> {
        self.lists.get(self.cur)
    }
    pub(crate) fn current_mut(&mut self) -> Option<&mut QfList> {
        self.lists.get_mut(self.cur)
    }
}

/// One errorformat part made ready: its kind of line (%E %W %I %N start a message over several
/// lines, %C goes on, %Z ends, %G is any line) and whether a matched line is dropped (%-) or kept
/// whole as the message (%+).
struct Part {
    re: Regex,
    lead: char,
    drop: bool,
    whole: bool,
}

/// errorformat's comma list (\, is a comma) as parts.
fn parts(efm: &str) -> Vec<Part> {
    let mut list = Vec::new();
    let mut cur = String::new();
    let mut it = efm.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\\' if it.peek() == Some(&',') => {
                cur.push(',');
                it.next();
            }
            ',' => list.push(std::mem::take(&mut cur)),
            c => cur.push(c),
        }
    }
    list.push(cur);
    list.into_iter().filter(|p| !p.is_empty()).filter_map(|p| part(&p)).collect()
}

fn part(p: &str) -> Option<Part> {
    let mut s = p;
    let (mut lead, mut drop, mut whole) = (' ', false, false);
    // %-X / %+X / %X at the start: the line's role
    loop {
        if let Some(r) = s.strip_prefix("%-") {
            drop = true;
            s = r;
        } else if let Some(r) = s.strip_prefix("%+") {
            whole = true;
            s = r;
        } else {
            break;
        }
    }
    if let Some(r) = s.strip_prefix('%') {
        if let Some(c) = r.chars().next().filter(|c| "EWINCZGAOPQ".contains(*c)) {
            lead = c;
            s = &r[1..];
        } else if drop || whole {
            // %-/%+ before a field: put the % back
        }
    }
    let mut re = String::from("^");
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c == '%' && i + 1 < cs.len() {
            let k = cs[i + 1];
            i += 2;
            match k {
                'f' => re.push_str(r"(?P<f>(?:[A-Za-z]:)?[^\s:()\[\]]+?)"),
                'l' => re.push_str(r"(?P<l>\d+)"),
                'c' => re.push_str(r"(?P<c>\d+)"),
                'v' => re.push_str(r"(?P<v>\d+)"),
                'n' => re.push_str(r"(?P<n>\d+)"),
                't' => re.push_str(r"(?P<t>.)"),
                'm' => re.push_str(r"(?P<m>.*)"),
                'r' => re.push_str(r"(?P<r>.*)"),
                'p' => re.push_str(r"(?P<p>[-\s.]*)"),
                's' => re.push_str(r"(?P<s>.*)"),
                '%' => re.push('%'),
                '*' => {
                    // %*{conv}: a scanf-like part, matched and not kept (%*\d, %*[^:])
                    if cs.get(i) == Some(&'\\') && i + 1 < cs.len() {
                        re.push('\\');
                        re.push(cs[i + 1]);
                        re.push('*');
                        i += 2;
                    } else if cs.get(i) == Some(&'[') {
                        let end = (i..cs.len()).find(|&j| cs[j] == ']' && j > i + 1).unwrap_or(cs.len() - 1);
                        re.extend(&cs[i..=end]);
                        re.push('*');
                        i = end + 1;
                    }
                }
                '#' => re.push('*'),
                _ => {}
            }
            continue;
        }
        // Vim's own pattern pieces in errorformat: \ escapes, . * [ ] ^ $ keep their meaning
        match c {
            '\\' if i + 1 < cs.len() => {
                let n = cs[i + 1];
                if n == ' ' { re.push(' ') } else { re.push('\\'); re.push(n) }
                i += 2;
                continue;
            }
            '(' | ')' | '+' | '?' | '{' | '}' | '|' => {
                re.push('\\');
                re.push(c);
            }
            c => re.push(c),
        }
        i += 1;
    }
    if !re.contains("(?P<m>") && !re.ends_with('$') {
        re.push_str(".*");
    }
    re.push('$');
    Regex::new(&re).ok().map(|re| Part { re, lead, drop, whole })
}

/// A program's output as list items, as errorformat reads it.
pub(crate) fn parse(efm: &str, text: &str) -> Vec<QfItem> {
    let ps = parts(efm);
    let mut out: Vec<QfItem> = Vec::new();
    // a message over several lines (%E … %C … %Z) still open
    let mut open: Option<usize> = None;
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        let hit = ps.iter().find_map(|p| p.re.captures(line).ok().flatten().map(|c| (p, c)));
        let Some((p, c)) = hit else {
            open = None;
            out.push(QfItem { text: line.to_string(), ..Default::default() });
            continue;
        };
        if p.drop {
            continue;
        }
        let get = |k: &str| c.name(k).map(|m| m.as_str().to_string());
        let num = |k: &str| get(k).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
        let msg = if p.whole { line.to_string() } else { get("m").unwrap_or_default() };
        match p.lead {
            'C' | 'Z' => {
                if let Some(i) = open {
                    let it = &mut out[i];
                    if !msg.trim().is_empty() {
                        if !it.text.is_empty() {
                            it.text.push('\n');
                        }
                        it.text.push_str(msg.trim());
                    }
                    if it.file.is_empty() {
                        it.file = get("f").unwrap_or_default();
                    }
                    if it.line == 0 {
                        it.line = num("l");
                    }
                    if it.col == 0 {
                        it.col = num("c").max(get("p").map(|p| p.len() + 1).unwrap_or(0));
                    }
                    it.valid = !it.file.is_empty() || it.line > 0;
                    if p.lead == 'Z' {
                        open = None;
                    }
                }
                continue;
            }
            _ => {}
        }
        let kind = match p.lead {
            'E' | 'W' | 'I' | 'N' => p.lead,
            _ => get("t").and_then(|t| t.chars().next()).map(|t| t.to_ascii_uppercase()).unwrap_or(' '),
        };
        let file = get("f").unwrap_or_default();
        let it = QfItem { file: file.clone(), line: num("l"), col: num("c").max(num("v")), kind, nr: num("n"), text: msg, valid: !file.is_empty() || num("l") > 0 };
        out.push(it);
        open = matches!(p.lead, 'E' | 'W' | 'I' | 'N' | 'A').then_some(out.len() - 1);
    }
    out
}

/// A list item as the quickfix window shows it: file|line col N error| text.
pub(crate) fn line_of(it: &QfItem) -> String {
    if !it.valid {
        return format!("|| {}", it.text);
    }
    let kind = match it.kind {
        'E' => " error",
        'W' => " warning",
        'I' => " info",
        'N' => " note",
        _ => "",
    };
    let at = match (it.line, it.col) {
        (0, _) => String::new(),
        (l, 0) => l.to_string(),
        (l, c) => format!("{l} col {c}"),
    };
    format!("{}|{at}{kind}| {}", it.file, it.text.replace('\n', " "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errorformat_reads_compilers_and_grep() {
        let gcc = "%f:%l:%c: %t%*[^:]: %m,%f:%l: %m";
        let out = parse(gcc, "src/a.c:3:7: error: no such thing\nIn file included\nb.h:9: other\n");
        assert_eq!(out[0], QfItem { file: "src/a.c".into(), line: 3, col: 7, kind: 'E', nr: 0, text: "no such thing".into(), valid: true });
        assert!(!out[1].valid && out[1].text == "In file included");
        assert_eq!((out[2].file.as_str(), out[2].line, out[2].text.as_str()), ("b.h", 9, "other"));
        // rustc: a message over lines (%E … %C --> file:line:col … %Z)
        let rust = "%Eerror%m,%C%*\\s--> %f:%l:%c,%-G%.%#";
        let out = parse(rust, "error[E0425]: cannot find x\n  --> src/main.rs:4:5\nnoise\n");
        assert_eq!(out.len(), 1, "{out:?}");
        assert_eq!((out[0].file.as_str(), out[0].line, out[0].col, out[0].kind), ("src/main.rs", 4, 5, 'E'));
        assert!(out[0].text.contains("cannot find x"));
        let grep = parse("%f:%l:%c:%m,%f:%l:%m", "a.rs:10:3:let x\nb.rs:2:fn y\n");
        assert_eq!((grep[0].col, grep[1].line, grep[1].text.as_str()), (3, 2, "fn y"));
        assert_eq!(line_of(&grep[0]), "a.rs|10 col 3| let x");
    }

    #[test]
    fn the_stack_keeps_older_lists() {
        let mut s = QfStack::default();
        for t in ["a", "b", "c"] {
            s.push(QfList { title: t.into(), ..Default::default() }, 2);
        }
        assert_eq!(s.lists.iter().map(|l| l.title.as_str()).collect::<Vec<_>>(), ["b", "c"], "chistory 2");
        s.cur = 0;
        s.push(QfList { title: "d".into(), ..Default::default() }, 10);
        assert_eq!(s.lists.iter().map(|l| l.title.as_str()).collect::<Vec<_>>(), ["b", "d"], "a new list drops the newer ones");
    }
}
