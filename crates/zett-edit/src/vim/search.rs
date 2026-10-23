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
                let hit = self.vim_find(&q, from.min(len), forward, fold).ok().flatten().map(|m| m.0);
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
                    match self.vim_find(&q, from.min(len), forward, fold).ok().flatten().map(|m| m.0) {
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
        // terse (Vim's old way to say shortmess s): no "search hit BOTTOM"
        let sm = if self.opt_on("terse", false) { format!("{sm}s") } else { sm };
        let mut m = if wrapped && !sm.contains('s') {
            if forward { "search hit BOTTOM, continuing at TOP".to_string() } else { "search hit TOP, continuing at BOTTOM".to_string() }
        } else {
            format!("{}{q}", if forward { '/' } else { '?' })
        };
        if !sm.contains('S') {
            let max = self.opt_num("maxsearchcount", 999).max(1) as usize;
            let len = self.buf.len_chars();
            // every match (vim_find_all stops at 1000; maxsearchcount says >N past it)
            let all = self.vim_find_all(q, fold, 0, len);
            let total = all.len().min(max + 1);
            let index = all.iter().take_while(|m| m.0 <= at).count().min(max + 1);
            let shown = if total > max { format!("[{}/>{max}]", if index > max { format!(">{max}") } else { index.to_string() }) } else { format!("[{index}/{total}]") };
            m = format!("{m}  {shown}");
        }
        self.vim.requests.push(Request::Message(m));
    }
}

impl Editor {
    /// A match of Vim pattern `q` from char `from` (forward: the first starting at or after it;
    /// backward: the last starting before it), as (start, end) chars; Err while it does not parse
    /// (a search being typed: `\(` not closed yet). After * and # the word is found as it is.
    pub(crate) fn vim_find(&self, q: &str, from: usize, forward: bool, fold: bool) -> Result<Option<(usize, usize)>, ()> {
        if self.vim.search_word {
            let n = q.chars().count();
            return Ok(self.buf.find(q, from, forward, fold).map(|i| (i, i + n)));
        }
        let re = crate::vimre::compile(q, fold, false).map_err(|_| ())?;
        let text = self.buf.text();
        let byte_of = |c: usize| text.char_indices().nth(c).map_or(text.len(), |(b, _)| b);
        let char_of = |b: usize| text[..b].chars().count();
        let at = byte_of(from);
        let hit = if forward {
            re.find_at(&text, at)
        } else {
            re.find_all(&text).into_iter().take_while(|(s, _)| *s < at).last()
        };
        Ok(hit.map(|(s, e)| (char_of(s), char_of(e))))
    }

    /// Every match of `q` starting in [from, to) (chars), compiled once and looked for only in the
    /// lines around that range: hlsearch, incsearch's other matches, the [n/N] count.
    pub(crate) fn vim_find_all(&self, q: &str, fold: bool, from: usize, to: usize) -> Vec<(usize, usize)> {
        if self.vim.search_word {
            let n = q.chars().count();
            let mut out = Vec::new();
            let mut at = from;
            while let Some(i) = self.buf.find(q, at, true, fold) {
                if i >= to || out.len() >= 1000 {
                    break;
                }
                if self.whole_word_at(i, n) {
                    out.push((i, i + n));
                }
                at = i + 1;
            }
            return out;
        }
        let Ok(re) = crate::vimre::compile(q, fold, false) else { return Vec::new() };
        let len = self.buf.len_chars();
        let (from, to) = (from.min(len), to.min(len));
        // whole lines, so ^ $ \< and lookbehind see what they need
        let start = self.buf.to_char(zett_core::Pos { line: self.buf.to_pos(from).line, col: 0 });
        let last = self.buf.to_pos(to).line;
        let end = if last + 1 < self.buf.len_lines() { self.buf.to_char(zett_core::Pos { line: last + 1, col: 0 }) } else { len };
        let text = self.buf.slice(start, end);
        let mut out = Vec::new();
        let mut chars = 0usize;
        let mut prev = 0usize;
        for (s, e) in re.find_all(&text) {
            chars += text[prev..s].chars().count();
            prev = s;
            let (cs, ce) = (start + chars, start + chars + text[s..e].chars().count());
            if cs >= to || out.len() >= 1000 {
                break;
            }
            if cs >= from {
                out.push((cs, ce));
            }
        }
        out
    }
}
