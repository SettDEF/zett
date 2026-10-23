//! Vim's patterns (`:help pattern`) into fancy-regex: the four magic levels (\v \m \M \V and the
//! `magic` option), groups \( \%( \), branches \|, multis * \+ \= \? \{n,m} \{-}, classes \s \d \w
//! \a \l \u \x \h \k … and \_x, \< \>, \zs \ze, lookaround \@= \@! \@<= \@<! \@>, \%[ ] \%^ \%$,
//! collections [ ], \c \C. One translator for /, :s, :g, matches and highlights.

use std::sync::atomic::{AtomicBool, Ordering};

static MAGIC: AtomicBool = AtomicBool::new(true);

/// The `magic` option (patterns without \v \m \M \V start in this level).
pub fn set_magic(on: bool) {
    MAGIC.store(on, Ordering::Relaxed);
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Level {
    VeryMagic,
    Magic,
    NoMagic,
    VeryNoMagic,
}

/// A compiled Vim pattern. `zs`: the group holding the match when the pattern has \zs or \ze.
#[derive(Debug)]
pub struct VimRegex {
    pub re: fancy_regex::Regex,
    pub zs: Option<usize>,
}

impl VimRegex {
    /// Vim's group `n` (\1 …) in the regex (the \zs group, when there is one, shifts those after it).
    pub fn group(&self, n: usize) -> usize {
        match self.zs {
            Some(z) if n >= z => n + 1,
            _ => n,
        }
    }

    /// The first match at or after byte `from`: (start, end) in bytes, \zs \ze applied.
    pub fn find_at(&self, text: &str, from: usize) -> Option<(usize, usize)> {
        match self.zs {
            None => self.re.find_from_pos(text, from).ok().flatten().map(|m| (m.start(), m.end())),
            Some(g) => {
                // the whole match may start before \zs (on this line): from the line's start, until the
                // part counted is at or past `from`
                let mut at = text[..from.min(text.len())].rfind('\n').map_or(0, |n| n + 1);
                while at <= text.len() {
                    let c = self.re.captures_from_pos(text, at).ok().flatten()?;
                    let m = c.get(g).or_else(|| c.get(0))?;
                    if m.start() >= from {
                        return Some((m.start(), m.end()));
                    }
                    let whole = c.get(0)?;
                    at = next_char(text, whole.start());
                }
                None
            }
        }
    }

    /// Every match (start, end) in bytes, \zs \ze applied, not overlapping.
    pub fn find_all(&self, text: &str) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut at = 0;
        while at <= text.len() {
            let Some((s, e)) = self.find_at(text, at) else { break };
            out.push((s, e));
            at = if e > s { e } else { next_char(text, e) };
        }
        out
    }
}

fn next_char(text: &str, i: usize) -> usize {
    text[i..].chars().next().map_or(text.len() + 1, |c| i + c.len_utf8())
}

/// Ignore case for `p`? `\c` / `\C` in it decide, else ignorecase, with smartcase off when it has
/// a capital (not one after a backslash).
pub fn ignores_case(p: &str, ignorecase: bool, smartcase: bool) -> bool {
    let mut it = p.chars();
    let (mut upper, mut forced) = (false, None);
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('c') => forced = Some(true),
                Some('C') => forced = forced.or(Some(false)),
                _ => {}
            }
        } else if c.is_uppercase() {
            upper = true;
        }
    }
    forced.unwrap_or(ignorecase && !(smartcase && upper))
}

/// Compiles `p` (case as [`ignores_case`] says).
pub fn compile(p: &str, ignorecase: bool, smartcase: bool) -> Result<VimRegex, String> {
    let (src, zs) = translate(p)?;
    let src = if ignores_case(p, ignorecase, smartcase) { format!("(?i){src}") } else { src };
    let re = fancy_regex::Regex::new(&src).map_err(|e| format!("E486: bad pattern: {e}"))?;
    Ok(VimRegex { re, zs })
}

/// `c` as a regex literal.
fn lit(out: &mut String, c: char) {
    if "\\.+*?()|[]{}^$#&-~".contains(c) {
        out.push('\\');
    }
    out.push(c);
}

