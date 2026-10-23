//! vim.snippet: a snippet put in, Tab / S-Tab through its stops (they move with every edit).

use super::*;

/// A snippet being filled in. Its stops are the editor's moving ranges, in jump order ($0 last).
pub(crate) struct Session {
    doc: usize,
    at: usize,
    /// the stop's placeholder is untouched: the first key typed replaces it
    fresh: bool,
}

/// `text` with the line's indent after each newline and tabs as 'expandtab' says, and where each
/// character of it went (one more for the end).
fn reindent(text: &str, indent: &str, tab: &str) -> (String, Vec<usize>) {
    let mut out = String::new();
    let mut map = Vec::new();
    let mut n = 0;
    for c in text.chars() {
        map.push(n);
        let put = match c {
            '\n' => format!("\n{indent}"),
            '\t' => tab.to_string(),
            c => c.to_string(),
        };
        n += put.chars().count();
        out.push_str(&put);
    }
    map.push(n);
    (out, map)
}

impl App {
    /// Puts `body` (LSP snippet syntax) in place of a..b (characters) and goes to its first stop.
    pub(crate) fn snippet_insert(&mut self, a: usize, b: usize, body: &str) {
        self.snippet_end();
        let i = self.cur;
        let path = self.docs[i].abs_path().unwrap_or_default();
        let line = self.docs[i].ed.cursor().line;
        let cur_line = self.docs[i].ed.buf.line(line).trim_end_matches('\n').to_string();
        let var = |v: &str| -> Option<String> {
            Some(match v {
                "TM_FILENAME" => path.file_name()?.to_string_lossy().to_string(),
                "TM_FILENAME_BASE" => path.file_stem()?.to_string_lossy().to_string(),
                "TM_FILEPATH" => path.display().to_string(),
                "TM_DIRECTORY" => path.parent()?.display().to_string(),
                "TM_LINE_INDEX" => line.to_string(),
                "TM_LINE_NUMBER" => (line + 1).to_string(),
                "TM_CURRENT_LINE" => cur_line.clone(),
                _ => return None,
            })
        };
        let sn = zett_edit::snippet::parse(body, &var);
        let indent: String = cur_line.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
        let tab = if self.flag(i, "expandtab") {
            let sw = self.num(i, "shiftwidth");
            " ".repeat(if sw == 0 { self.num(i, "tabstop").max(1) } else { sw })
        } else {
            "\t".to_string()
        };
        let (text, map) = reindent(&sn.text, &indent, &tab);
        let stops: Vec<(usize, usize)> = sn.order().into_iter().map(|(s, e)| (a + map[s], a + map[e])).collect();
        let ed = &mut self.docs[i].ed;
        ed.replace_text(a, b.max(a), &text);
        self.docs[i].touched(line);
        self.edit_at = Instant::now();
        if stops.len() == 1 {
            // only the end: no stops to fill in
            self.docs[i].ed.set_offset(stops[0].0);
            return;
        }
        self.docs[i].ed.set_ranges(stops);
        self.snip = Some(Session { doc: i, at: 0, fresh: false });
        self.snippet_go(0);
    }

    /// To stop `n`: the cursor at its start, its placeholder ready to be typed over. The last
    /// stop ($0) ends the snippet.
    fn snippet_go(&mut self, n: usize) {
        let Some(s) = self.snip.as_mut() else { return };
        let d = &mut self.docs[s.doc];
        let Some(&(a, b)) = d.ed.ranges().get(n) else { return };
        s.at = n;
        s.fresh = b > a;
        d.ed.set_offset(a);
        if n + 1 == d.ed.ranges().len() {
            self.snippet_end();
        } else {
            self.command("lua vim.g._zett_snippet = true");
        }
    }

    /// Tab (1) / S-Tab (-1) in a snippet: false when there is none, or no stop that way.
    pub(crate) fn snippet_jump(&mut self, dir: i32) -> bool {
        let Some(s) = &self.snip else { return false };
        if s.doc != self.cur {
            self.snippet_end();
            return false;
        }
        let n = self.docs[s.doc].ed.ranges().len();
        let to = s.at as i64 + dir as i64;
        if to < 0 || to as usize >= n {
            return false;
        }
        self.snippet_go(to as usize);
        true
    }

