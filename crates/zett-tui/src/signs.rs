//! :sign as Vim has it: define (text texthl linehl numhl), undefine, list, place (id line name
//! priority, file= or buffer=), unplace (an id, or *), jump. Placed signs are drawn in the sign
//! column (signcolumn=number: in the numbers), over a diagnostic when their priority is above 10.
//! They stay on their line number (Vim's move with the text).

use super::*;
use ratatui::style::Style;

/// A sign's look (:sign define).
#[derive(Clone, Debug, Default)]
pub(crate) struct SignDef {
    pub text: String,
    pub texthl: String,
    pub linehl: String,
    pub numhl: String,
}

/// A sign placed in a buffer.
#[derive(Clone, Debug)]
pub(crate) struct Placed {
    pub id: u32,
    pub line: usize,
    pub name: String,
    pub priority: i32,
}

/// What a line shows of its signs: the text (two cells), its group, the line's and the number's.
pub(crate) struct Shown {
    pub text: String,
    pub texthl: Style,
    pub linehl: Option<Style>,
    pub numhl: Option<Style>,
    pub priority: i32,
}

/// `key=value` words of a :sign line.
fn kv(words: &[&str], key: &str) -> Option<String> {
    words.iter().find_map(|w| w.strip_prefix(key)?.strip_prefix('=').map(str::to_string))
}

impl App {
    /// Each line's sign in buffer `di` (the highest priority there), for drawing.
    pub(crate) fn sign_rows(&self, di: usize) -> std::collections::HashMap<usize, Shown> {
        let mut out: std::collections::HashMap<usize, Shown> = std::collections::HashMap::new();
        for p in &self.docs[di].signs {
            let Some(def) = self.sign_defs.get(&p.name) else { continue };
            if out.get(&p.line).is_some_and(|s| s.priority >= p.priority) {
                continue;
            }
            let group = |g: &str| (!g.is_empty()).then(|| hl::hl(g));
            let text: String = format!("{:<2}", def.text).chars().take(2).collect();
            out.insert(p.line, Shown { text, texthl: group(&def.texthl).unwrap_or_else(|| hl::hl("SignColumn")), linehl: group(&def.linehl), numhl: group(&def.numhl), priority: p.priority });
        }
        out
    }

    /// The buffer a :sign line names (file= or buffer=), else the current one.
    fn sign_doc(&self, words: &[&str]) -> Option<usize> {
        if let Some(f) = kv(words, "file") {
            let p = self.root.join(&f);
            let real = std::fs::canonicalize(&p).ok();
            return self.docs.iter().position(|d| d.name == f || (real.is_some() && d.abs_path().and_then(|a| std::fs::canonicalize(a).ok()) == real));
        }
        if let Some(b) = kv(words, "buffer") {
            return b.parse::<usize>().ok().and_then(|n| n.checked_sub(1)).filter(|i| *i < self.docs.len());
        }
        Some(self.cur)
    }