/// A class letter after a backslash (\s \d \a …), as regex; None for another letter.
fn class(c: char) -> Option<&'static str> {
    Some(match c {
        's' => "[ \\t]",
        'S' => "[^ \\t\\n]",
        'd' => "[0-9]",
        'D' => "[^0-9\\n]",
        'w' => "[0-9A-Za-z_]",
        'W' => "[^0-9A-Za-z_\\n]",
        'a' => "[A-Za-z]",
        'A' => "[^A-Za-z\\n]",
        'l' => "[a-z]",
        'L' => "[^a-z\\n]",
        'u' => "[A-Z]",
        'U' => "[^A-Z\\n]",
        'x' => "[0-9A-Fa-f]",
        'X' => "[^0-9A-Fa-f\\n]",
        'o' => "[0-7]",
        'O' => "[^0-7\\n]",
        'h' => "[A-Za-z_]",
        'H' => "[^A-Za-z_\\n]",
        // keyword, identifier: letters (any script), digits, _
        'k' | 'i' => "[\\w]",
        'K' | 'I' => "[^\\W\\d]",
        'f' => "[\\w./\\-~+,#$%]",
        'F' => "[^\\W\\d./\\-~+,#$%]",
        'p' => "[^\\x00-\\x1f\\x7f]",
        'P' => "[^\\x00-\\x1f\\x7f0-9]",
        _ => return None,
    })
}

/// Is `c` an operator here (rather than itself), given the level and whether a backslash came first?
fn op(c: char, level: Level, escaped: bool) -> bool {
    const ALWAYS_BARE_IN_MAGIC: &str = ".*[~";
    const ANCHORS: &str = "^$";
    const OPS: &str = "()|+?={}@%<>*.[~^$";
    if !OPS.contains(c) {
        return false;
    }
    match level {
        Level::VeryMagic => !escaped,
        Level::Magic => {
            if ALWAYS_BARE_IN_MAGIC.contains(c) || ANCHORS.contains(c) {
                !escaped
            } else {
                escaped
            }
        }
        Level::NoMagic => {
            if ANCHORS.contains(c) {
                !escaped
            } else {
                escaped
            }
        }
        Level::VeryNoMagic => escaped,
    }
}

