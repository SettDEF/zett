//! Text objects: iw aw i" a( ip …

use super::*;

impl Editor {
    // ── text objects ──

    pub(crate) fn text_object_key(&mut self, inner: bool, obj: char) {
        let Some((s, e, lines)) = self.text_object(inner, obj) else {
            // an object zett does not know is a key no command took; one not found here just cancels
            if !"wW\"'`()b[]{}B<>pst".contains(obj) {
                return self.vim_unknown();
            }
            self.op = None;
            self.count = None;
            return;
        };
        match self.mode {
            Mode::Visual | Mode::VisualLine => {
                if lines && self.mode == Mode::Visual {
                    self.mode = Mode::VisualLine;
                }
                self.anchor = s;
                self.cursor = e.saturating_sub(1).max(s);
            }
            _ if self.op.is_some() => {
                let span = if lines { Span::Lines(self.buf.to_pos(s).line, self.buf.to_pos(e.saturating_sub(1).max(s)).line) } else { Span::Chars(s, e) };
                self.motion(span, None);
            }
            _ => {}
        }
    }

    /// The text object at the cursor: (start, end exclusive, whole lines?).
    pub(crate) fn text_object(&self, inner: bool, obj: char) -> Option<(usize, usize, bool)> {
        match obj {
            'w' | 'W' => self.word_object(inner, obj == 'W'),
            '"' | '\'' | '`' => self.quote_object(inner, obj),
            '(' | ')' | 'b' => self.bracket_object(inner, '(', ')'),
            '[' | ']' => self.bracket_object(inner, '[', ']'),
            '{' | '}' | 'B' => self.bracket_object(inner, '{', '}'),
            '<' | '>' => self.bracket_object(inner, '<', '>'),
            'p' => self.paragraph_object(inner),
            's' => self.sentence_object(inner),
            't' => self.tag_object(inner),
            _ => None,
        }
    }

    pub(crate) fn word_object(&self, inner: bool, big: bool) -> Option<(usize, usize, bool)> {
        let line = self.cur_line();
        let (ls, le) = (self.line_start(line), self.line_end(line));
        if ls == le {
            return None;
        }
        let i = self.cursor.min(le - 1);
        let kind = |c: char| if c.is_whitespace() { 0 } else if big { 1 } else { class(c) };
        let k = kind(self.char(i)?);
        let (mut s, mut e) = (i, i + 1);
        while s > ls && self.char(s - 1).is_some_and(|c| kind(c) == k) {
            s -= 1;
        }
        while e < le && self.char(e).is_some_and(|c| kind(c) == k) {
            e += 1;
        }
        if inner {
            return Some((s, e, false));
        }
        if k == 0 {
            // on spaces: the spaces and the word after them
            let k2 = self.char(e).map(kind);
            while e < le && self.char(e).is_some_and(|c| Some(kind(c)) == k2) {
                e += 1;
            }
            return Some((s, e, false));
        }
        // a word: with the spaces after it, or (at the line end) the ones before it
        let mut e2 = e;
        while e2 < le && self.char(e2).is_some_and(|c| c == ' ' || c == '\t') {
            e2 += 1;
        }
        if e2 > e {
            return Some((s, e2, false));
        }
        let mut s2 = s;
        while s2 > ls && self.char(s2 - 1).is_some_and(|c| c == ' ' || c == '\t') {
            s2 -= 1;
        }
        Some((s2, e, false))
    }

    pub(crate) fn quote_object(&self, inner: bool, q: char) -> Option<(usize, usize, bool)> {
        let line = self.cur_line();
        let (ls, le) = (self.line_start(line), self.line_end(line));
        let quotes: Vec<usize> = (ls..le).filter(|i| self.char(*i) == Some(q) && (*i == ls || self.char(i - 1) != Some('\\'))).collect();
        let cur = self.cursor;
        // the pair around the cursor, else the first pair after it (as Vim does)
        let mut pair = None;
        for w in quotes.chunks(2) {
            if let [a, b] = w {
                if cur >= *a && cur <= *b {
                    pair = Some((*a, *b));
                    break;
                }
                if pair.is_none() && *a > cur {
                    pair = Some((*a, *b));
                    break;
                }
            }
        }
        let (a, b) = pair?;
        if inner {
            return Some((a + 1, b, false));
        }
        // around: with the spaces after, or else the ones before
        let mut e = b + 1;
        while e < le && self.char(e).is_some_and(|c| c == ' ' || c == '\t') {
            e += 1;
        }
        let mut s = a;
        if e == b + 1 {
            while s > ls && self.char(s - 1).is_some_and(|c| c == ' ' || c == '\t') {
                s -= 1;
            }
        }
        Some((s, e, false))
    }

