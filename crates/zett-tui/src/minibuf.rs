//! The minibuffer: one completion UI for M-x, find-file, switch-buffer, help and every picker.
//! Words match in any order (orderless), each candidate has a note (marginalia), choices are
//! remembered per kind (savehist), drawn at the bottom (vertico) or as a float (Vim profile).

use super::*;

/// One choice: what Enter hands over, and the note beside it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Cand {
    pub text: String,
    pub note: String,
    /// keys bound to it, shown before the note (commands)
    pub keys: String,
}

impl Cand {
    pub fn plain(text: impl Into<String>) -> Cand {
        Cand { text: text.into(), note: String::new(), keys: String::new() }
    }
    pub fn noted(text: impl Into<String>, note: impl Into<String>) -> Cand {
        Cand { text: text.into(), note: note.into(), keys: String::new() }
    }
}

/// Where the minibuffer is drawn: `zett.opt.minibuffer`, by default the profile's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Placement {
    Bottom,
    Float,
}

impl Placement {
    pub fn from_opt(v: Option<String>, emacs: bool) -> Placement {
        match v.as_deref().map(str::trim) {
            Some("bottom") => Placement::Bottom,
            Some("float") => Placement::Float,
            _ if emacs => Placement::Bottom,
            _ => Placement::Float,
        }
    }
}

/// What a key did in the minibuffer.
#[derive(Debug, PartialEq)]
pub(crate) enum Done {
    /// still open; the input changed (live sources search again)
    Edited,
    /// still open, nothing for the app to do
    Moved,
    Quit,
    /// Enter: the chosen candidate, or what was typed when nothing matches
    Accept(String),
}

const ROWS: usize = 10;

pub(crate) struct Minibuf {
    pub kind: Pick,
    pub title: String,
    pub items: Vec<Cand>,
    query: String,
    /// cursor in the input, in chars
    at: usize,
    pub sel: usize,
    /// indices into items, best first
    shown: Vec<usize>,
    hist: Vec<String>,
    hist_at: Option<usize>,
    /// widest name and widest keys, for the note columns
    cols: (usize, usize),
}

impl Minibuf {
    pub fn new(kind: Pick, title: impl Into<String>, mut items: Vec<Cand>) -> Minibuf {
        let hist = history(&kind);
        // what was chosen recently first, as vertico + savehist
        if !hist.is_empty() && kind.remembers() {
            let rank = |c: &Cand| hist.iter().position(|h| *h == c.text).unwrap_or(usize::MAX);
            items.sort_by_key(rank);
        }
        let cols = measure(&items);
        let mut m = Minibuf { kind, title: title.into(), items, query: String::new(), at: 0, sel: 0, shown: Vec::new(), hist, hist_at: None, cols };
        m.refilter();
        m
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn set_query(&mut self, q: &str) {
        self.query = q.to_string();
        self.at = q.chars().count();
        self.refilter();
    }

    /// New candidates from a live source (ripgrep), keeping the input.
    pub fn set_items(&mut self, items: Vec<Cand>) {
        self.items = items;
        self.cols = measure(&self.items);
        self.refilter();
    }

    pub fn shown(&self) -> impl Iterator<Item = &Cand> {
        self.shown.iter().map(|&i| &self.items[i])
    }

    pub fn count(&self) -> usize {
        self.shown.len()
    }

    /// Rows for the candidates at the bottom: fixed while it is open, so it does not jump.
    pub fn rows(&self) -> usize {
        if self.kind.live() { ROWS } else { ROWS.min(self.items.len()) }
    }

    pub fn selected(&self) -> Option<&Cand> {
        self.shown.get(self.sel).map(|&i| &self.items[i])
    }

    /// The cursor's column in the input.
    pub fn cursor(&self) -> usize {
        self.at
    }

    fn refilter(&mut self) {
        self.sel = 0;
        if self.kind.live() || self.query.trim().is_empty() {
            self.shown = (0..self.items.len()).collect();
            return;
        }
        let terms = Terms::parse(&self.query);
        let mut scored: Vec<(i32, usize)> = self.items.iter().enumerate().filter_map(|(i, c)| terms.score(c).map(|s| (s, i))).collect();
        // stable: equal scores keep the history order
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(self.items[a.1].text.len().cmp(&self.items[b.1].text.len())));
        self.shown = scored.into_iter().map(|(_, i)| i).collect();
    }

