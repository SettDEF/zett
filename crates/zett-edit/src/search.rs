//! Emacs's incremental search (C-s / C-r): every typed character moves to the next match at
//! once, C-s / C-r go on to the next one either way and wrap around after a failing search,
//! Backspace steps back exactly, Enter stays (leaving the mark where it began), C-g returns.
//! Smart case: a query without capitals matches either case. The matcher is Buffer::find.

use crate::Editor;

/// The search commands (M-x, C-h f); keys point at them through the "isearch" keymap.
pub const COMMANDS: &[(&str, &str)] = &[
    ("isearch-forward", "Search forward as you type"),
    ("isearch-backward", "Search backward as you type"),
    ("isearch-repeat-forward", "Search on to the next match (wraps after a failing search)"),
    ("isearch-repeat-backward", "Search back to the previous match (wraps after a failing search)"),
    ("isearch-delete-char", "Undo the last step of the search"),
    ("isearch-exit", "End the search where it is (the mark stays where it began)"),
    ("isearch-abort", "Cancel the search and go back to where it began"),
    ("isearch-forward-regexp", "Search forward for a regexp as you type (Emacs's syntax: \\( \\| \\) groups)"),
    ("isearch-backward-regexp", "Search backward for a regexp as you type"),
    ("isearch-forward-word", "Search forward for words, whatever is between them"),
    ("isearch-forward-symbol", "Search forward for a whole symbol"),
    ("isearch-forward-thing-at-point", "Search for the symbol at the cursor, as a whole symbol"),
    ("isearch-toggle-regexp", "During a search: as a regexp, or as text again"),
    ("isearch-toggle-word", "During a search: by words, or as text again"),
    ("isearch-toggle-symbol", "During a search: a whole symbol, or as text again"),
];

/// What a search looks for: the text, a regexp, words, a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SearchKind {
    #[default]
    Text,
    Regexp,
    Word,
    Symbol,
    /// Vim's / and ?: a Vim pattern (vimre.rs)
    Vim,
}

/// Emacs's regexp syntax in the regex crate's: \( \) \| \{ \} group and count, bare ones are
/// themselves; \< \> word edges, \` \' the buffer's ends, \s- a blank, \_< \_> symbol edges.
pub fn emacs_regex(p: &str) -> String {
    let mut out = String::new();
    let mut it = p.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\\' => match it.next() {
                Some(d @ ('(' | ')' | '|' | '{' | '}')) => {
                    // \(?: a shy group
                    if d == '(' && it.peek() == Some(&'?') {
                        it.next();
                        if it.peek() == Some(&':') {
                            it.next();
                        }
                        out.push_str("(?:");
                    } else {
                        out.push(d);
                    }
                }
                Some('<' | '>') => out.push_str(r"\b"),
                Some('_') => {
                    it.next();
                    out.push_str(r"\b");
                }
                Some('`') => out.push_str(r"\A"),
                Some('\'') => out.push_str(r"\z"),
                Some('s') => {
                    it.next();
                    out.push_str(r"\s");
                }
                Some('S') => {
                    it.next();
                    out.push_str(r"\S");
                }
                Some(d @ ('w' | 'W' | 'b' | 'B' | '1'..='9')) => {
                    out.push('\\');
                    out.push(d);
                }
                Some(d) => out.push_str(&regex::escape(&d.to_string())),
                None => out.push_str(r"\\"),
            },
            '(' | ')' | '|' | '{' | '}' => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out
}

/// The pattern a search of this kind runs (None for plain text): Err for a regexp not finished yet.
fn pattern(query: &str, kind: SearchKind, fold: bool) -> Option<Result<regex::Regex, ()>> {
    let p = match kind {
        SearchKind::Text => return None,
        SearchKind::Regexp => emacs_regex(query),
        // words: whatever is between them (blanks, punctuation, line breaks)
        SearchKind::Word => query.split(|c: char| !c.is_alphanumeric() && c != '_').filter(|w| !w.is_empty()).map(regex::escape).collect::<Vec<_>>().join(r"\W+"),
        SearchKind::Symbol => regex::escape(query),
        SearchKind::Vim => return None,
    };
    // a word or a symbol: whole, never part of a longer one
    let p = if matches!(kind, SearchKind::Word | SearchKind::Symbol) { format!(r"\b{p}\b") } else { p };
    Some(regex::RegexBuilder::new(&p).case_insensitive(fold).multi_line(true).build().map_err(|_| ()))
}

