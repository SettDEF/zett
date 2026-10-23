//! LSP snippets (`foo(${1:x}, $2)$0`): the text and where its stops are.

/// A snippet read: its text, and each stop as (number, start, end) in characters of that text.
#[derive(Debug, PartialEq)]
pub struct Snippet {
    pub text: String,
    pub stops: Vec<(u32, usize, usize)>,
}

impl Snippet {
    /// The stops in jump order: 1, 2, … then 0 (the end; at the text's end when it has none).
    /// A number used twice (a mirror) jumps once, to its first place.
    pub fn order(&self) -> Vec<(usize, usize)> {
        let mut nums: Vec<u32> = self.stops.iter().map(|s| s.0).filter(|&n| n != 0).collect();
        nums.sort();
        nums.dedup();
        let mut out: Vec<(usize, usize)> = nums.iter().filter_map(|n| self.stops.iter().find(|s| s.0 == *n).map(|s| (s.1, s.2))).collect();
        let end = self.text.chars().count();
        out.push(self.stops.iter().find(|s| s.0 == 0).map(|s| (s.1, s.2)).unwrap_or((end, end)));
        out
    }
}

/// Reads a snippet; `var` gives a variable's value ($TM_FILENAME …), None leaves its default.
pub fn parse(s: &str, var: &dyn Fn(&str) -> Option<String>) -> Snippet {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Snippet { text: String::new(), stops: Vec::new() };
    let mut i = 0;
    let mut len = 0;
    body(&chars, &mut i, &mut out, &mut len, var, false);
    out
}

fn push(out: &mut Snippet, len: &mut usize, c: char) {
    out.text.push(c);
    *len += 1;
}

/// Reads until the end, or a `}` closing the placeholder we are in.
fn body(c: &[char], i: &mut usize, out: &mut Snippet, len: &mut usize, var: &dyn Fn(&str) -> Option<String>, inner: bool) {
    while *i < c.len() {
        match c[*i] {
            '\\' if *i + 1 < c.len() && matches!(c[*i + 1], '$' | '}' | '\\' | ',' | '|') => {
                push(out, len, c[*i + 1]);
                *i += 2;
            }
            '}' if inner => return,
            '$' => {
                *i += 1;
                // $1, $0
                if *i < c.len() && c[*i].is_ascii_digit() {
                    let n = number(c, i);
                    out.stops.push((n, *len, *len));
                    continue;
                }
                // $NAME
                if *i < c.len() && (c[*i].is_ascii_alphabetic() || c[*i] == '_') {
                    let name = ident(c, i);
                    for ch in var(&name).unwrap_or_default().chars() {
                        push(out, len, ch);
                    }
                    continue;
                }
                if *i < c.len() && c[*i] == '{' {
                    *i += 1;
                    if *i < c.len() && c[*i].is_ascii_digit() {
                        let n = number(c, i);
                        let start = *len;
                        match c.get(*i) {
                            // ${1:default}, which can hold more stops
                            Some(':') => {
                                *i += 1;
                                body(c, i, out, len, var, true);
                            }
                            // ${1|one,two|}: the first choice
                            Some('|') => {
                                *i += 1;
                                let mut first = true;
                                while *i < c.len() && !(c[*i] == '|' && c.get(*i + 1) == Some(&'}')) {
                                    if c[*i] == ',' {
                                        first = false;
                                    } else if first {
                                        let ch = if c[*i] == '\\' && *i + 1 < c.len() {
                                            *i += 1;
                                            c[*i]
                                        } else {
                                            c[*i]
                                        };
                                        push(out, len, ch);
                                    }
                                    *i += 1;
                                }
                                *i += 1;
                            }
                            _ => {}
                        }
                        out.stops.push((n, start, *len));
                    } else {
                        // ${NAME} or ${NAME:default} (transforms are left out)
                        let name = ident(c, i);
                        match var(&name) {
                            Some(v) => {
                                for ch in v.chars() {
                                    push(out, len, ch);
                                }
                                skip_to_close(c, i);
                                continue;
                            }
                            None if c.get(*i) == Some(&':') => {
                                *i += 1;
                                body(c, i, out, len, var, true);
                            }
                            None => {}
                        }
                    }
                    skip_to_close(c, i);
                    continue;
                }
                push(out, len, '$');
            }
            ch => {
                push(out, len, ch);
                *i += 1;
            }
        }
    }
}

fn number(c: &[char], i: &mut usize) -> u32 {
    let mut n = 0u32;
    while *i < c.len() && c[*i].is_ascii_digit() {
        n = n.saturating_mul(10).saturating_add(c[*i] as u32 - '0' as u32);
        *i += 1;
    }
    n
}

fn ident(c: &[char], i: &mut usize) -> String {
    let mut s = String::new();
    while *i < c.len() && (c[*i].is_ascii_alphanumeric() || c[*i] == '_') {
        s.push(c[*i]);
        *i += 1;
    }
    s
}

/// Past the `}` that ends this ${…} (nested braces counted).
fn skip_to_close(c: &[char], i: &mut usize) {
    let mut depth = 1;
    while *i < c.len() {
        match c[*i] {
            '\\' => *i += 1,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    *i += 1;
                    return;
                }
            }
            _ => {}
        }
        *i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(s: &str) -> Snippet {
        parse(s, &|v| (v == "TM_FILENAME").then(|| "a.rs".to_string()))
    }

    #[test]
    fn stops_and_text() {
        let s = p("push(${1:value})$0");
        assert_eq!(s.text, "push(value)");
        assert_eq!(s.order(), vec![(5, 10), (11, 11)]);
        let s = p("fn ${1:name}(${2}) {\n\t$0\n}");
        assert_eq!(s.text, "fn name() {\n\t\n}");
        assert_eq!(s.order(), vec![(3, 7), (8, 8), (13, 13)]);
    }

    #[test]
    fn nested_choice_variables_escapes() {
        let s = p("${1:a ${2:b}} ${3|x,y|} $TM_FILENAME ${NOPE:dflt} \\$1 \\}");
        assert_eq!(s.text, "a b x a.rs dflt $1 }");
        assert_eq!(s.order(), vec![(0, 3), (2, 3), (4, 5), (20, 20)]);
        // no $0: the end; a mirror jumps once
        assert_eq!(p("$1 = $1").order(), vec![(0, 0), (3, 3)]);
    }
}

#[cfg(test)]
mod probe_cw {
    #[test]
    fn insert_ctrl_w() {
        let mut e = crate::Editor::from_str("\n");
        e.feed("ifoo bar<C-w>");
        assert_eq!(e.text(), "foo \n");
    }
}

#[cfg(test)]
mod insert_ctrl_u {
    #[test]
    fn ctrl_u_back_to_where_typing_started() {
        let mut e = crate::Editor::from_str("    keep \n");
        e.feed("Atyped<C-u>");
        assert_eq!(e.text(), "    keep \n");
        e.feed("<C-u>");
        assert_eq!(e.text(), "    \n");
        let mut e = crate::Editor::from_str("\n");
        e.feed("ia.b  <C-w>");
        assert_eq!(e.text(), "a.\n");
    }
}