/// The pattern as regex source, and the \zs group's number (if any).
pub fn translate(p: &str) -> Result<(String, Option<usize>), String> {
    let chars: Vec<char> = p.chars().collect();
    let mut level = if MAGIC.load(Ordering::Relaxed) { Level::Magic } else { Level::NoMagic };
    let mut out = String::new();
    // where the last atom starts in `out` (a multi or \@ applies to it), open groups' starts
    let mut atom: Option<usize> = None;
    let mut groups: Vec<usize> = Vec::new();
    let mut ngroups = 0usize;
    let (mut zs, mut zs_open, mut ze_open) = (None, false, false);
    let mut i = 0;
    // at the start of the pattern or a branch (^ anchors there only)
    let branch_start = |out: &str| out.is_empty() || out.ends_with('(') || out.ends_with('|') || out.ends_with("(?:") || out.ends_with("(?P<zs>");
    while i < chars.len() {
        let c = chars[i];
        let escaped = c == '\\';
        let (c, next) = if escaped {
            match chars.get(i + 1) {
                Some(&n) => (n, i + 2),
                None => {
                    out.push_str("\\\\");
                    break;
                }
            }
        } else {
            (c, i + 1)
        };
        i = next;
        if escaped {
            // backslash meanings that do not depend on the level
            match c {
                'v' => {
                    level = Level::VeryMagic;
                    continue;
                }
                'm' => {
                    level = Level::Magic;
                    continue;
                }
                'M' => {
                    level = Level::NoMagic;
                    continue;
                }
                'V' => {
                    level = Level::VeryNoMagic;
                    continue;
                }
                'c' | 'C' => continue,
                'n' => {
                    atom = Some(out.len());
                    out.push_str("\\n");
                    continue;
                }
                't' => {
                    atom = Some(out.len());
                    out.push_str("\\t");
                    continue;
                }
                'e' => {
                    atom = Some(out.len());
                    out.push_str("\\x1b");
                    continue;
                }
                'r' => {
                    atom = Some(out.len());
                    out.push_str("\\r");
                    continue;
                }
                '1'..='9' => {
                    atom = Some(out.len());
                    out.push('\\');
                    out.push(c);
                    continue;
                }
                'z' => {
                    match chars.get(i) {
                        Some('s') if !zs_open && !ze_open => {
                            ngroups += 1;
                            zs = Some(ngroups);
                            zs_open = true;
                            out.push_str("(?P<zs>");
                        }
                        Some('e') if !ze_open => {
                            if !zs_open {
                                // the match starts with the pattern: its group opens at the start
                                ngroups += 1;
                                zs = Some(1);
                                out.insert_str(0, "(?P<zs>");
                                groups.iter_mut().for_each(|g| *g += 7);
                            }
                            out.push_str(")(?=");
                            zs_open = false;
                            ze_open = true;
                        }
                        _ => {}
                    }
                    i += 1;
                    atom = None;
                    continue;
                }
                '_' => {
                    // \_x: x and a line break as well
                    let Some(&k) = chars.get(i) else { return Err("E769: \\_ at the end".into()) };
                    i += 1;
                    atom = Some(out.len());
                    match k {
                        '.' => out.push_str("(?s:.)"),
                        '^' => out.push('^'),
                        '$' => out.push('$'),
                        '[' => {
                            let (body, used) = collection(&chars[i..]).ok_or("E769: Missing ] after \\_[")?;
                            i += used;
                            // the class with \n added
                            out.push_str(&format!("(?:{body}|\\n)"));
                        }
                        k => match class(k) {
                            Some(cl) => out.push_str(&format!("(?:{cl}|\\n)")),
                            None => return Err(format!("E71: Invalid character after \\_ : {k}")),
                        },
                    }
                    continue;
                }
                k if class(k).is_some() => {
                    atom = Some(out.len());
                    out.push_str(class(k).unwrap());
                    continue;
                }
                _ => {}
            }
        }
        if !op(c, level, escaped) {
            atom = Some(out.len());
            lit(&mut out, c);
            continue;
        }
        match c {
            '^' => {
                if branch_start(&out) {
                    out.push('^');
                    atom = None;
                } else {
                    atom = Some(out.len());
                    lit(&mut out, '^');
                }
            }
            '$' => {
                // an anchor at the end of the pattern or a branch, else itself
                let rest: String = chars[i..].iter().collect();
                let ends = rest.is_empty() || rest.starts_with("\\|") || rest.starts_with("\\)") || (level == Level::VeryMagic && (rest.starts_with('|') || rest.starts_with(')'))) || rest.starts_with("\\ze");
                if ends {
                    out.push('$');
                    atom = None;
                } else {
                    atom = Some(out.len());
                    lit(&mut out, '$');
                }
            }
            '.' => {
                atom = Some(out.len());
                out.push('.');
            }
            '~' => {
                // the last substitute string is not kept here: an empty match
                atom = None;
            }
            '[' => match collection(&chars[i..]) {
                Some((body, used)) => {
                    i += used;
                    atom = Some(out.len());
                    out.push_str(&body);
                }
                None => {
                    atom = Some(out.len());
                    lit(&mut out, '[');
                }
            },
            '(' => {
                groups.push(out.len());
                ngroups += 1;
                out.push('(');
                atom = None;
            }
            ')' => {
                let Some(s) = groups.pop() else { return Err("E54: Unmatched \\)".into()) };
                out.push(')');
                atom = Some(s);
            }
            '|' => {
                out.push('|');
                atom = None;
            }
            '<' | '>' => {
                out.push_str("\\b");
                atom = None;
            }
            '*' | '+' | '?' | '=' => {
                let Some(_) = atom else {
                    // nothing before it: itself (as Vim reads a leading *)
                    atom = Some(out.len());
                    lit(&mut out, c);
                    continue;
                };
                out.push(match c {
                    '*' => '*',
                    '+' => '+',
                    _ => '?',
                });
                atom = None;
            }
            '{' => {
                // \{n,m} \{-n,m} \{n} \{n,} \{,m} \{} \{-}: up to the closing } (or \})
                let mut body = String::new();
                while i < chars.len() && chars[i] != '}' {
                    if chars[i] != '\\' {
                        body.push(chars[i]);
                    }
                    i += 1;
                }
                if i >= chars.len() {
                    return Err("E554: Syntax error in \\{...}".into());
                }
                i += 1;
                if atom.is_none() {
                    return Err("E64: \\{ follows nothing".into());
                }
                let lazy = body.starts_with('-');
                let b = body.trim_start_matches('-');
                let q = match b.split_once(',') {
                    None if b.is_empty() => "*".to_string(),
                    None => format!("{{{b}}}"),
                    Some(("", "")) => "*".to_string(),
                    Some(("", m)) => format!("{{0,{m}}}"),
                    Some((n, m)) => format!("{{{n},{m}}}"),
                };
                out.push_str(&q);
                if lazy {
                    out.push('?');
                }
                atom = None;
            }
            '}' => lit(&mut out, '}'),
            '@' => {
                // \@= \@! \@<= \@<! \@>: lookaround on the atom before
                let Some(s) = atom else { return Err("E64: \\@ follows nothing".into()) };
                while chars.get(i).is_some_and(|c| c.is_ascii_digit()) {
                    i += 1;
                }
                let kind = match (chars.get(i), chars.get(i + 1)) {
                    (Some('='), _) => (1, "(?="),
                    (Some('!'), _) => (1, "(?!"),
                    (Some('<'), Some('=')) => (2, "(?<="),
                    (Some('<'), Some('!')) => (2, "(?<!"),
                    (Some('>'), _) => (1, "(?>"),
                    _ => return Err("E64: invalid \\@".into()),
                };
                i += kind.0;
                let inner = out.split_off(s);
                out.push_str(kind.1);
                out.push_str(&inner);
                out.push(')');
                atom = None;
            }
            '%' => {
                let Some(&k) = chars.get(i) else { return Err("E71: \\% at the end".into()) };
                i += 1;
                match k {
                    '(' => {
                        groups.push(out.len());
                        out.push_str("(?:");
                        atom = None;
                    }
                    '^' => out.push_str("\\A"),
                    '$' => out.push_str("\\z"),
                    // the Visual area, the cursor, marks: not known to a pattern here (empty)
                    'V' | '#' => {}
                    'd' | 'x' | 'o' | 'u' | 'U' => {
                        let radix = match k {
                            'd' => 10,
                            'o' => 8,
                            _ => 16,
                        };
                        let mut num = String::new();
                        while chars.get(i).is_some_and(|c| c.is_digit(radix)) {
                            num.push(chars[i]);
                            i += 1;
                        }
                        let ch = u32::from_str_radix(&num, radix).ok().and_then(char::from_u32).ok_or("E678: Invalid character after \\%[dxouU]")?;
                        atom = Some(out.len());
                        lit(&mut out, ch);
                    }
                    '[' => {
                        // \%[abc]: as much of the sequence as is there
                        let mut seq = Vec::new();
                        while i < chars.len() && chars[i] != ']' {
                            seq.push(chars[i]);
                            i += 1;
                        }
                        if i >= chars.len() {
                            return Err("E69: Missing ] after \\%[".into());
                        }
                        i += 1;
                        atom = Some(out.len());
                        let mut s = String::new();
                        for c in seq.iter().rev() {
                            let mut l = String::new();
                            lit(&mut l, *c);
                            s = format!("(?:{l}{s})?");
                        }
                        out.push_str(&s);
                    }
                    k => return Err(format!("E71: Invalid character after \\%: {k}")),
                }
            }
            c => {
                atom = Some(out.len());
                lit(&mut out, c);
            }
        }
    }
    if !groups.is_empty() {
        return Err("E54: Unmatched \\(".into());
    }
    if zs_open || ze_open {
        out.push(')');
    }
    Ok((out, zs))
}