#[derive(Debug, Clone, PartialEq)]
struct Step {
    query: String,
    forward: bool,
    /// the match (start, end), None while failing
    at: Option<(usize, usize)>,
    cursor: usize,
    wrapped: bool,
}

#[derive(Debug, Clone)]
pub struct Isearch {
    origin: usize,
    /// what it looks for (C-M-s: a regexp, M-s w: words, M-s _: a symbol)
    pub kind: SearchKind,
    /// started with `/` or `?` (the prompt reads as Vim's)
    vim: bool,
    /// the case rule the last step ran with
    fold: bool,
    /// the regexp typed so far does not parse yet
    incomplete: bool,
    now: Step,
    /// every step before this one, for isearch-delete-char
    back: Vec<Step>,
}

impl Isearch {
    /// "I-search: foo", "Failing I-search backward: foo", "Wrapped I-search: foo"
    pub fn prompt(&self) -> String {
        let s = &self.now;
        if self.vim {
            return format!("{}{}", if s.forward { "/" } else { "?" }, s.query);
        }
        let state = match (s.at.is_none() && !s.query.is_empty(), s.wrapped) {
            (true, _) => "Failing ",
            (false, true) => "Wrapped ",
            _ => "",
        };
        let kind = match self.kind {
            SearchKind::Text => "",
            SearchKind::Regexp => "Regexp ",
            SearchKind::Word => "Word ",
            SearchKind::Symbol => "Symbol ",
            SearchKind::Vim => "",
        };
        // a regexp still being typed (an open \\( ): not a failure yet
        let state = if self.incomplete { "Pending " } else { state };
        format!("{state}{kind}I-search{}: {}", if s.forward { "" } else { " backward" }, s.query)
    }

    pub(crate) fn origin(&self) -> usize {
        self.origin
    }
    pub(crate) fn forward(&self) -> bool {
        self.now.forward
    }

    pub fn query(&self) -> &str {
        &self.now.query
    }

    /// The current match, the one the cursor is on.
    pub fn current(&self) -> Option<(usize, usize)> {
        self.now.at
    }

    /// Either case matches: as the search ran (ignorecase, and smartcase looking for capitals).
    pub fn folds(&self) -> bool {
        self.fold
    }
}

impl Editor {
    /// The next match of a search from `from` (char index) either way: (start, end); Err when a
    /// regexp does not parse yet.
    fn search_find(&self, query: &str, kind: SearchKind, from: usize, forward: bool, fold: bool) -> Result<Option<(usize, usize)>, ()> {
        if kind == SearchKind::Vim {
            return self.vim_find(query, from, forward, fold);
        }
        let Some(re) = pattern(query, kind, fold) else {
            let n = query.chars().count();
            return Ok(self.buf.find(query, from, forward, fold).map(|i| (i, i + n)));
        };
        let re = re?;
        let text = self.buf.text();
        let byte_of = |c: usize| text.char_indices().nth(c).map_or(text.len(), |(b, _)| b);
        let char_of = |b: usize| text[..b].chars().count();
        let at = byte_of(from);
        let hit = if forward {
            re.find_at(&text, at).filter(|m| m.end() > m.start())
        } else {
            // backward: the last match starting before `from`
            re.find_iter(&text).filter(|m| m.start() < at && m.end() > m.start()).last()
        };
        Ok(hit.map(|m| (char_of(m.start()), char_of(m.end()))))
    }

    pub fn isearch(&self) -> Option<&Isearch> {
        self.isearch.as_ref()
    }

    /// The matches of the search in progress that start in [from, to) (char indices): for
    /// drawing the other matches dimmer than the current one.
    pub fn isearch_matches(&self, from: usize, to: usize) -> Vec<(usize, usize)> {
        let Some(s) = &self.isearch else { return Vec::new() };
        if s.now.query.is_empty() {
            return Vec::new();
        }
        if s.kind == SearchKind::Vim {
            return self.vim_find_all(&s.now.query, s.folds(), from, to);
        }
        let mut out = Vec::new();
        let mut at = from;
        while let Ok(Some((a, b))) = self.search_find(&s.now.query, s.kind, at, true, s.folds()) {
            if a >= to || out.len() >= 1000 {
                break;
            }
            out.push((a, b));
            at = a + 1;
        }
        out
    }

