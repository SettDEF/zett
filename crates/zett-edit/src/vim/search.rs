//! Vim's search: / ? n N * # g* g# (wrapscan, smart case from the settings).

use super::*;

impl Editor {
    pub(crate) fn vim_search_start(&mut self, forward: bool) {
        self.isearch_start_vim(forward);
    }

    pub(crate) fn vim_search_key(&mut self, k: Key) {
        match k.code {
            Code::Enter => {
                let Some(s) = self.isearch.take() else { return };
                let q = if s.query().is_empty() { self.last_search.clone() } else { s.query().to_string() };
                let (origin, forward) = (s.origin(), s.forward());
                self.cursor = origin;
                if q.is_empty() {
                    return;
                }
                self.last_search = q;
                self.vim.search_back = !forward;
                self.vim.search_word = false;
                self.vim_next(forward);
            }
            Code::Esc => {
                if let Some(s) = self.isearch.take() {
                    self.cursor = s.origin();
                }
                self.op = None;
                self.count = None;
            }
            Code::Backspace => {
                if self.isearch.as_ref().is_some_and(|s| s.query().is_empty()) {
                    let o = self.isearch.take().map(|s| s.origin()).unwrap_or(self.cursor);
                    self.cursor = o;
                    self.op = None;
                } else {
                    self.exec("isearch-delete-char", 1);
                }
            }
            _ => {
                if let Some(c) = k.char().filter(|_| !k.ctrl && !k.alt) {
                    self.isearch_char(c);
                }
            }
        }
    }

    /// The next match of the last search (`n`; `forward` false: `N`), as a motion.
    pub(crate) fn vim_next(&mut self, forward: bool) {
        let q = self.last_search.clone();
        if q.is_empty() {
            self.count = None;
            self.op = None;
            return self.vim.requests.push(Request::Message("E35: No previous regular expression".into()));
        }
        let n = self.take_count();
        let fold = self.folds_for(&q);
        let len = self.buf.len_chars();
        let mut at = self.cursor;
        for _ in 0..n {
            let mut from = if forward { at + 1 } else { at };
            let found = loop {
                let hit = self.buf.find(&q, from.min(len), forward, fold);
                match hit {
                    Some(i) if self.vim.search_word && !self.whole_word_at(i, q.chars().count()) => {
                        from = if forward { i + 1 } else { i };
                    }
                    other => break other,
                }
            };
            let found = found.or_else(|| {
                if !self.vim.wrapscan {
                    return None;
                }
                let mut from = if forward { 0 } else { len };
                loop {
                    match self.buf.find(&q, from.min(len), forward, fold) {
                        Some(i) if self.vim.search_word && !self.whole_word_at(i, q.chars().count()) => from = if forward { i + 1 } else { i },
                        other => break other,
                    }
                }
            });
            match found {
                Some(i) => at = i,
                None => {
                    self.op = None;
                    return self.vim.requests.push(Request::Message(format!("E486: Pattern not found: {q}")));
                }
            }
        }
        self.vim.marks.insert('\'', self.cursor);
        self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at)), Some(at));
    }

    pub(crate) fn whole_word_at(&self, i: usize, n: usize) -> bool {
        let before = i.checked_sub(1).and_then(|p| self.char(p)).is_none_or(|c| !is_word(c));
        let after = self.char(i + n).is_none_or(|c| !is_word(c));
        before && after
    }

    /// The word under (or after) the cursor: (start, end).
    pub(crate) fn word_here(&self) -> Option<(usize, usize)> {
        let line = self.cur_line();
        let (s, e) = (self.line_start(line), self.line_end(line));
        let mut i = self.cursor;
        while i < e && !self.char(i).is_some_and(is_word) {
            i += 1;
        }
        if i >= e {
            return None;
        }
        let mut a = i;
        while a > s && self.char(a - 1).is_some_and(is_word) {
            a -= 1;
        }
        let mut b = i;
        while b < e && self.char(b).is_some_and(is_word) {
            b += 1;
        }
        Some((a, b))
    }

    /// `*` `#` (whole word), `g*` `g#` (any match).
    pub(crate) fn vim_star(&mut self, forward: bool, whole: bool) {
        let Some((a, b)) = self.word_here() else {
            self.count = None;
            return self.vim.requests.push(Request::Message("E348: No string under cursor".into()));
        };
        self.last_search = self.buf.slice(a, b);
        self.vim.search_word = whole;
        self.vim.search_back = !forward;
        // from the word's start, so `#` skips this one
        self.cursor = a;
        self.vim_next(forward);
    }

}