    /// Chars of `text` the input matched, for drawing.
    pub fn hits(&self, text: &str) -> Vec<usize> {
        if self.kind.live() {
            return Vec::new();
        }
        Terms::parse(&self.query).want.iter().flat_map(|t| fuzzy_hits(t, text)).collect()
    }

    /// One key. `emacs`: C-k kills to the end (else it moves up, as in Vim pickers).
    pub fn key(&mut self, k: &KeyEvent, emacs: bool) -> Done {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let alt = k.modifiers.contains(KeyModifiers::ALT);
        let last = self.count().saturating_sub(1);
        let len = self.query.chars().count();
        match k.code {
            KeyCode::Esc => return Done::Quit,
            KeyCode::Char('g') if ctrl => return Done::Quit,
            KeyCode::Enter => {
                let chosen = self.selected().map(|c| c.text.clone()).unwrap_or_else(|| self.query.clone());
                return Done::Accept(chosen);
            }
            // moving through the candidates
            KeyCode::Down => self.sel = (self.sel + 1).min(last),
            KeyCode::Up => self.sel = self.sel.saturating_sub(1),
            KeyCode::Char('n' | 'j') if ctrl => self.sel = (self.sel + 1).min(last),
            KeyCode::Char('p') if ctrl => self.sel = self.sel.saturating_sub(1),
            KeyCode::Char('k') if ctrl && !emacs => self.sel = self.sel.saturating_sub(1),
            KeyCode::PageDown => self.sel = (self.sel + ROWS).min(last),
            KeyCode::Char('v') if ctrl => self.sel = (self.sel + ROWS).min(last),
            KeyCode::PageUp => self.sel = self.sel.saturating_sub(ROWS),
            KeyCode::Char('v') if alt => self.sel = self.sel.saturating_sub(ROWS),
            KeyCode::Char('<') if alt => self.sel = 0,
            KeyCode::Char('>') if alt => self.sel = last,
            // history: what was chosen before
            KeyCode::Char('p') if alt => return self.recall(1),
            KeyCode::Char('n') if alt => return self.recall(-1),
            // Tab puts the candidate into the input, to go on from there
            KeyCode::Tab => {
                if let Some(t) = self.selected().map(|c| c.text.clone()) {
                    self.set_query(&t);
                    return Done::Edited;
                }
            }
            // editing the input
            KeyCode::Left => self.at = self.at.saturating_sub(1),
            KeyCode::Char('b') if ctrl => self.at = self.at.saturating_sub(1),
            KeyCode::Right => self.at = (self.at + 1).min(len),
            KeyCode::Char('f') if ctrl => self.at = (self.at + 1).min(len),
            KeyCode::Home => self.at = 0,
            KeyCode::Char('a') if ctrl => self.at = 0,
            KeyCode::End => self.at = len,
            KeyCode::Char('e') if ctrl => self.at = len,
            KeyCode::Char('b') if alt => self.at = self.word_left(),
            KeyCode::Char('f') if alt => self.at = self.word_right(),
            KeyCode::Backspace if alt || ctrl => return self.cut(self.word_left(), self.at),
            KeyCode::Char('w') if ctrl => return self.cut(self.word_left(), self.at),
            KeyCode::Char('d') if alt => return self.cut(self.at, self.word_right()),
            KeyCode::Backspace => return self.cut(self.at.saturating_sub(1), self.at),
            KeyCode::Delete => return self.cut(self.at, (self.at + 1).min(len)),
            KeyCode::Char('d') if ctrl => return self.cut(self.at, (self.at + 1).min(len)),
            KeyCode::Char('k') if ctrl => return self.cut(self.at, len),
            KeyCode::Char('u') if ctrl => return self.cut(0, self.at),
            KeyCode::Char(c) if !ctrl && !alt => {
                let b = self.byte(self.at);
                self.query.insert(b, c);
                self.at += 1;
                self.hist_at = None;
                self.refilter();
                return Done::Edited;
            }
            _ => {}
        }
        Done::Moved
    }

