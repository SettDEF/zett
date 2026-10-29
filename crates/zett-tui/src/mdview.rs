//! Markdown rendered in the window (`:MarkdownView`, `markdown-view-mode`, C-c C-v): headings, emphasis without
//! its markers, bullets and checkboxes, quotes, code blocks coloured by their language, tables, rules, images.
//! Every terminal gets the styled text; zash also draws headings large, code in panels and tables as cards
//! (OSC 7733;ui) and pictures (Kitty graphics). j/k move, x ticks a task (in the source), e / Enter / q back.
use super::*;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;

/// A rendered row: its pieces, the source line it came from, and what zash may draw over it.
#[derive(Clone, Debug, Default)]
pub(crate) struct Row {
    pub spans: Vec<(Style, String)>,
    pub src: usize,
    pub hint: Option<Hint>,
    /// a task's checkbox on this row: its source line
    pub task: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Hint {
    /// the first row of a heading (rows: how many it takes), its words
    Heading { level: u8, text: String, rows: usize },
    /// the first row of a code block: its height
    Code { rows: usize },
    /// the first row of a table: its height, column widths, header words
    Table { rows: usize, cols: Vec<usize>, head: Vec<String> },
    /// a picture's first row: its file and height
    Image { path: PathBuf, rows: usize },
}

/// Per buffer in the view: where it is scrolled to and the chosen row; the rendering kept by (revision, width).
#[derive(Default)]
pub(crate) struct State {
    pub top: usize,
    pub sel: usize,
    cache: Option<(u64, u16, Vec<Row>)>,
}

/// What was last told to zash (to send only changes), and the pictures on the screen.
#[derive(Default)]
pub(crate) struct Sent {
    ui: String,
    imgs: Vec<u32>,
    imgs_key: String,
    /// pictures sent to the terminal: id, their pixel size
    sent: HashMap<u32, (u32, u32)>,
}

fn level(l: HeadingLevel) -> u8 {
    match l {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Pieces wrapped to `width` columns: the first row starts with `first`, the rest with `rest` (same width).
fn wrap(chunks: &[(Style, String)], width: usize, first: &[(Style, String)], rest: &[(Style, String)]) -> Vec<Vec<(Style, String)>> {
    let lead = first.iter().map(|(_, s)| s.chars().count()).sum::<usize>();
    let room = width.saturating_sub(lead).max(8);
    let mut out: Vec<Vec<(Style, String)>> = Vec::new();
    let mut line: Vec<(Style, String)> = first.to_vec();
    let mut used = 0usize;
    // words, each keeping its style (a space before it unless at a row's start)
    // a space anywhere between two words (inside a piece or at its edge) is kept as one
    let mut words: Vec<(Style, String, bool)> = Vec::new();
    let mut pending = false;
    for (st, s) in chunks {
        for (i, w) in s.split(' ').enumerate() {
            if i > 0 {
                pending = true;
            }
            if !w.is_empty() {
                words.push((*st, w.to_string(), pending));
                pending = false;
            }
        }
    }
    for (st, w, space) in words {
        if w.is_empty() {
            continue;
        }
        let n = w.chars().count();
        let sp = usize::from(space && used > 0);
        if used > 0 && used + sp + n > room {
            out.push(std::mem::replace(&mut line, rest.to_vec()));
            used = 0;
        }
        if used > 0 && space {
            line.push((st, " ".into()));
            used += 1;
        }
        line.push((st, w));
        used += n;
    }
    if used > 0 || out.is_empty() {
        out.push(line);
    }
    out
}

/// The rendering of `md` at `width` columns. `code` colours a code block's lines in a language.
pub(crate) fn render(md: &str, width: usize, base: &Path, code: &dyn Fn(&str, &[String]) -> Vec<Vec<(Style, String)>>) -> Vec<Row> {
    let starts: Vec<usize> = std::iter::once(0).chain(md.match_indices('\n').map(|(i, _)| i + 1)).collect();
    let line_of = |off: usize| starts.partition_point(|&s| s <= off).saturating_sub(1);
    let mut rows: Vec<Row> = Vec::new();
    let mut chunks: Vec<(Style, String)> = Vec::new();
    let mut styles: Vec<Style> = vec![Style::default()];
    let mut lists: Vec<Option<u64>> = Vec::new();
    let mut quote = 0usize;
    let mut heading: Option<u8> = None;
    let mut code_block: Option<(String, Vec<String>, usize)> = None;
    let mut table: Option<(Vec<Vec<String>>, usize)> = None;
    let mut cell = String::new();
    let mut task: Option<(bool, usize)> = None;
    let mut item_start = false;
    let mut src = 0usize;
    let mut image: Option<(String, String)> = None;
    let blank = |rows: &mut Vec<Row>, src: usize| {
        if rows.last().is_some_and(|r| !r.spans.is_empty()) {
            rows.push(Row { src, ..Default::default() });
        }
    };
    let quote_bar = |q: usize| -> Vec<(Style, String)> { (0..q).map(|_| (hl("Comment"), "│ ".to_string())).collect() };
    for (ev, range) in Parser::new_ext(md, Options::ENABLE_TABLES | Options::ENABLE_TASKLISTS | Options::ENABLE_STRIKETHROUGH).into_offset_iter() {
        let here = line_of(range.start);
        match ev {
            Event::Start(Tag::Paragraph) => src = here,
            Event::Start(Tag::Heading { level: l, .. }) => {
                blank(&mut rows, here);
                heading = Some(level(l));
                src = here;
            }
            Event::Start(Tag::BlockQuote(_)) => {
                blank(&mut rows, here);
                quote += 1;
            }
            Event::Start(Tag::List(first)) => {
                // an item's own words before its sub-list
                flush(&mut rows, &mut chunks, width, quote, &lists, &mut task, &mut item_start, src);
                if lists.is_empty() {
                    blank(&mut rows, here);
                }
                lists.push(first);
            }
            Event::Start(Tag::Item) => {
                item_start = true;
                src = here;
            }
            Event::TaskListMarker(done) => task = Some((done, here)),
            Event::Start(Tag::CodeBlock(kind)) => {
                blank(&mut rows, here);
                let lang = match kind { pulldown_cmark::CodeBlockKind::Fenced(l) => l.split(|c: char| c == ',' || c.is_whitespace()).next().unwrap_or("").to_string(), _ => String::new() };
                code_block = Some((lang, Vec::new(), here + 1));
            }
            Event::Start(Tag::Table(_)) => {
                blank(&mut rows, here);
                table = Some((Vec::new(), here));
            }
            Event::Start(Tag::TableHead) | Event::Start(Tag::TableRow) => {
                if let Some((t, _)) = table.as_mut() { t.push(Vec::new()) }
            }
            Event::Start(Tag::TableCell) => cell.clear(),
            Event::End(TagEnd::TableCell) => {
                if let Some((t, _)) = table.as_mut() { if let Some(r) = t.last_mut() { r.push(std::mem::take(&mut cell)) } }
            }
            Event::Start(Tag::Emphasis) => { let s = styles.last().copied().unwrap_or_default().add_modifier(Modifier::ITALIC); styles.push(s) }
            Event::Start(Tag::Strong) => { let s = styles.last().copied().unwrap_or_default().add_modifier(Modifier::BOLD); styles.push(s) }
            Event::Start(Tag::Strikethrough) => { let s = styles.last().copied().unwrap_or_default().add_modifier(Modifier::CROSSED_OUT); styles.push(s) }
            Event::Start(Tag::Link { .. }) => { let s = styles.last().copied().unwrap_or_default().patch(hl("Underlined")).add_modifier(Modifier::UNDERLINED); styles.push(s) }
            Event::Start(Tag::Image { dest_url, .. }) => image = Some((dest_url.to_string(), String::new())),
            Event::End(TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link) => { styles.pop(); }
            Event::Text(t) => {
                if let Some((_, lines, _)) = code_block.as_mut() {
                    lines.extend(t.lines().map(str::to_string));
                } else if let Some((_, alt)) = image.as_mut() {
                    alt.push_str(&t);
                } else if table.is_some() {
                    cell.push_str(&t);
                } else {
                    chunks.push((styles.last().copied().unwrap_or_default(), t.to_string()));
                }
            }
            Event::Code(t) => {
                if table.is_some() { cell.push_str(&t) } else { chunks.push((hl("String").patch(hl("CursorLine")), format!(" {t} "))) }
            }
            Event::SoftBreak => chunks.push((Style::default(), " ".into())),
            Event::HardBreak => chunks.push((Style::default(), "\n".into())),
            Event::End(TagEnd::Image) => {
                if let Some((url, alt)) = image.take() {
                    let p = if url.starts_with('/') { PathBuf::from(&url) } else { base.join(&url) };
                    if p.is_file() && !url.contains("://") {
                        // a picture takes its own rows (drawn over them in zash); else its words in its place
                        flush(&mut rows, &mut chunks, width, quote, &lists, &mut task, &mut item_start, src);
                        // the picture's rows (empty: drawn over in zash), its words under it as the caption
                        let n = 10;
                        rows.push(Row { src: here, hint: Some(Hint::Image { path: p, rows: n }), ..Default::default() });
                        for _ in 1..n {
                            rows.push(Row { src: here, ..Default::default() });
                        }
                        rows.push(Row { spans: vec![(hl("Comment").add_modifier(Modifier::ITALIC), format!("🖼 {}", if alt.is_empty() { url.clone() } else { alt }))], src: here, ..Default::default() });
                    } else {
                        chunks.push((hl("Comment"), format!("🖼 {}", if alt.is_empty() { url } else { alt })));
                    }
                }
            }
            Event::Rule => {
                blank(&mut rows, here);
                rows.push(Row { spans: vec![(hl("Comment"), "─".repeat(width.min(120)))], src: here, ..Default::default() });
            }
            Event::End(TagEnd::Paragraph) | Event::End(TagEnd::Item) => {
                flush(&mut rows, &mut chunks, width, quote, &lists, &mut task, &mut item_start, src);
                if matches!(ev, Event::End(TagEnd::Item)) {
                    if let Some(Some(n)) = lists.last_mut() { *n += 1 }
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                let lv = heading.take().unwrap_or(1);
                let text: String = chunks.iter().map(|(_, s)| s.as_str()).collect();
                let style = hl("Title").add_modifier(Modifier::BOLD);
                // h1 and h2: two rows (drawn larger by zash), the rest one
                let n = if lv <= 2 { 2 } else { 1 };
                let prefix = if lv >= 4 { "#".repeat(lv as usize) + " " } else { String::new() };
                rows.push(Row { spans: vec![(style, format!("{prefix}{text}"))], src, hint: Some(Hint::Heading { level: lv, text: text.clone(), rows: n }), task: None });
                for _ in 1..n {
                    rows.push(Row { src, ..Default::default() });
                }
                chunks.clear();
            }
            Event::End(TagEnd::BlockQuote(_)) => quote = quote.saturating_sub(1),
            Event::End(TagEnd::List(_)) => {
                lists.pop();
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((lang, lines, first)) = code_block.take() {
                    let coloured = code(&lang, &lines);
                    let n = lines.len().max(1);
                    for i in 0..n {
                        let mut spans = vec![(Style::default(), "  ".to_string())];
                        spans.extend(coloured.get(i).cloned().unwrap_or_default());
                        rows.push(Row { spans, src: first + i, hint: (i == 0).then(|| Hint::Code { rows: n }), task: None });
                    }
                }
            }
            Event::End(TagEnd::Table) => {
                if let Some((t, at)) = table.take() {
                    let ncols = t.iter().map(|r| r.len()).max().unwrap_or(0);
                    let mut cols: Vec<usize> = (0..ncols).map(|c| t.iter().map(|r| r.get(c).map(|s| s.chars().count()).unwrap_or(0)).max().unwrap_or(0) + 3).collect();
                    // the widest columns give way when it is too wide
                    while cols.iter().sum::<usize>() > width.saturating_sub(2) && cols.iter().any(|c| *c > 6) {
                        let m = cols.iter().enumerate().max_by_key(|(_, c)| **c).map(|(i, _)| i).unwrap_or(0);
                        cols[m] -= 1;
                    }
                    let head = t.first().cloned().unwrap_or_default();
                    for (ri, r) in t.iter().enumerate() {
                        let mut spans = Vec::new();
                        for (ci, w) in cols.iter().enumerate() {
                            let s: String = r.get(ci).map(String::as_str).unwrap_or("").chars().take(w - 1).collect();
                            let st = if ri == 0 { hl("Title").add_modifier(Modifier::BOLD) } else { Style::default() };
                            spans.push((st, format!("{s:<w$}", w = *w)));
                        }
                        rows.push(Row { spans, src: at + ri + usize::from(ri > 0), hint: (ri == 0).then(|| Hint::Table { rows: t.len(), cols: cols.clone(), head: head.clone() }), task: None });
                    }
                }
            }
            _ => {}
        }
    }
    flush(&mut rows, &mut chunks, width, quote, &lists, &mut task, &mut item_start, src);
    // a row of air above (a large first heading is not cut at the window's top)
    rows.insert(0, Row::default());
    rows
}

/// The paragraph (or item) gathered so far as wrapped rows: a bullet, number or checkbox before an item's first.
#[allow(clippy::too_many_arguments)]
fn flush(rows: &mut Vec<Row>, chunks: &mut Vec<(Style, String)>, width: usize, quote: usize, lists: &[Option<u64>], task: &mut Option<(bool, usize)>, item_start: &mut bool, src: usize) {
    if chunks.is_empty() {
        return;
    }
    let depth = lists.len();
    let indent = "  ".repeat(depth.saturating_sub(1));
    let bar: Vec<(Style, String)> = (0..quote).map(|_| (hl("Comment"), "│ ".to_string())).collect();
    let mut first = bar.clone();
    let mut rest = bar;
    let mut task_line = None;
    if depth > 0 {
        let marker = if *item_start {
            if let Some((done, line)) = task.take() {
                task_line = Some(line);
                if done { (hl("Comment"), format!("{indent}☑ ")) } else { (hl("Special"), format!("{indent}☐ ")) }
            } else {
                match lists.last().copied().flatten() {
                    Some(n) => (hl("Special"), format!("{indent}{n}. ")),
                    None => (hl("Special"), format!("{indent}{} ", ["•", "◦", "▪"][(depth - 1) % 3])),
                }
            }
        } else {
            (Style::default(), format!("{indent}  "))
        };
        let pad = " ".repeat(marker.1.chars().count());
        first.push(marker);
        rest.push((Style::default(), pad));
    }
    *item_start = false;
    // a ticked task reads as done
    let mut body = std::mem::take(chunks);
    if task_line.is_some() && first.iter().any(|(_, s)| s.contains('☑')) {
        for (st, _) in body.iter_mut() { *st = st.patch(hl("Comment")).add_modifier(Modifier::CROSSED_OUT) }
    }
    // hard breaks: rows of their own
    let mut parts: Vec<Vec<(Style, String)>> = vec![Vec::new()];
    for (st, s) in body {
        for (i, piece) in s.split('\n').enumerate() {
            if i > 0 { parts.push(Vec::new()) }
            if !piece.is_empty() { parts.last_mut().unwrap().push((st, piece.to_string())) }
        }
    }
    let mut n = 0;
    for (pi, p) in parts.iter().enumerate() {
        for (li, l) in wrap(p, width, if pi == 0 { &first } else { &rest }, &rest).into_iter().enumerate() {
            rows.push(Row { spans: l, src: src + pi, hint: None, task: (n == 0 && li == 0).then_some(task_line).flatten() });
            n += 1;
        }
    }
    if depth == 0 {
        rows.push(Row { src, ..Default::default() });
    }
}

/// Percent-encoding for a UI hint's value.
fn pct(s: &str) -> String {
    s.chars().map(|c| match c { ',' => "%2C".into(), ';' => "%3B".into(), '%' => "%25".into(), '\x07' | '\x1b' => String::new(), c => c.to_string() }).collect()
}

impl App {
    /// :MarkdownView: this buffer rendered, or its source again.
    pub(crate) fn md_toggle(&mut self) {
        let name = self.docs[self.cur].name.clone();
        if self.md.remove(&name).is_some() {
            return self.say("the Markdown source");
        }
        let line = self.docs[self.cur].ed.cursor().line;
        self.md.insert(name, State { top: 0, sel: 0, cache: None });
        // the row the cursor's line renders to, chosen and in view (found when the rows exist: md_rows)
        self.md_goto_src = Some(line);
        self.say("Markdown view: j/k move · x ticks a task · e or Enter back to the source");
    }

    /// The rendered rows of buffer `di` at `width` (kept until the text or the width changes).
    pub(crate) fn md_rows(&mut self, di: usize, width: u16) -> Vec<Row> {
        let name = self.docs[di].name.clone();
        let rev = self.docs[di].ed.buf.rev();
        if let Some(Some((r, w, rows))) = self.md.get(&name).map(|s| s.cache.as_ref()) {
            if *r == rev && *w == width {
                return rows.clone();
            }
        }
        let text = self.docs[di].ed.text();
        let base = self.docs[di].abs_path().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_else(|| self.root.clone());
        let (ss, theme) = (&self.ss, &self.theme);
        let hlr = Highlighter::new(theme);
        let code = |lang: &str, lines: &[String]| {
            let syn = ss.find_syntax_by_token(lang).map(|s| s.name.clone()).unwrap_or_else(|| "Plain Text".into());
            crate::code_spans(ss, &hlr, &syn, lines)
        };
        let rows = render(&text, width as usize, &base, &code);
        if let Some(s) = self.md.get_mut(&name) {
            s.cache = Some((rev, width, rows.clone()));
            if let Some(l) = self.md_goto_src.take() {
                let at = rows.iter().rposition(|r| r.src <= l && !r.spans.is_empty()).unwrap_or(0);
                s.sel = at;
                s.top = at.saturating_sub(3);
            }
        }
        rows
    }

    /// Draws the view over a window's text area (`a`: without its status line).
    pub(crate) fn md_draw(&mut self, f: &mut Frame, di: usize, a: Rect, focused: bool) {
        let rows = self.md_rows(di, a.width.saturating_sub(2));
        let name = self.docs[di].name.clone();
        let Some(s) = self.md.get_mut(&name) else { return };
        s.sel = s.sel.min(rows.len().saturating_sub(1));
        // the chosen row stays in view
        if s.sel < s.top { s.top = s.sel }
        if s.sel >= s.top + a.height as usize { s.top = s.sel + 1 - a.height as usize }
        let (top, sel) = (s.top, s.sel);
        f.render_widget(ratatui::widgets::Clear, a);
        let lines: Vec<Line> = rows.iter().skip(top).take(a.height as usize).enumerate().map(|(i, r)| {
            let mut spans: Vec<Span> = vec![Span::raw(" ")];
            spans.extend(r.spans.iter().map(|(st, t)| Span::styled(t.clone(), *st)));
            let l = Line::from(spans);
            if focused && top + i == sel { l.style(hl("CursorLine")) } else { l }
        }).collect();
        f.render_widget(Paragraph::new(lines), a);
        // what zash draws over it: headings larger, code panels, table cards, pictures (sent after the frame)
        let mut regions = Vec::new();
        let mut pics = Vec::new();
        for (i, r) in rows.iter().enumerate().skip(top).take(a.height as usize) {
            let y = a.y as usize + i - top;
            let x = a.x as usize + 1;
            let room = (a.y + a.height) as usize - y;
            match &r.hint {
                Some(Hint::Heading { level, text, rows: n }) => {
                    let scale = match level { 1 => 1.9, 2 => 1.5, 3 => 1.15, _ => 1.0 };
                    let h = (*n).min(room);
                    regions.push(format!("text,{x},{y},{},{h},text={},scale={scale},weight={},font=ui,role=primary", a.width.saturating_sub(2), pct(text), if *level <= 2 { 700 } else { 600 }));
                }
                Some(Hint::Code { rows: n }) => regions.push(format!("panel,{x},{y},{},{},role=surface-container-high,pad=0.4", a.width.saturating_sub(3), (*n).min(room))),
                Some(Hint::Table { rows: n, cols, head }) => {
                    let w: usize = cols.iter().sum();
                    regions.push(format!("table,{x},{y},{w},{},cols={}", (*n).min(room), cols.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("|")));
                    let _ = head;
                }
                Some(Hint::Image { .. }) | None => {}
            }
        }
        // pictures whose rows reach into the window, the top ones too: the part in view (cut by zash, x,y,w,h)
        let end = top + a.height as usize;
        for (i, r) in rows.iter().enumerate().take(end).skip(top.saturating_sub(64)) {
            let Some(Hint::Image { path, rows: n }) = &r.hint else { continue };
            let (from, to) = (i.max(top), (i + n).min(end));
            if from >= to {
                continue;
            }
            pics.push(Pic { path: path.clone(), x: a.x + 3, y: a.y + (from - top) as u16, w: a.width.saturating_sub(6), n: *n, skip: from - i, vis: to - from });
        }
        self.md_hints.extend(regions);
        self.md_pics.extend(pics);
    }

    /// The view's keys; false: not one of them.
    pub(crate) fn md_key(&mut self, k: &KeyEvent) -> bool {
        let name = self.docs[self.cur].name.clone();
        let Some(rows) = self.md.get(&name).and_then(|s| s.cache.as_ref()).map(|c| c.2.clone()) else { return false };
        let page = self.area.height.max(2) as usize;
        let s = self.md.get_mut(&name).unwrap();
        let last = rows.len().saturating_sub(1);
        match k.code {
            KeyCode::Char('j') | KeyCode::Down => s.sel = (s.sel + 1).min(last),
            KeyCode::Char('k') | KeyCode::Up => s.sel = s.sel.saturating_sub(1),
            KeyCode::PageDown | KeyCode::Char(' ') => { s.sel = (s.sel + page).min(last); s.top = (s.top + page).min(last) }
            KeyCode::PageUp | KeyCode::Backspace => { s.sel = s.sel.saturating_sub(page); s.top = s.top.saturating_sub(page) }
            KeyCode::Char('g') | KeyCode::Home => { s.sel = 0; s.top = 0 }
            KeyCode::Char('G') | KeyCode::End => s.sel = last,
            // the next / previous heading
            KeyCode::Char('}') | KeyCode::Char(']') => s.sel = rows.iter().enumerate().skip(s.sel + 1).find(|(_, r)| matches!(r.hint, Some(Hint::Heading { .. }))).map(|(i, _)| i).unwrap_or(s.sel),
            KeyCode::Char('{') | KeyCode::Char('[') => s.sel = rows.iter().enumerate().take(s.sel).rev().find(|(_, r)| matches!(r.hint, Some(Hint::Heading { .. }))).map(|(i, _)| i).unwrap_or(s.sel),
            KeyCode::Char('x') => {
                // a task's box ticked or unticked, in the source
                let Some(line) = rows.get(s.sel).and_then(|r| r.task) else { self.say("not a task"); return true };
                let ed = &mut self.docs[self.cur].ed;
                let text = ed.buf.line(line).to_string();
                let new = if let Some(i) = text.find("[ ]") { format!("{}[x]{}", &text[..i], &text[i + 3..]) } else if let Some(i) = text.find("[x]").or_else(|| text.find("[X]")) { format!("{}[ ]{}", &text[..i], &text[i + 3..]) } else { return true };
                let a = ed.buf.to_char(zett_core::Pos { line, col: 0 });
                let b = a + text.chars().count();
                ed.replace_text(a, b, &new);
                self.docs[self.cur].touched(line);
            }
            KeyCode::Char('e') | KeyCode::Char('q') | KeyCode::Enter | KeyCode::Esc => {
                let line = rows.get(s.sel).map(|r| r.src).unwrap_or(0);
                self.md.remove(&name);
                let ed = &mut self.docs[self.cur].ed;
                let line = line.min(ed.buf.len_lines().saturating_sub(1));
                ed.set_cursor(zett_core::Pos { line, col: 0 });
            }
            _ => return false,
        }
        true
    }

    /// After the frame (and its synchronized update): the hints and pictures of the views on screen, sent when changed.
    pub(crate) fn md_paint(&mut self) {
        if !crate::viewer::graphics_on() {
            self.md_hints.clear();
            self.md_pics.clear();
            return;
        }
        use std::io::Write;
        let ui = if self.md_hints.is_empty() { String::new() } else { format!("\x1b]7733;ui;{}\x07", self.md_hints.join(";")) };
        let mut out = String::new();
        if ui != self.md_sent.ui {
            out.push_str(if ui.is_empty() { "\x1b]7733;ui\x07" } else { &ui });
            self.md_sent.ui = ui;
        }
        let key = format!("{:?}", self.md_pics);
        if key != self.md_sent.imgs_key {
            // the placements go (the pictures stay sent), then each is placed again: its slice in view
            for id in self.md_sent.imgs.drain(..) {
                out.push_str(&format!("\x1b_Ga=d,d=i,i={id},q=2\x1b\\"));
            }
            let (cw, ch) = crate::viewer::cell_px();
            for p in &self.md_pics {
                // one id per file and width, sent once
                let id = 7720 + (p.path.to_string_lossy().bytes().fold(p.w as u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32)) % 4000);
                if !self.md_sent.sent.contains_key(&id) {
                    let Some((seq, pw, ph)) = crate::viewer::picture_transmit(&p.path, p.w, id) else { continue };
                    out.push_str(&seq);
                    self.md_sent.sent.insert(id, (pw, ph));
                }
                let (pw, ph) = self.md_sent.sent[&id];
                // its size at full: the shape kept, as wide as the window allows, at most its rows tall
                let (cols, full) = crate::viewer::fit(pw, ph, Rect { x: 0, y: 0, width: p.w, height: p.n as u16 }, cw, ch, 1.0);
                let full = full as usize;
                if p.skip >= full {
                    continue;
                }
                let vis = p.vis.min(full - p.skip);
                let (y_px, h_px) = ((p.skip * ph as usize / full) as u32, (vis * ph as usize / full).max(1) as u32);
                out.push_str(&format!("\x1b7\x1b[{};{}H\x1b_Ga=p,i={id},x=0,y={y_px},w={pw},h={h_px},c={cols},r={vis},C=1,q=2;\x1b\\\x1b8", p.y + 1, p.x + 1 + (p.w - cols) / 2));
                self.md_sent.imgs.push(id);
            }
            self.md_sent.imgs_key = key;
        }
        self.md_hints.clear();
        self.md_pics.clear();
        if !out.is_empty() {
            let _ = std::io::stdout().write_all(out.as_bytes());
            let _ = std::io::stdout().flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(md: &str) -> Vec<String> {
        render(md, 40, Path::new("/tmp"), &|_, l| l.iter().map(|s| vec![(Style::default(), s.clone())]).collect()).into_iter().map(|r| r.spans.iter().map(|(_, s)| s.as_str()).collect::<String>()).collect()
    }

    #[test]
    fn markers_go_and_structure_stays() {
        let rows = plain("# Title\n\nSome **bold** and `code` text.\n\n- one\n- [ ] todo\n- [x] done\n\n1. first\n2. second\n\n> quoted\n\n```rust\nfn x() {}\n```\n\n| a | b |\n|---|---|\n| 1 | 2 |\n");
        let all = rows.join("\n");
        assert!(rows[1].starts_with("Title"), "{all}");
        assert!(all.contains("Some bold and code text."), "{all}");
        assert!(!all.contains("**") && !all.contains("# "), "{all}");
        assert!(all.contains("• one") && all.contains("☐ todo") && all.contains("☑ done"), "{all}");
        assert!(all.contains("1. first") && all.contains("2. second"), "{all}");
        assert!(all.contains("│ quoted"), "{all}");
        assert!(all.contains("  fn x() {}"), "{all}");
        assert!(all.contains("a") && all.contains("1"), "{all}");
    }

    #[test]
    fn long_paragraphs_wrap_and_tasks_know_their_line() {
        let r = render("- [ ] a task\n\nword word word word word word word word word word word word\n", 20, Path::new("/tmp"), &|_, _| vec![]);
        let task = r.iter().find(|x| x.task.is_some()).unwrap();
        assert_eq!(task.task, Some(0));
        assert!(r.iter().filter(|x| x.spans.iter().any(|(_, s)| s == "word")).count() >= 3, "wrapped into rows");
    }
}

#[cfg(test)]
mod key_tests {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    // x on a task's row ticks it in the source; e goes back to the source at that line
    #[test]
    fn x_ticks_the_task_in_the_source_and_e_returns_there() {
        let dir = std::env::temp_dir().join(format!("zett-md-key-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("t.md");
        std::fs::write(&f, "# T\n\n- [ ] first\n- [ ] second\n").unwrap();
        let mut app = crate::App::new(vec![f.clone()]);
        app.md_toggle();
        let rows = app.md_rows(app.cur, 60);
        let at = rows.iter().position(|r| r.task == Some(3)).expect("the second task's row");
        let name = app.docs[app.cur].name.clone();
        app.md.get_mut(&name).unwrap().sel = at;
        let key = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
        app.key(key('x'));
        assert_eq!(app.docs[app.cur].ed.text(), "# T\n\n- [ ] first\n- [x] second\n");
        app.key(key('e'));
        assert!(!app.md.contains_key(&name), "back to the source");
        assert_eq!(app.docs[app.cur].ed.cursor().line, 3);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// A picture's part in view: where (cells), how wide, its rows in full, how many of them are above the window
/// (scrolled off) and how many show.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Pic {
    pub path: PathBuf,
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub n: usize,
    pub skip: usize,
    pub vis: usize,
}