    /// Runs a search command; false when `name` is not one.
    pub(crate) fn search_command(&mut self, name: &str) -> bool {
        match name {
            "isearch-forward" | "isearch-backward" => self.isearch_start(name == "isearch-forward"),
            "isearch-forward-regexp" | "isearch-backward-regexp" => {
                self.isearch_start(name == "isearch-forward-regexp");
                self.isearch_kind(SearchKind::Regexp);
            }
            "isearch-forward-word" | "isearch-forward-symbol" => {
                self.isearch_start(true);
                self.isearch_kind(if name.ends_with("word") { SearchKind::Word } else { SearchKind::Symbol });
            }
            "isearch-forward-thing-at-point" => {
                let Some(w) = self.word_at_cursor() else { return true };
                self.isearch_start(true);
                self.isearch_kind(SearchKind::Symbol);
                for c in w.chars() {
                    self.isearch_char(c);
                }
            }
            "isearch-toggle-regexp" | "isearch-toggle-word" | "isearch-toggle-symbol" => {
                let want = match name { "isearch-toggle-regexp" => SearchKind::Regexp, "isearch-toggle-word" => SearchKind::Word, _ => SearchKind::Symbol };
                let Some(s) = &self.isearch else { return true };
                let k = if s.kind == want { SearchKind::Text } else { want };
                self.isearch_kind(k);
            }
            "isearch-repeat-forward" | "isearch-repeat-backward" => {
                let forward = name == "isearch-repeat-forward";
                if self.isearch.is_none() {
                    self.isearch_start(forward);
                }
                self.isearch_repeat(forward);
            }
            "isearch-delete-char" => {
                if let Some(s) = &mut self.isearch {
                    if let Some(prev) = s.back.pop() {
                        self.cursor = prev.cursor;
                        s.now = prev;
                    }
                }
            }
            "isearch-exit" => {
                if let Some(s) = self.isearch.take() {
                    if !s.now.query.is_empty() {
                        self.last_search = s.now.query.clone();
                        // Emacs keeps no highlight once the search ends (hlsearch is a Vim thing)
                        self.vim.hl_off = true;
                    }
                    // the mark where the search began, not active (Emacs: "Mark saved where search started")
                    if self.cursor != s.origin {
                        self.mark = Some(s.origin);
                        self.mark_active = false;
                    }
                }
            }
            "isearch-abort" => {
                if let Some(s) = self.isearch.take() {
                    self.cursor = s.origin;
                }
            }
            _ => return false,
        }
        true
    }

    /// The case rule (the ignorecase and smartcase settings; both on is Emacs's way).
    pub fn set_case(&mut self, ignore: bool, smart: bool) {
        self.case = (ignore, smart);
    }

    /// Does a search for `query` match either case?
    pub fn folds_for(&self, query: &str) -> bool {
        let (ignore, smart) = self.case;
        ignore && !(smart && query.chars().any(char::is_uppercase))
    }

    /// `/` and `?` in the Vim profile: the same search, Vim's prompt.
    pub(crate) fn isearch_start_vim(&mut self, forward: bool) {
        self.isearch_start(forward);
        if let Some(s) = self.isearch.as_mut() {
            s.vim = true;
            s.kind = SearchKind::Vim;
        }
    }

    /// The search in progress looks for this kind; the query so far is tried again that way.
    pub(crate) fn isearch_kind(&mut self, kind: SearchKind) {
        let Some(s) = self.isearch.as_mut() else { return };
        s.kind = kind;
        let (q, fwd, origin) = (s.now.query.clone(), s.now.forward, s.origin);
        if !q.is_empty() {
            self.isearch_go(q, fwd, origin, false);
        }
    }

    fn isearch_start(&mut self, forward: bool) {
        let at = self.cursor;
        self.mark_active = false;
        self.isearch = Some(Isearch {
            origin: at,
            kind: SearchKind::Text,
            incomplete: false,
            vim: false,
            fold: true,
            now: Step { query: String::new(), forward, at: None, cursor: at, wrapped: false },
            back: Vec::new(),
        });
    }