    fn byte(&self, ch: usize) -> usize {
        self.query.char_indices().nth(ch).map(|(b, _)| b).unwrap_or(self.query.len())
    }

    /// Removes chars `from..to` of the input.
    fn cut(&mut self, from: usize, to: usize) -> Done {
        if from >= to {
            return Done::Moved;
        }
        let (a, b) = (self.byte(from), self.byte(to));
        self.query.replace_range(a..b, "");
        self.at = from;
        self.refilter();
        Done::Edited
    }

    fn word_left(&self) -> usize {
        let cs: Vec<char> = self.query.chars().collect();
        let mut i = self.at;
        while i > 0 && !cs[i - 1].is_alphanumeric() {
            i -= 1;
        }
        while i > 0 && cs[i - 1].is_alphanumeric() {
            i -= 1;
        }
        i
    }

    fn word_right(&self) -> usize {
        let cs: Vec<char> = self.query.chars().collect();
        let mut i = self.at;
        while i < cs.len() && !cs[i].is_alphanumeric() {
            i += 1;
        }
        while i < cs.len() && cs[i].is_alphanumeric() {
            i += 1;
        }
        i
    }

    /// M-p / M-n: an older (+1) or newer (-1) choice into the input.
    fn recall(&mut self, by: isize) -> Done {
        let next = match (self.hist_at, by) {
            (None, 1) if !self.hist.is_empty() => Some(0),
            (None, _) => return Done::Moved,
            (Some(0), -1) => None,
            (Some(i), _) => Some((i as isize + by).clamp(0, self.hist.len() as isize - 1) as usize),
        };
        let text = next.map(|i| self.hist[i].clone()).unwrap_or_default();
        self.set_query(&text);
        self.hist_at = next;
        Done::Edited
    }
}

/// The widest name and the widest keys.
fn measure(items: &[Cand]) -> (usize, usize) {
    let w = |f: fn(&Cand) -> &str| items.iter().map(|c| f(c).chars().count()).max().unwrap_or(0);
    (w(|c| &c.text), w(|c| &c.keys))
}

/// The input split orderless-style: every word must match, in any order; `!word` excludes.
struct Terms {
    want: Vec<Vec<char>>,
    not: Vec<String>,
}

impl Terms {
    fn parse(q: &str) -> Terms {
        let (mut want, mut not) = (Vec::new(), Vec::new());
        for w in q.split_whitespace() {
            match w.strip_prefix('!') {
                Some("") => {}
                Some(x) => not.push(x.to_lowercase()),
                None => want.push(w.to_lowercase().chars().collect()),
            }
        }
        Terms { want, not }
    }

    /// A word matches the text (fuzzy) or, worth less, the note (as written).
    fn score(&self, c: &Cand) -> Option<i32> {
        let text = c.text.to_lowercase();
        if self.not.iter().any(|n| text.contains(n.as_str())) {
            return None;
        }
        let note = c.note.to_lowercase();
        let mut total = 0;
        for w in &self.want {
            total += match fuzzy(w, &c.text) {
                Some(s) => s,
                None if note.contains(&w.iter().collect::<String>()) => 0,
                None => return None,
            };
        }
        Some(total)
    }
}

/// Subsequence match: letters in order; runs and starts of names score higher.
pub(crate) fn fuzzy(q: &[char], s: &str) -> Option<i32> {
    let chars: Vec<char> = s.to_lowercase().chars().collect();
    let name_start = s.rfind('/').map(|i| s[..i + 1].chars().count()).unwrap_or(0);
    let (mut qi, mut score, mut last) = (0, 0i32, None::<usize>);
    for (i, c) in chars.iter().enumerate() {
        if qi < q.len() && *c == q[qi] {
            score += 1;
            if last == Some(i.wrapping_sub(1)) {
                score += 4;
            }
            if i == name_start || i == 0 || matches!(chars.get(i.wrapping_sub(1)), Some('/' | '_' | '-' | '.')) {
                score += 6;
            }
            if i >= name_start {
                score += 2;
            }
            last = Some(i);
            qi += 1;
        }
    }
    (qi == q.len()).then_some(score)
}