    pub(crate) fn sign_command(&mut self, arg: &str) {
        let words: Vec<&str> = arg.split_whitespace().collect();
        match words.first().copied().unwrap_or("") {
            "define" => {
                let Some(name) = words.get(1) else { return self.say("E156: Missing sign name") };
                let def = self.sign_defs.entry(name.to_string()).or_default();
                for (k, slot) in [("text", &mut def.text), ("texthl", &mut def.texthl), ("linehl", &mut def.linehl), ("numhl", &mut def.numhl)] {
                    if let Some(v) = kv(&words[2..], k) {
                        *slot = v;
                    }
                }
            }
            "undefine" => {
                let Some(name) = words.get(1) else { return self.say("E156: Missing sign name") };
                if self.sign_defs.remove(*name).is_none() {
                    self.say(format!("E155: Unknown sign: {name}"));
                }
            }
            "list" | "" => {
                let want = words.get(1).copied();
                let mut rows: Vec<(String, String)> = self.sign_defs.iter().filter(|(n, _)| want.is_none_or(|w| w == n.as_str())).map(|(n, d)| (format!("sign {n}"), format!("text={} texthl={} linehl={} numhl={}", d.text, d.texthl, d.linehl, d.numhl))).collect();
                rows.sort();
                if rows.is_empty() {
                    return self.say(if want.is_some() { "E155: Unknown sign" } else { "no signs defined" });
                }
                self.popup = Some(Popup::Help("signs".into(), rows));
            }
            "place" => {
                // :sign place (alone, or with file= / buffer=): the placed signs
                let Some(id) = words.get(1).and_then(|w| w.parse::<u32>().ok()) else {
                    let docs: Vec<usize> = if words.len() > 1 { self.sign_doc(&words[1..]).into_iter().collect() } else { (0..self.docs.len()).collect() };
                    let rows: Vec<(String, String)> = docs.iter().flat_map(|&i| self.docs[i].signs.iter().map(move |p| (i, p.clone()))).map(|(i, p)| (format!("{}:{}", self.docs[i].name, p.line + 1), format!("id={} name={} priority={}", p.id, p.name, p.priority))).collect();
                    if rows.is_empty() {
                        return self.say("no signs placed");
                    }
                    self.popup = Some(Popup::Help("placed signs".into(), rows));
                    return;
                };
                let rest = &words[2..];
                let Some(di) = self.sign_doc(rest) else { return self.say("E158: Invalid buffer name") };
                let name = kv(rest, "name");
                let line = kv(rest, "line").and_then(|l| l.parse::<usize>().ok());
                let priority = kv(rest, "priority").and_then(|p| p.parse().ok()).unwrap_or(10);
                let signs = &mut self.docs[di].signs;
                match (name, line) {
                    (Some(name), Some(line)) => {
                        if !self.sign_defs.contains_key(&name) {
                            return self.say(format!("E155: Unknown sign: {name}"));
                        }
                        signs.retain(|p| p.id != id);
                        signs.push(Placed { id, line: line.saturating_sub(1), name, priority });
                    }
                    // the sign `id` another name (its line stays)
                    (Some(name), None) => match signs.iter_mut().find(|p| p.id == id) {
                        Some(p) => p.name = name,
                        None => self.say(format!("E157: Invalid sign ID: {id}")),
                    },
                    _ => self.say("E159: Missing sign number or name"),
                }
            }
            "unplace" => {
                let rest = &words[1..];
                let all = rest.first() == Some(&"*");
                let id = rest.first().and_then(|w| w.parse::<u32>().ok());
                let docs: Vec<usize> = if all || kv(rest, "file").is_some() || kv(rest, "buffer").is_some() { self.sign_doc(rest).filter(|_| !all || rest.len() > 1).map(|d| vec![d]).unwrap_or_else(|| (0..self.docs.len()).collect()) } else { vec![self.cur] };
                for di in docs {
                    let signs = &mut self.docs[di].signs;
                    match id {
                        Some(id) => signs.retain(|p| p.id != id),
                        None if all => signs.clear(),
                        // :sign unplace alone: the one on the cursor's line
                        None => {
                            let l = self.docs[di].ed.cursor().line;
                            self.docs[di].signs.retain(|p| p.line != l);
                        }
                    }
                }
            }
            "jump" => {
                let Some(id) = words.get(1).and_then(|w| w.parse::<u32>().ok()) else { return self.say("E159: Missing sign number") };
                let Some(di) = self.sign_doc(&words[2..]) else { return self.say("E158: Invalid buffer name") };
                let Some(line) = self.docs[di].signs.iter().find(|p| p.id == id).map(|p| p.line) else { return self.say(format!("E157: Invalid sign ID: {id}")) };
                self.show_doc(di);
                self.goto_line(line + 1);
            }
            other => self.say(format!("E160: Unknown sign command: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn define_place_draw_jump_unplace() {
        let mut app = crate::App::new(Vec::new());
        app.docs[app.cur].ed = zett_edit::Editor::from_str("a\nb\nc\nd\n");
        app.command("sign define bp text=>> texthl=Error linehl=Visual");
        app.command("sign place 7 line=3 name=bp");
        app.command("sign place 8 line=1 name=bp priority=5");
        let rows = app.sign_rows(app.cur);
        assert_eq!(rows.get(&2).map(|s| s.text.as_str()), Some(">>"), "line 3");
        assert!(rows.get(&2).is_some_and(|s| s.linehl.is_some()));
        app.command("sign jump 7");
        assert_eq!(app.docs[app.cur].ed.cursor().line, 2);
        app.command("sign unplace 7");
        assert!(app.sign_rows(app.cur).get(&2).is_none());
        app.command("sign unplace *");
        assert!(app.docs[app.cur].signs.is_empty());
        app.command("sign place 1 line=1 name=nope");
        assert!(app.docs[app.cur].signs.is_empty(), "an unknown sign is refused");
    }
}
