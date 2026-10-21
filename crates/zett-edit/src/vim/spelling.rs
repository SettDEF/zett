//! Spelling keys (spell.rs does the checking): ]s [s ]S [S to the next or previous misspelled
//! word, z= suggestions (with a count: that one at once), zg zw zG zW and their undo zug zuw zuG zuW.

use super::*;

/// After z u: the third key says which list to take the word off.
pub(crate) const ZU: char = '\u{1f}';

impl Editor {
    /// The word under the cursor for spelling: letters and apostrophes, (start, end).
    fn spell_word(&self) -> Option<(usize, usize)> {
        let line = self.cur_line();
        let (s, e) = (self.line_start(line), self.line_end(line));
        let is = |c: char| c.is_alphabetic() || c == '\'';
        let mut i = self.cursor.min(e);
        while i < e && !self.char(i).is_some_and(is) {
            i += 1;
        }
        if i >= e {
            return None;
        }
        let (mut a, mut b) = (i, i);
        while a > s && self.char(a - 1).is_some_and(is) {
            a -= 1;
        }
        while b < e && self.char(b).is_some_and(is) {
            b += 1;
        }
        Some((a, b))
    }

    /// ]s [s: the next or previous word the spell checker does not know.
    pub(crate) fn spell_jump(&mut self, fwd: bool) {
        if !crate::spell::with(|s| s.available()) {
            return self.vim.requests.push(Request::Message("E756: Spell checking is not possible: no dictionary (install hunspell-en_us, or put a word list in ~/.local/share/zett/spell/)".into()));
        }
        let n = self.take_count();
        let text: Vec<char> = self.buf.text().chars().collect();
        let mut words: Vec<(usize, usize)> = Vec::new();
        let mut i = 0;
        while i < text.len() {
            if text[i].is_alphabetic() {
                let a = i;
                while i < text.len() && (text[i].is_alphabetic() || text[i] == '\'') {
                    i += 1;
                }
                words.push((a, i));
            } else {
                i += 1;
            }
        }
        let bad: Vec<usize> = crate::spell::with(|s| words.iter().filter(|(a, b)| !s.ok(&text[*a..*b].iter().collect::<String>())).map(|w| w.0).collect());
        let mut at = self.cursor;
        for _ in 0..n {
            let next = if fwd { bad.iter().copied().find(|&p| p > at).or(bad.first().copied()) } else { bad.iter().rev().copied().find(|&p| p < at).or(bad.last().copied()) };
            match next {
                Some(p) => at = p,
                None => return self.vim.requests.push(Request::Message("No misspelled words".into())),
            }
        }
        self.jump_motion(at, fwd);
    }

    /// z= : suggestions for the word under the cursor; with a count, that one at once.
    pub(crate) fn spell_suggest(&mut self) {
        let Some((a, b)) = self.spell_word() else { return };
        let word = self.buf.slice(a, b);
        let n = self.count.take();
        // spellsuggest: how many ("best,9"; Vim lists 25 when no number is given)
        let many = self.opt("spellsuggest").unwrap_or("best").split(',').find_map(|p| p.parse::<usize>().ok()).unwrap_or(25);
        let list = crate::spell::with(|s| if s.available() { s.suggest(&word, many) } else { Vec::new() });
        if list.is_empty() {
            return self.vim.requests.push(Request::Message(format!("Sorry, no suggestions for \"{word}\"")));
        }
        match n {
            Some(k) => {
                if let Some(w) = list.get(k.saturating_sub(1)) {
                    self.vim.last_spell = Some((word.clone(), w.clone()));
                    self.replace_range(a, b, w);
                    self.cursor = a;
                }
            }
            None => self.vim.requests.push(Request::Suggest(a, b, list)),
        }
    }

    /// The last z= replacement: the word and what it became (:spellrepall repeats it).
    pub fn last_spell(&self) -> Option<(String, String)> {
        self.vim.last_spell.clone()
    }

    /// zg zw zG zW (and with undo: zug zuw zuG zuW).
    pub(crate) fn spell_mark(&mut self, c: char, undo: bool) {
        let Some((a, b)) = self.spell_word() else { return };
        let w = self.buf.slice(a, b);
        let (good, session) = (c.eq_ignore_ascii_case(&'g'), c.is_uppercase());
        crate::spell::with(|s| s.mark(&w, good, session, undo));
        let what = match (good, undo) { (true, false) => "added to the good words", (false, false) => "marked wrong", _ => "taken off the list" };
        self.vim.requests.push(Request::Message(format!("Word \"{w}\" {what}")));
    }

    /// Replaces [a, b) with `t` as one change (the window takes a z= suggestion this way).
    pub fn replace(&mut self, a: usize, b: usize, t: &str) {
        self.vim.last_spell = Some((self.buf.slice(a, b), t.to_string()));
        self.replace_range(a, b, t);
        self.cursor = a;
        self.clamp();
    }
}