/// Where fuzzy's letters landed (the same greedy walk).
fn fuzzy_hits(q: &[char], s: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut qi = 0;
    for (i, c) in s.to_lowercase().chars().enumerate() {
        if qi < q.len() && c == q[qi] {
            out.push(i);
            qi += 1;
        }
    }
    if qi == q.len() { out } else { Vec::new() }
}

/// Keys as Emacs writes them: `<C-x><C-f>` → `C-x C-f`, `<M-lt>` → `M-<`.
pub(crate) fn emacs_keys(typed: &str) -> String {
    zett_script::keys(typed)
        .iter()
        .map(|k| {
            let inner = k.strip_prefix('<').and_then(|k| k.strip_suffix('>')).unwrap_or(k);
            let (mods, key) = inner.rsplit_once('-').filter(|(_, k)| !k.is_empty()).unwrap_or(("", inner));
            let key = match key {
                "lt" => "<",
                "gt" => ">",
                "Enter" | "CR" => "RET",
                "Space" => "SPC",
                "BS" => "DEL",
                "Tab" => "TAB",
                "Esc" => "ESC",
                k => k,
            };
            // the engine writes Alt as A-; Emacs says M-
            let mods: Vec<&str> = mods.split('-').filter(|m| !m.is_empty()).map(|m| if m == "A" { "M" } else { m }).collect();
            if mods.is_empty() { key.to_string() } else { format!("{}-{key}", mods.join("-")) }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn hist_file(kind: &Pick) -> PathBuf {
    doom::state_dir().join("history").join(kind.name())
}

/// What was chosen in this kind of minibuffer, newest first.
fn history(kind: &Pick) -> Vec<String> {
    std::fs::read_to_string(hist_file(kind)).map(|t| t.lines().filter(|l| !l.is_empty()).map(String::from).collect()).unwrap_or_default()
}

fn remember_choice(kind: &Pick, chosen: &str) {
    if chosen.is_empty() || chosen.contains('\n') || !kind.remembers() {
        return;
    }
    let mut list = history(kind);
    list.retain(|x| x != chosen);
    list.insert(0, chosen.to_string());
    list.truncate(100);
    let f = hist_file(kind);
    if let Some(dir) = f.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(f, list.join("\n") + "\n");
}

impl App {
    pub(crate) fn minibuf_key(&mut self, k: KeyEvent) {
        let emacs = self.emacs_keys();
        let Some(m) = self.minibuf.as_mut() else { return };
        match m.key(&k, emacs) {
            Done::Moved => {}
            Done::Edited => self.grep_update(),
            Done::Quit => self.minibuf = None,
            Done::Accept(text) => {
                let m = self.minibuf.take().expect("open");
                if m.count() == 0 && !m.kind.takes_typed() {
                    return self.say(format!("no match for {:?}", m.query()));
                }
                remember_choice(&m.kind, &text);
                self.picked(m.kind, text);
            }
        }
    }

    /// `list`: the rows the layout set aside for the candidates (Bottom); `cmd`: the echo area.
    pub(crate) fn draw_minibuf(&self, f: &mut Frame, list: Rect, cmd: Rect) {
        let Some(m) = &self.minibuf else { return };
        match self.placement {
            Placement::Bottom => draw_bottom(f, m, list, cmd),
            Placement::Float => draw_float(f, m),
        }
    }
}

/// One candidate row: the matched letters lit, directories dimmed, keys and note to the right.
fn row<'a>(m: &Minibuf, c: &'a Cand, sel: bool, width: usize) -> Line<'a> {
    let base = if sel { hl("PickerSel") } else { Style::default() };
    let hits = m.hits(&c.text);
    let dir_end = c.text.rfind('/').map(|i| c.text[..i + 1].chars().count()).unwrap_or(0);
    let mut spans = vec![Span::styled(if sel { " ▌ " } else { "   " }, base.patch(hl("PickerPrompt")))];
    for (i, ch) in c.text.chars().enumerate() {
        let st = if hits.contains(&i) {
            base.patch(hl("PickerMatch"))
        } else if i < dir_end {
            base.patch(hl("PickerDir"))
        } else {
            base
        };
        spans.push(Span::styled(ch.to_string(), st));
    }
    // keys and note in columns after the widest name (marginalia); too narrow, the note is cut
    let used = 3 + c.text.chars().count();
    let (name_w, keys_w) = m.cols;
    let col = (3 + name_w.min(width * 45 / 100) + 2).max(used + 2);
    if (!c.keys.is_empty() || !c.note.is_empty()) && col < width {
        let keys = format!("{:<w$}", c.keys, w = if keys_w > 0 { keys_w + 2 } else { 0 });
        spans.push(Span::styled(" ".repeat(col - used), base));
        let mut room = width - col;
        for (t, g) in [(keys.as_str(), "PickerKey"), (c.note.as_str(), "PickerNote")] {
            let t: String = t.chars().take(room).collect();
            room -= t.chars().count();
            spans.push(Span::styled(t, base.patch(hl(g))));
        }
    }
    let filled: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    if sel && filled < width {
        spans.push(Span::styled(" ".repeat(width - filled), base));
    }
    Line::from(spans)
}

fn rows<'a>(m: &'a Minibuf, n: usize, width: usize) -> Vec<Line<'a>> {
    let first = m.sel.saturating_sub(n.saturating_sub(1));
    m.shown().enumerate().skip(first).take(n).map(|(i, c)| row(m, c, i == m.sel, width)).collect()
}