    /// A typed character joins the query; the match moves only as far as it must.
    pub fn isearch_char(&mut self, c: char) {
        let Some(s) = &self.isearch else { return };
        let mut q = s.now.query.clone();
        q.push(c);
        // extend in place: the current match (or where the search stands) is tried first
        let from = match (s.now.at, s.now.forward) {
            (Some((a, _)), true) => a,
            (Some((a, _)), false) => a + 1,
            // failing (or a regexp not finished): from where the last match was, else where it began
            (None, true) => s.back.iter().rev().find_map(|st| st.at).map_or(s.origin, |(a, _)| a),
            (None, false) => s.back.iter().rev().find_map(|st| st.at).map_or(s.origin, |(a, _)| a + 1),
        };
        let forward = s.now.forward;
        let wrapped = s.now.wrapped;
        self.isearch_go(q, forward, from, wrapped);
        // noincsearch (Vim): the cursor waits where the search began until Enter
        if !self.emacs && !self.opt_on("incsearch", true) {
            if let Some(o) = self.isearch.as_ref().map(|s| s.origin) {
                self.cursor = o;
            }
        }
        self.want_col = self.cursor().col;
    }

    fn isearch_repeat(&mut self, forward: bool) {
        let Some(s) = &self.isearch else { return };
        // C-s on an empty search: the last one again (Emacs)
        let q = if s.now.query.is_empty() { self.last_search.clone() } else { s.now.query.clone() };
        if q.is_empty() {
            return;
        }
        let len = self.buf.len_chars();
        let (from, wrapped) = match (s.now.at, s.now.query.is_empty()) {
            // a fresh search, or one that just changed direction: from where it stands
            (_, true) => (s.now.cursor, false),
            (Some(_), false) if s.now.forward != forward => (s.now.at.map(|(a, b)| if forward { a } else { b }).unwrap_or(s.now.cursor), s.now.wrapped),
            (Some((a, _)), false) => (if forward { a + 1 } else { a }, s.now.wrapped),
            // failing: around the end, as Emacs does on the second C-s
            (None, false) => (if forward { 0 } else { len }, true),
        };
        self.isearch_go(q, forward, from, wrapped);
    }