    pub(crate) fn snippet_end(&mut self) {
        if let Some(s) = self.snip.take() {
            if let Some(d) = self.docs.get_mut(s.doc) {
                d.ed.set_ranges(Vec::new());
            }
            self.command("lua vim.g._zett_snippet = false");
        }
    }

    /// Before an insert-mode key: on an untouched placeholder, a typed character or Backspace
    /// takes the placeholder away first (Backspace then has done its work: true).
    pub(crate) fn snippet_before_key(&mut self, k: &KeyEvent) -> bool {
        let Some(s) = self.snip.as_mut() else { return false };
        if !std::mem::take(&mut s.fresh) || s.doc != self.cur {
            return false;
        }
        let d = &mut self.docs[s.doc];
        let Some(&(a, b)) = d.ed.ranges().get(s.at) else { return false };
        if d.ed.offset() != a {
            return false;
        }
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        match k.code {
            KeyCode::Char(_) if !ctrl => {
                d.ed.replace_text(a, b, "");
                false
            }
            KeyCode::Backspace | KeyCode::Delete => {
                d.ed.replace_text(a, b, "");
                true
            }
            _ => false,
        }
    }

    /// After a key: the cursor left the snippet (or its buffer), or insert mode was left: it ends.
    pub(crate) fn snippet_check(&mut self) {
        let Some(s) = &self.snip else { return };
        let d = &self.docs[s.doc.min(self.docs.len() - 1)];
        let r = d.ed.ranges();
        let inside = r.iter().map(|x| x.0).min().zip(r.iter().map(|x| x.1).max()).is_some_and(|(lo, hi)| (lo..=hi).contains(&d.ed.offset()));
        if s.doc != self.cur || !inside || d.ed.mode() != Mode::Insert {
            self.snippet_end();
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn app(text: &str) -> crate::App {
        let mut app = crate::App::new(Vec::new());
        app.docs[app.cur].ed = zett_edit::Editor::from_str(text);
        app
    }
    fn key(app: &mut crate::App, k: KeyCode) {
        app.key(KeyEvent::new(k, KeyModifiers::NONE));
    }
    fn typed(app: &mut crate::App, s: &str) {
        for c in s.chars() {
            key(app, KeyCode::Char(c));
        }
    }

    #[test]
    fn placeholders_are_typed_over_and_tab_walks_the_stops() {
        let mut a = app("\n");
        key(&mut a, KeyCode::Char('i'));
        a.snippet_insert(0, 0, "fn ${1:name}(${2:arg}) {\n\t$0\n}");
        typed(&mut a, "go");
        key(&mut a, KeyCode::Tab);
        // Backspace on an untouched placeholder takes it all
        key(&mut a, KeyCode::Backspace);
        typed(&mut a, "x: u8");
        key(&mut a, KeyCode::Tab);
        typed(&mut a, "body");
        assert_eq!(a.docs[a.cur].ed.text(), "fn go(x: u8) {\n    body\n}\n");
        assert!(a.snip.is_none(), "$0 ends it");
    }

    #[test]
    fn stops_follow_edits_and_leaving_ends_it() {
        let mut a = app("\n");
        key(&mut a, KeyCode::Char('i'));
        a.snippet_insert(0, 0, "f(${1:a}, ${2:b})");
        // typed into the first, then a word deleted with C-w: the second stop still right
        typed(&mut a, "long_name");
        a.key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL));
        typed(&mut a, "z");
        key(&mut a, KeyCode::Tab);
        typed(&mut a, "y");
        assert_eq!(a.docs[a.cur].ed.text(), "f(z, y)\n");
        // Esc leaves insert mode: the snippet is over, Tab is Tab again
        a.snippet_insert(0, 0, "${1:p} ${2:q}");
        key(&mut a, KeyCode::Esc);
        assert!(a.snip.is_none());
    }
}