    pub(crate) fn bracket_object(&self, inner: bool, open: char, close: char) -> Option<(usize, usize, bool)> {
        let len = self.buf.len_chars();
        // the open bracket: the one under the cursor, or the nearest unmatched one before it
        let mut depth = 0i32;
        let mut i = self.cursor.min(len.saturating_sub(1));
        let start = loop {
            let c = self.char(i)?;
            if c == close && i != self.cursor {
                depth += 1;
            } else if c == open {
                if depth == 0 {
                    break i;
                }
                depth -= 1;
            }
            if i == 0 {
                return None;
            }
            i -= 1;
        };
        let mut depth = 0i32;
        let mut j = start + 1;
        let end = loop {
            let c = self.char(j)?;
            if c == open {
                depth += 1;
            } else if c == close {
                if depth == 0 {
                    break j;
                }
                depth -= 1;
            }
            j += 1;
            if j >= len {
                return None;
            }
        };
        if !inner {
            return Some((start, end + 1, false));
        }
        let (mut s, mut e) = (start + 1, end);
        // a block over several lines: the line break after the open bracket and the
        // indent before the close one are not inside
        if self.char(s) == Some('\n') {
            s += 1;
            let close_line = self.buf.to_pos(end).line;
            let cs = self.line_start(close_line);
            if (cs..end).all(|k| self.char(k).is_some_and(|c| c == ' ' || c == '\t')) && cs > s {
                e = cs;
                return Some((s, e, true));
            }
        }
        Some((s, e.max(s), false))
    }

    pub(crate) fn paragraph_object(&self, inner: bool) -> Option<(usize, usize, bool)> {
        let last = self.buf.len_lines() - 1;
        let blank = |l: usize| self.buf.line(l).trim().is_empty();
        let l = self.cur_line();
        let kind = blank(l);
        let (mut a, mut b) = (l, l);
        while a > 0 && blank(a - 1) == kind {
            a -= 1;
        }
        while b < last && blank(b + 1) == kind {
            b += 1;
        }
        if !inner {
            // with the blank lines after it (or before, at the end)
            let b0 = b;
            while b < last && blank(b + 1) != kind {
                b += 1;
                if !blank(b) {
                    b -= 1;
                    break;
                }
            }
            if b == b0 {
                while a > 0 && blank(a - 1) != kind && blank(a - 1) {
                    a -= 1;
                }
            }
        }
        let e = if b + 1 <= last { self.line_start(b + 1) } else { self.buf.len_chars() };
        Some((self.line_start(a), e, true))
    }

    pub(crate) fn is_word_end(&self, i: usize) -> bool {
        match (self.char(i), self.char(i + 1)) {
            (Some(a), Some(b)) => class(a) != class(b) || b.is_whitespace(),
            (Some(_), None) => true,
            _ => false,
        }
    }

}

impl Editor {
    /// `is` / `as`: the sentence around the cursor (`as` with the blanks after it).
    pub(crate) fn sentence_object(&self, inner: bool) -> Option<(usize, usize, bool)> {
        let n = self.buf.len_chars();
        if n == 0 {
            return None;
        }
        let ends = |e: &Self, k: usize| e.char(k).is_some_and(|c| matches!(c, '.' | '!' | '?')) && e.char(k + 1).is_none_or(char::is_whitespace);
        let para = |e: &Self, k: usize| e.char(k) == Some('\n') && e.char(k + 1) == Some('\n');
        let mut s = self.cursor.min(n - 1);
        while s > 0 && !ends(self, s - 1) && !para(self, s - 1) {
            s -= 1;
        }
        while s < n && self.char(s).is_some_and(char::is_whitespace) {
            s += 1;
        }
        let mut e = self.cursor.max(s);
        while e < n && !ends(self, e) && !para(self, e) {
            e += 1;
        }
        let mut e = (e + 1).min(n);
        if !inner {
            while e < n && self.char(e).is_some_and(|c| c == ' ' || c == '\t') {
                e += 1;
            }
        }
        (e > s).then_some((s, e, false))
    }

    /// `it` / `at`: the XML/HTML tag pair around the cursor (inner: what is between them).
    pub(crate) fn tag_object(&self, inner: bool) -> Option<(usize, usize, bool)> {
        let text: Vec<char> = self.buf.text().chars().collect();
        let at = self.cursor;
        // each opening tag before the cursor, nearest first, whose closing tag comes after it
        let mut i = at.min(text.len().saturating_sub(1));
        loop {
            if text.get(i) == Some(&'<') && text.get(i + 1).is_some_and(|c| c.is_alphabetic()) {
                let name: String = text[i + 1..].iter().take_while(|c| c.is_alphanumeric() || **c == '-' || **c == ':').collect();
                let open_end = (i..text.len()).find(|&k| text[k] == '>')?;
                if text[open_end.saturating_sub(1)] != '/' {
                    // the matching close, counting nested tags of the same name
                    let (open_pat, close_pat): (Vec<char>, Vec<char>) = (format!("<{name}").chars().collect(), format!("</{name}>").chars().collect());
                    let mut depth = 1;
                    let mut k = open_end + 1;
                    while k < text.len() {
                        if text[k..].starts_with(&close_pat) {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        } else if text[k..].starts_with(&open_pat) && text.get(k + open_pat.len()).is_some_and(|c| !c.is_alphanumeric()) {
                            depth += 1;
                        }
                        k += 1;
                    }
                    if depth == 0 && k + close_pat.len() > at {
                        return Some(if inner { (open_end + 1, k, false) } else { (i, k + close_pat.len(), false) });
                    }
                }
            }
            if i == 0 {
                return None;
            }
            i -= 1;
        }
    }
}