    fn isearch_go(&mut self, query: String, forward: bool, from: usize, wrapped: bool) {
        let fold = self.folds_for(&query);
        let kind = self.isearch.as_ref().map_or(SearchKind::Text, |s| s.kind);
        let (at, incomplete) = match self.search_find(&query, kind, from, forward, fold) {
            Ok(at) => (at, false),
            Err(()) => (None, true),
        };
        let Some(s) = &mut self.isearch else { return };
        s.incomplete = incomplete;
        let cursor = s.now.cursor;
        s.fold = fold;
        let prev = std::mem::replace(&mut s.now, Step { query, forward, at, cursor, wrapped });
        s.back.push(prev);
        if let Some((a, b)) = at {
            // forward ends after the match, backward before it (as Emacs leaves point)
            s.now.cursor = if forward { b } else { a };
            self.cursor = s.now.cursor;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Editor;

    pub(super) fn ed(text: &str) -> Editor {
        let mut e = Editor::from_str(text);
        e.set_emacs(true);
        e
    }
    pub(super) fn typed(e: &mut Editor, s: &str) {
        for c in s.chars() {
            e.isearch_char(c);
        }
    }

    #[test]
    fn typing_moves_to_the_match_and_backspace_steps_back() {
        let mut e = ed("alpha beta alpine");
        e.exec("isearch-forward", 1);
        typed(&mut e, "al");
        assert_eq!(e.cursor, 2);
        typed(&mut e, "pi");
        assert_eq!(e.cursor, 15, "alpi is only in alpine");
        e.exec("isearch-delete-char", 1);
        assert_eq!(e.cursor, 3, "back to alp in alpha");
        assert_eq!(e.isearch().unwrap().prompt(), "I-search: alp");
        e.exec("isearch-exit", 1);
        assert!(e.isearch().is_none());
        assert_eq!(e.mark, Some(0));
    }

    #[test]
    fn repeat_wraps_after_failing_and_abort_returns() {
        let mut e = ed("x one x one");
        e.exec("isearch-forward", 1);
        typed(&mut e, "one");
        assert_eq!(e.cursor, 5);
        e.exec("isearch-repeat-forward", 1);
        assert_eq!(e.cursor, 11);
        e.exec("isearch-repeat-forward", 1);
        assert!(e.isearch().unwrap().prompt().starts_with("Failing"));
        e.exec("isearch-repeat-forward", 1);
        assert_eq!(e.cursor, 5);
        assert!(e.isearch().unwrap().prompt().starts_with("Wrapped"));
        e.exec("isearch-abort", 1);
        assert_eq!(e.cursor, 0);
    }

    #[test]
    fn backward_smart_case_and_the_last_search_again() {
        let mut e = ed("Foo foo FOO");
        e.cursor = 11;
        e.exec("isearch-backward", 1);
        typed(&mut e, "foo");
        assert_eq!(e.cursor, 8, "lower case matches FOO");
        e.exec("isearch-exit", 1);
        e.cursor = 11;
        e.exec("isearch-backward", 1);
        typed(&mut e, "F");
        assert_eq!(e.cursor, 8, "a capital: only capital F");
        e.exec("isearch-abort", 1);
        // C-s C-s: the last search again
        e.cursor = 0;
        e.exec("isearch-forward", 1);
        e.exec("isearch-repeat-forward", 1);
        assert_eq!(e.isearch().unwrap().query(), "foo");
        assert_eq!(e.cursor, 3);
    }

    #[test]
    fn up_and_down_keep_the_column_the_search_left() {
        let mut e = ed("abc\nxxalp\n");
        e.exec("isearch-forward", 1);
        typed(&mut e, "alp");
        e.exec("isearch-exit", 1);
        e.exec("previous-line", 1);
        assert_eq!(e.cursor().col, 3);
    }

    #[test]
    fn matches_in_a_range() {
        let mut e = ed("ab ab ab");
        e.exec("isearch-forward", 1);
        typed(&mut e, "ab");
        assert_eq!(e.isearch_matches(0, 8), vec![(0, 2), (3, 5), (6, 8)]);
        assert_eq!(e.isearch_matches(1, 6), vec![(3, 5)]);
    }
}

#[cfg(test)]
mod case_tests {
    use crate::Editor;
    use super::tests::{ed, typed};

    #[test]
    fn ignorecase_and_smartcase_decide_the_match() {
        let mut e = Editor::from_str("Alpha alpha");
        e.set_emacs(true);
        // both on (the default): lower case matches either, a capital only itself
        assert!(e.folds_for("alpha") && !e.folds_for("Alpha"));
        e.set_case(true, false);
        assert!(e.folds_for("Alpha"), "ignorecase alone: always either case");
        e.set_case(false, true);
        assert!(!e.folds_for("alpha"), "noignorecase: exact, whatever smartcase says");
        // a search really follows it
        e.exec("isearch-forward", 1);
        for c in "alpha".chars() {
            e.isearch_char(c);
        }
        assert_eq!(e.isearch().unwrap().current(), Some((6, 11)), "exact: skips Alpha");
    }

    #[test]
    fn regexp_word_and_symbol_searches() {
        let mut e = ed("foo12 bar foo_x foo-bar\nfoo  bar");
        e.exec("isearch-forward-regexp", 1);
        typed(&mut e, r"foo[0-9]+");
        assert_eq!(e.isearch().unwrap().current(), Some((0, 5)));
        assert!(e.isearch().unwrap().prompt().starts_with("Regexp I-search"));
        e.exec("isearch-abort", 1);
        e.exec("isearch-forward-regexp", 1);
        typed(&mut e, r"\(ba");
        assert!(e.isearch().unwrap().prompt().starts_with("Pending"), "an open group is not a failure");
        typed(&mut e, r"r\)");
        assert_eq!(e.isearch().unwrap().current(), Some((6, 9)));
        e.exec("isearch-abort", 1);
        e.exec("isearch-forward-word", 1);
        typed(&mut e, "foo bar");
        assert_eq!(e.isearch().unwrap().current(), Some((16, 23)), "foo-bar: words, whatever is between");
        e.exec("isearch-repeat-forward", 1);
        assert_eq!(e.isearch().unwrap().current(), Some((24, 32)), "across two blanks too");
        e.exec("isearch-abort", 1);
        e.exec("isearch-forward-symbol", 1);
        typed(&mut e, "foo");
        assert_eq!(e.isearch().unwrap().current(), Some((16, 19)), "not foo12 nor foo_x: a whole symbol");
        e.exec("isearch-toggle-symbol", 1);
        assert_eq!(e.isearch().unwrap().current(), Some((0, 3)), "as text again: from where it began");
    }
}