/// Vertico: the candidates just above the echo area, the input in it.
fn draw_bottom(f: &mut Frame, m: &Minibuf, list: Rect, cmd: Rect) {
    f.render_widget(Clear, list);
    f.render_widget(Paragraph::new(rows(m, list.height as usize, list.width as usize)), list);
    let count = format!("{}/{} ", if m.count() == 0 { 0 } else { m.sel + 1 }, m.count());
    // titles end in their own separator where Emacs has one ("M-x ", "Find file: ")
    let prompt = if m.title.ends_with(' ') { m.title.clone() } else { format!("{}: ", m.title) };
    f.render_widget(Clear, cmd);
    f.render_widget(
        Paragraph::new(Line::from(vec![Span::styled(count.clone(), hl("PickerDir")), Span::styled(prompt.clone(), hl("Prompt")), Span::raw(m.query())])),
        cmd,
    );
    let x = cmd.x + (count.chars().count() + prompt.chars().count() + m.cursor()) as u16;
    f.set_cursor_position(Position { x: x.min(cmd.right().saturating_sub(1)), y: cmd.y });
}

/// The Vim profile's float: framed, centred.
fn draw_float(f: &mut Frame, m: &Minibuf) {
    let area = f.area();
    let w = (area.width * 7 / 10).max(40).min(area.width);
    let h = (area.height * 7 / 10).max(10).min(area.height);
    let a = Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height - h) / 3, width: w, height: h };
    f.render_widget(Clear, a);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(hl("FloatBorder"))
        .title(format!(" {} ", m.title.trim_end()))
        .title_bottom(format!(" {} of {} ", m.count(), m.items.len()));
    let inner = block.inner(a);
    f.render_widget(block, a);
    let [input, list] = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).areas(inner);
    f.render_widget(Paragraph::new(Line::from(vec![Span::styled(" ❯ ", hl("PickerPrompt")), Span::raw(m.query())])), input);
    f.set_cursor_position(Position { x: input.x + 3 + m.cursor() as u16, y: input.y });
    f.render_widget(Paragraph::new(rows(m, list.height as usize, list.width as usize)), list);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mb(items: &[&str]) -> Minibuf {
        let mut m = Minibuf { kind: Pick::Lines, title: "t".into(), items: items.iter().map(|s| Cand::plain(*s)).collect(), query: String::new(), at: 0, sel: 0, shown: Vec::new(), hist: Vec::new(), hist_at: None, cols: (0, 0) };
        m.refilter();
        m
    }
    fn texts(m: &Minibuf) -> Vec<&str> {
        m.shown().map(|c| c.text.as_str()).collect()
    }
    fn press(m: &mut Minibuf, code: KeyCode, mods: KeyModifiers) -> Done {
        m.key(&KeyEvent::new(code, mods), true)
    }
    fn typed(m: &mut Minibuf, s: &str) {
        for c in s.chars() {
            press(m, KeyCode::Char(c), KeyModifiers::NONE);
        }
    }

    #[test]
    fn fuzzy_prefers_name_starts_and_runs() {
        let q: Vec<char> = "main".chars().collect();
        let a = fuzzy(&q, "src/main.rs").unwrap();
        let b = fuzzy(&q, "src/my_animation.rs").unwrap();
        assert!(a > b, "{a} vs {b}");
        assert!(fuzzy(&q, "src/lib.rs").is_none());
    }

    #[test]
    fn words_match_in_any_order_and_bang_excludes() {
        let mut m = mb(&["save-file", "find-file", "file-save-as", "kill-buffer"]);
        typed(&mut m, "file save");
        assert_eq!(texts(&m), ["save-file", "file-save-as"]);
        typed(&mut m, " !as");
        assert_eq!(texts(&m), ["save-file"]);
    }

    #[test]
    fn a_word_can_match_the_note() {
        let mut m = mb(&[]);
        m.items = vec![Cand::noted("kill-line", "Kill to the end of the line"), Cand::noted("yank", "Paste the last kill")];
        m.set_query("paste");
        assert_eq!(texts(&m), ["yank"]);
    }

    #[test]
    fn editing_keys_work_in_the_middle() {
        let mut m = mb(&[]);
        typed(&mut m, "find file");
        press(&mut m, KeyCode::Char('a'), KeyModifiers::CONTROL);
        press(&mut m, KeyCode::Char('d'), KeyModifiers::ALT);
        assert_eq!(m.query(), " file");
        press(&mut m, KeyCode::Char('e'), KeyModifiers::CONTROL);
        press(&mut m, KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert_eq!((m.query(), m.cursor()), (" ", 1));
        press(&mut m, KeyCode::Char('a'), KeyModifiers::CONTROL);
        typed(&mut m, "ab");
        press(&mut m, KeyCode::Char('k'), KeyModifiers::CONTROL);
        assert_eq!(m.query(), "ab");
    }

    #[test]
    fn enter_gives_the_selection_or_what_was_typed() {
        let mut m = mb(&["alpha", "beta"]);
        press(&mut m, KeyCode::Char('n'), KeyModifiers::CONTROL);
        assert_eq!(press(&mut m, KeyCode::Enter, KeyModifiers::NONE), Done::Accept("beta".into()));
        let mut m = mb(&["alpha"]);
        typed(&mut m, "new.md");
        assert_eq!(press(&mut m, KeyCode::Enter, KeyModifiers::NONE), Done::Accept("new.md".into()));
    }

    #[test]
    fn tab_takes_the_candidate_into_the_input() {
        let mut m = mb(&["src/main.rs", "README.md"]);
        typed(&mut m, "mai");
        press(&mut m, KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!((m.query(), m.cursor()), ("src/main.rs", 11));
    }

    #[test]
    fn history_recalls_older_and_newer() {
        let mut m = mb(&[]);
        m.hist = vec!["newest".into(), "older".into()];
        press(&mut m, KeyCode::Char('p'), KeyModifiers::ALT);
        press(&mut m, KeyCode::Char('p'), KeyModifiers::ALT);
        assert_eq!(m.query(), "older");
        press(&mut m, KeyCode::Char('n'), KeyModifiers::ALT);
        assert_eq!(m.query(), "newest");
        press(&mut m, KeyCode::Char('n'), KeyModifiers::ALT);
        assert_eq!(m.query(), "");
    }

    #[test]
    fn keys_written_the_emacs_way() {
        assert_eq!(emacs_keys("<C-x><C-f>"), "C-x C-f");
        assert_eq!(emacs_keys("<M-lt>"), "M-<");
        assert_eq!(emacs_keys("<C-x>b"), "C-x b");
        assert_eq!(emacs_keys("<C-Space>"), "C-SPC");
    }

    #[test]
    fn placement_follows_the_profile_unless_set() {
        assert_eq!(Placement::from_opt(None, true), Placement::Bottom);
        assert_eq!(Placement::from_opt(None, false), Placement::Float);
        assert_eq!(Placement::from_opt(Some("bottom".into()), false), Placement::Bottom);
    }
}