/// A collection `[...]` (after its `[`): its regex and how many chars it took (with the `]`).
fn collection(rest: &[char]) -> Option<(String, usize)> {
    let mut out = String::from("[");
    let mut i = 0;
    if rest.first() == Some(&'^') {
        out.push('^');
        i += 1;
    }
    // a ] first is itself
    if rest.get(i) == Some(&']') {
        out.push_str("\\]");
        i += 1;
    }
    while i < rest.len() {
        let c = rest[i];
        match c {
            ']' => {
                out.push(']');
                return Some((out, i + 1));
            }
            '[' if rest.get(i + 1) == Some(&':') => {
                // [:alpha:] as it is
                let end = (i + 2..rest.len().saturating_sub(1)).find(|&j| rest[j] == ':' && rest[j + 1] == ']')?;
                out.extend(&rest[i..end + 2]);
                i = end + 2;
                continue;
            }
            '\\' => {
                match rest.get(i + 1) {
                    Some('e') => out.push_str("\\x1b"),
                    Some('t') => out.push_str("\\t"),
                    Some('r') => out.push_str("\\r"),
                    Some('n') => out.push_str("\\n"),
                    Some('\\') => out.push_str("\\\\"),
                    Some(&o @ (']' | '^' | '-')) => {
                        out.push('\\');
                        out.push(o);
                    }
                    // a backslash before anything else is itself
                    Some(&o) => {
                        out.push_str("\\\\");
                        lit(&mut out, o);
                    }
                    None => return None,
                }
                i += 2;
                continue;
            }
            '[' | '&' | '~' => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(p: &str, text: &str) -> Option<String> {
        let r = compile(p, false, false).unwrap_or_else(|e| panic!("{p}: {e}"));
        r.find_at(text, 0).map(|(s, e)| text[s..e].to_string())
    }

    #[test]
    fn magic_levels() {
        // magic: ( | + are themselves, \( \| \+ operators
        assert_eq!(m("a(b)", "xa(b)"), Some("a(b)".into()));
        assert_eq!(m(r"\(ab\)\+", "ababx"), Some("abab".into()));
        assert_eq!(m(r"foo\|bar", "xbar"), Some("bar".into()));
        assert_eq!(m("l.ne 2", "on line 2"), Some("line 2".into()));
        // very magic: everything is an operator
        assert_eq!(m(r"\v(ab)+", "ababx"), Some("abab".into()));
        assert_eq!(m(r"\vfoo|bar", "xbar"), Some("bar".into()));
        // nomagic and very nomagic: . is itself
        assert_eq!(m(r"\Ma.c", "abc a.c"), Some("a.c".into()));
        assert_eq!(m(r"\Va.c", "abc a.c"), Some("a.c".into()));
        assert_eq!(m(r"\Va\.c", "abc a.c"), Some("abc".into()));
        // the option
        set_magic(false);
        assert_eq!(m("a.c", "abc a.c"), Some("a.c".into()));
        set_magic(true);
    }

    #[test]
    fn multis_classes_and_anchors() {
        assert_eq!(m(r"a\{2,3}", "aaaa"), Some("aaa".into()));
        assert_eq!(m(r"a\{-1,}", "aaaa"), Some("a".into()));
        assert_eq!(m(r"x\=y", "y"), Some("y".into()));
        assert_eq!(m(r"\d\+", "ab 123"), Some("123".into()));
        assert_eq!(m(r"\<on\>", "one on"), Some("on".into()));
        assert_eq!(m(r"^so", "some"), Some("so".into()));
        assert_eq!(m(r"a^b", "a^b"), Some("a^b".into()));
        assert_eq!(m(r"e$", "the\nfile"), Some("e".into()));
        assert_eq!(m(r"a\_sb", "a\nb"), Some("a\nb".into()));
        assert_eq!(m(r"[0-9]\+", "x42"), Some("42".into()));
        assert_eq!(m(r"\%[abc]x", "abx"), Some("abx".into()));
        assert_eq!(m(r"\%(ab\)\+", "ababx"), Some("abab".into()));
        assert_eq!(m(r"\%d65", "xA"), Some("A".into()));
    }

    #[test]
    fn zs_ze_and_lookaround() {
        assert_eq!(m(r"foo\zsbar", "foobar"), Some("bar".into()));
        assert_eq!(m(r"foo\zebar", "foobar foo"), Some("foo".into()));
        assert_eq!(m(r"a\zsb\zec", "abc"), Some("b".into()));
        assert_eq!(m(r"\(foo\)\@<=bar", "xbar foobar"), Some("bar".into()));
        assert_eq!(m(r"foo\(bar\)\@!", "foobar food"), Some("foo".into()));
        // a match whose \zs part is past where the search starts
        let r = compile(r"a\zsb", false, false).unwrap();
        assert_eq!(r.find_at("ab ab", 1), Some((1, 2)));
        assert_eq!(r.find_at("ab ab", 2), Some((4, 5)));
    }

    #[test]
    fn case() {
        assert!(ignores_case("foo", true, true));
        assert!(!ignores_case("Foo", true, true));
        assert!(ignores_case("Foo", true, false));
        assert!(ignores_case(r"Foo\c", false, false));
        assert!(!ignores_case(r"foo\C", true, false));
        assert!(ignores_case(r"\Sfoo", true, true), "a capital after a backslash is no capital");
        assert_eq!(m(r"\cFOO", "a foo"), Some("foo".into()));
    }
}

