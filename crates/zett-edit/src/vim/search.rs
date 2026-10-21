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
                self.vim.hl_off = false;
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
        self.vim.hl_off = false;
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
        let (start, plain) = (self.cursor, self.op.is_none());
        self.vim.marks.insert('\'', self.cursor);
        self.motion(Span::Chars(self.cursor.min(at), self.cursor.max(at)), Some(at));
        if plain {
            self.search_note(&q, fold, at, if forward { at <= start } else { at >= start }, forward);
        }
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
        self.vim.hl_off = false;
        self.vim.search_word = whole;
        self.vim.search_back = !forward;
        // from the word's start, so `#` skips this one
        self.cursor = a;
        self.vim_next(forward);
    }

}

impl Editor {
    /// After n N / ?: "search hit BOTTOM, continuing at TOP" when it went round (unless shortmess
    /// has s), and the match's place among them, [2/5] (unless shortmess has S; past
    /// maxsearchcount it says >N).
    fn search_note(&mut self, q: &str, fold: bool, at: usize, wrapped: bool, forward: bool) {
        let sm = self.opt("shortmess").unwrap_or("ltToOCF").to_string();
        let mut m = if wrapped && !sm.contains('s') {
            if forward { "search hit BOTTOM, continuing at TOP".to_string() } else { "search hit TOP, continuing at BOTTOM".to_string() }
        } else {
            format!("{}{q}", if forward { '/' } else { '?' })
        };
        if !sm.contains('S') {
            let max = self.opt_num("maxsearchcount", 999).max(1) as usize;
            let (len, n) = (self.buf.len_chars(), q.chars().count());
            let (mut total, mut index, mut from) = (0usize, 0usize, 0usize);
            while let Some(i) = self.buf.find(q, from.min(len), true, fold) {
                if !self.vim.search_word || self.whole_word_at(i, n) {
                    total += 1;
                    if i <= at {
                        index = total;
                    }
                    if total > max {
                        break;
                    }
                }
                from = i + 1;
                if from > len {
                    break;
                }
            }
            let shown = if total > max { format!("[{}/>{max}]", if index > max { format!(">{max}") } else { index.to_string() }) } else { format!("[{index}/{total}]") };
            m = format!("{m}  {shown}");
        }
        self.vim.requests.push(Request::Message(m));
    }
}
