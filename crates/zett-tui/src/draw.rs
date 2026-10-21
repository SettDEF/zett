//! Drawing: the buffers, their text and line numbers, the tab line, the : line, the which-key menu.

use super::*;
use crate::display::{ends_sentence, fold_cells, gut, spell_marks, Signs, TAB_FILL, TAB_FIRST, TAB_LAST};

impl App {
    // ── drawing ──

    pub(crate) fn draw(&mut self, f: &mut Frame) {
        // a highlight group changed (zett.hl, :hi): new styles, and the syntax colours again
        if hl::sync(&self.engine) {
            self.theme = palette_theme();
            for d in self.docs.iter_mut() {
                d.states.clear();
            }
        }
        let cur = self.cur.min(self.docs.len() - 1);
        // showtabline: 0 never, 2 always, 1 (Neovim's default) as zett's tabs setting says
        let tabs = match self.setting(cur, "showtabline").as_str() {
            "0" => false,
            "2" => true,
            _ => self.tabs && self.docs.len() > 1,
        };
        // cmdheight: the rows under the windows (0: only while there is something to show)
        let busy = self.cmdline.is_some() || self.msg.is_some() || self.minibuf.is_some() || self.docs[cur].ed.isearch().is_some() || !self.keyseq.is_empty();
        let cmd_rows = match self.num(cur, "cmdheight") {
            0 if busy => 1,
            n => n.min(10) as u16,
        };
        // laststatus 3: one status line for the focused window, under all of them
        let laststatus = self.num(cur, "laststatus");
        let global = u16::from(laststatus == 3);
        // the minibuffer's candidates take rows from the windows, as in Emacs
        let mini = self.minibuf.as_ref().filter(|_| self.placement == Placement::Bottom).map_or(0, |m| m.rows() as u16);
        let [tabline, body, gline, list, cmd] = Layout::vertical([
            Constraint::Length(if tabs { 1 } else { 0 }),
            Constraint::Min(2),
            Constraint::Length(global),
            Constraint::Length(mini),
            Constraint::Length(cmd_rows),
        ])
        .areas(f.area());
        if tabs {
            // tabline: its own format, else zett's buffer tabs
            let fmt = self.setting(cur, "tabline");
            if fmt.is_empty() {
                self.draw_tabs(f, tabline);
            } else {
                let p = self.fmt_line(&fmt, cur, body.height as usize, tabline.width as usize);
                crate::statusline::draw_pieces(f, tabline, p, hl("TabLineFill"));
            }
        }
        // title: the terminal's title from titlestring (zett's own when empty)
        self.title_want = self.flag(cur, "title").then(|| {
            let fmt = self.setting(cur, "titlestring");
            let fmt = if fmt.is_empty() { "%t%( %M%) - zett".to_string() } else { fmt };
            let len = self.num(cur, "titlelen");
            let width = if len == 0 { 200 } else { (f.area().width as usize * len / 100).max(10) };
            crate::statusline::plain(&self.fmt_line(&fmt, cur, body.height as usize, width)).trim_end().to_string()
        });
        if self.branch_at.elapsed() > Duration::from_secs(3) {
            self.branch = doom::git_branch(&self.root);
            self.branch_at = Instant::now();
        }
        // the focused window shows the current buffer
        self.cur = self.cur.min(self.docs.len() - 1);
        if let Some(w) = self.wins.get_mut(self.focus).and_then(|w| w.as_mut()) {
            w.doc = self.cur;
        }
        // plugin panels take the sides
        let body = self.draw_panels(f, body);
        let mut placed = Vec::new();
        let mut seps = Vec::new();
        self.tree.place(body, &mut placed, &mut seps);
        self.win_rects = placed.clone();
        // fillchars vert: the character between windows
        let vert = crate::display::fillchars(&self.setting(self.cur.min(self.docs.len() - 1), "fillchars")).vert.to_string();
        for s in seps {
            let bar: Vec<Line> = (0..s.height).map(|_| Line::from(Span::styled(vert.clone(), hl("WinSeparator")))).collect();
            f.render_widget(Paragraph::new(bar), s);
        }
        // laststatus: 0 no status lines, 1 only with two windows or more, 2 each window (3: the one above)
        let n_wins = placed.len();
        let status = match laststatus {
            0 | 3 => false,
            1 => n_wins > 1,
            _ => true,
        };
        for (id, r) in placed {
            let Some(w) = self.wins.get(id).and_then(|w| w.clone()) else { continue };
            let focused = id == self.focus;
            let di = w.doc.min(self.docs.len() - 1);
            // winbar: a format line at the window's top
            let bar_fmt = self.setting(di, "winbar");
            let wb = u16::from(!bar_fmt.is_empty() && r.height > 2);
            let text = Rect { y: r.y + wb, height: r.height.saturating_sub(u16::from(status) + wb), ..r };
            let line = Rect { y: r.bottom().saturating_sub(1), height: 1, ..r };
            if focused && self.dash && di == self.cur && self.docs[di].name == "[scratch]" && !self.docs[di].dirty() {
                self.area = text;
                self.draw_dashboard(f, text);
            } else {
                if focused {
                    self.dash = false;
                }
                // a window keeps its own scroll position, even on a buffer shown twice
                self.docs[di].top = w.top;
                self.docs[di].left = w.left;
                // time travel: the buffer as it was, drawn in its place
                let travelling = focused && self.travel.as_ref().is_some_and(|t| t.doc == di);
                if travelling {
                    std::mem::swap(&mut self.docs[di], &mut self.travel.as_mut().unwrap().preview);
                }
                self.draw_text(f, text, di, focused);
                if travelling {
                    std::mem::swap(&mut self.docs[di], &mut self.travel.as_mut().unwrap().preview);
                }
                if let Some(Some(w)) = self.wins.get_mut(id) {
                    w.top = self.docs[di].top;
                    w.left = self.docs[di].left;
                }
            }
            if wb > 0 {
                let p = self.fmt_line(&bar_fmt, di, text.height as usize, r.width as usize);
                crate::statusline::draw_pieces(f, Rect { height: 1, ..r }, p, hl(if focused { "WinBar" } else { "WinBarNC" }));
            }
            if status {
                self.draw_modeline(f, line, di, focused);
            }
        }
        if global > 0 {
            self.draw_modeline(f, gline, cur, true);
        }
        self.draw_float_panels(f, body);
        self.draw_floats(f, body);
        let (line_at, boxed, below) = self.cmd_place(body, cmd);
        self.draw_cmd(f, cmd, line_at, boxed);
        // ruler: where the cursor is, at the right of the last line, when no status line says it
        if !status && global == 0 && self.cmdline.is_none() && self.minibuf.is_none() && cmd.height > 0 && self.flag(cur, "ruler") {
            let fmt = self.setting(cur, "rulerformat");
            let fmt = if fmt.is_empty() { "%l,%c%V%=%P".to_string() } else { fmt };
            let w = 18.min(cmd.width);
            let at = Rect { x: cmd.right() - w, y: cmd.y, width: w, height: 1 };
            let p = self.fmt_line(&fmt, cur, body.height as usize, w as usize);
            crate::statusline::draw_pieces(f, at, p, hl("MsgArea"));
        }
        self.draw_suggestions(f, line_at, boxed, below);
        if !self.menu.is_empty() {
            self.draw_menu(f, body);
        }
        if self.comp.is_some() {
            self.draw_comp(f);
        }
        if self.minibuf.is_some() {
            self.draw_minibuf(f, list, cmd);
        }
        if let Some(Popup::Help(title, rows)) = &self.popup {
            draw_help(f, title, rows);
        }
    }

    /// The editor settings a config can set (zett.opt.tabs = true, number, relativenumber), as :set does.
    pub(crate) fn take_opts(&mut self) {
        let on = |v: Option<String>| v.map(|v| v == "true");
        if let Some(b) = on(self.engine.opt("tabs")) {
            self.tabs = b;
        }
        if let Some(b) = on(self.engine.opt("number")) {
            self.numbers = b;
        }
        if let Some(b) = on(self.engine.opt("relativenumber")) {
            self.relative = b;
        }
        self.emacs = self.engine.opt("keys").is_none_or(|k| k != "vim");
        self.placement = Placement::from_opt(self.engine.opt("minibuffer"), self.emacs);
    }

    pub(crate) fn draw_tabs(&self, f: &mut Frame, a: Rect) {
        let mut spans = Vec::new();
        for (i, d) in self.docs.iter().enumerate() {
            let name = Path::new(&d.name).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or(d.name.clone());
            // numbered, as Alt+1…9 and :b N count them
            let label = format!(" {} {name}{} ", i + 1, if d.dirty() { " ●" } else { "" });
            let st = if i == self.cur { hl("TabLineSel") } else { hl("TabLine") };
            spans.push(Span::styled(label, st));
            spans.push(Span::raw(" "));
        }
        f.render_widget(Paragraph::new(Line::from(spans)), a);
    }

    pub(crate) fn draw_text(&mut self, f: &mut Frame, a: Rect, di: usize, focused: bool) {
        // H M L and gm need the lines and columns shown
        let top = self.docs[di].top;
        self.docs[di].ed.set_screen(top, a.height as usize, a.width as usize);
        if focused {
            self.area = a;
        }
        let (ss, theme) = (&self.ss, &self.theme);
        let hl = Highlighter::new(theme);
        let (numbers, relative) = (self.numbers, self.relative);
        // this buffer's display settings (settings.rs)
        let (wrap, words, cul, scrolloff, list) = (self.flag(di, "wrap"), self.flag(di, "linebreak"), self.flag(di, "cursorline"), self.num(di, "scrolloff"), self.flag(di, "list"));
        let colorcols: Vec<usize> = self.setting(di, "colorcolumn").split(',').filter_map(|s| s.trim().parse::<usize>().ok()).filter(|&c| c > 0).collect();
        SHOW.with(|s| s.set((self.num(di, "tabstop").max(1), list)));
        let show = self.display(di);
        // spell: only when on and a dictionary is there (the first use loads it)
        let spell_on = self.flag(di, "spell") && zett_edit::spell::with(|s| s.available());
        let camel = self.setting(di, "spelloptions").split(',').any(|o| o == "camel");
        let capcheck = !self.setting(di, "spellcapcheck").is_empty();
        let lc = show.list.clone().unwrap_or_default();
        // showmatch: in insert mode, right after a closing bracket, its pair stands out
        let paren = (self.docs[di].ed.mode() == zett_edit::Mode::Insert && self.flag(di, "showmatch")).then(|| self.docs[di].ed.match_before_cursor()).flatten();
        let (queries, embeds, nb, root) = (&mut self.queries, &mut self.embeds, &self.notebook, self.root.clone());
        let d = &mut self.docs[di];
        let lines = d.ed.buf.len_lines();
        // live query blocks: their results under the closing fence (Markdown only)
        let qrows = if d.syntax == "Markdown" && lines < 20_000 {
            let text: Vec<String> = (0..lines).map(|i| d.ed.buf.line(i).trim_end_matches('\n').to_string()).collect();
            let file = std::fs::canonicalize(&d.name).ok();
            let root = file.as_deref().and_then(doom::project_of).unwrap_or(root);
            // and the block references' text, and what links here (refs.rs)
            let mut rows = query::rows(queries, &root, &text);
            rows.extend(refs::rows(embeds, queries, &root, file.as_deref(), &text));
            // and what the code blocks printed (notebook.rs)
            rows.extend(nb.rows(&d.name, &text));
            rows
        } else {
            Vec::new()
        };
        // the gutter: fold column, sign column, line numbers (signs in the numbers by default, zett's)
        let digits = lines.to_string().len();
        let sign_w = match show.signs {
            Signs::Yes => 2,
            Signs::Auto if !d.diags.is_empty() => 2,
            _ => 0,
        };
        let num_w = match (numbers, show.signs) {
            (true, Signs::Number) => (digits + 2).max(show.numberwidth),
            (true, _) => (digits + 1).max(show.numberwidth),
            (false, Signs::Number) => 1,
            (false, _) => 0,
        };
        let gutter = (show.foldcolumn + sign_w + num_w) as u16;
        if focused {
            self.gutter = gutter;
        }
        let width = a.width.saturating_sub(gutter) as usize;
        // H M L gm gj gk: the lines shown and the text's width, without the gutter
        d.ed.set_screen(top, a.height as usize, width);
        d.ed.set_wrap(wrap);
        let height = a.height as usize;
        // keep the cursor in view (a few lines of margin, like scrolloff), counting shown lines:
        // folded and narrowed-away ones take no rows
        d.ed.refresh_view();
        let c = d.ed.cursor();
        let margin = scrolloff.min(height.saturating_sub(1) / 2);
        d.top = d.top.min(lines.saturating_sub(1));
        while d.top + 1 < lines && d.ed.is_hidden(d.top) {
            d.top += 1;
        }
        // the rows a line takes: one, or its wrapped rows
        // a wrapped line's later rows start with showbreak, and with breakindent at the line's indent
        let sbr: String = show.showbreak.clone();
        let sbr_w = sbr.chars().count();
        let cont_of = |t: &str| (sbr_w + if show.breakindent { screen_col(t, t.chars().take_while(|c| *c == ' ' || *c == '\t').count()) } else { 0 }).min(width / 2);
        let rows_of = |d: &Doc, l: usize| {
            if wrap {
                let t = d.ed.buf.line(l);
                wrap_starts(&t, width, words, cont_of(&t)).len()
            } else {
                1
            }
        };
        let cur_line_text = d.ed.buf.line(c.line);
        let ccol = screen_col(&cur_line_text, c.col);
        // which wrapped row of its line the cursor is on, and its column there
        let cur_cont = cont_of(&cur_line_text);
        let starts = if wrap { wrap_starts(&cur_line_text, width, words, cur_cont) } else { vec![0] };
        let crow = starts.iter().rposition(|&s| s <= ccol).unwrap_or(0);
        let was = d.top;
        let above = d.ed.step_shown(c.line, margin, true);
        if above < d.top {
            d.top = above;
        } else {
            // the rows from the top to the cursor: lines (wrapped ones count each row), a
            // suggestion's lines and a block's results; the top moves down until the cursor fits
            loop {
                let extra: usize = d.ed.suggestions().iter().filter(|h| h.end() > d.top && h.end() <= c.line).map(|h| h.new.len()).sum::<usize>()
                    + qrows.iter().filter(|(after, _)| *after >= d.top && *after < c.line).map(|(_, r)| r.len()).sum::<usize>();
                let rows: usize = (d.top..c.line).filter(|&l| !d.ed.is_hidden(l)).take(height * 2).map(|l| rows_of(d, l)).sum::<usize>() + extra + crow;
                if rows + margin < height || d.top >= c.line {
                    break;
                }
                let next = d.ed.step_shown(d.top, 1, false);
                if next <= d.top {
                    break;
                }
                d.top = next;
            }
        }
        // scrolljump: once the cursor has left the window, at least this many lines at a time
        // (negative: a percentage of the height)
        let sj = if show.scrolljump < 0 { height * show.scrolljump.unsigned_abs() as usize / 100 } else { show.scrolljump as usize };
        if sj > 1 {
            if d.top > was && d.top - was < sj {
                d.top = (was + sj).min(c.line.saturating_sub(margin)).max(d.top);
            } else if d.top < was && was - d.top < sj {
                d.top = was.saturating_sub(sj).max((c.line + margin + 1).saturating_sub(height)).min(d.top);
            }
        }
        if wrap {
            d.left = 0;
        } else {
            // sidescrolloff: columns kept beside the cursor; sidescroll: at least that many at a time
            // (0: the cursor goes to the middle)
            let w = width.max(1);
            let off = show.sidescrolloff.min((w - 1) / 2);
            if ccol < d.left + off {
                d.left = match show.sidescroll {
                    0 => ccol.saturating_sub(w / 2),
                    s => ccol.saturating_sub(off).min(d.left.saturating_sub(s)),
                };
            } else if ccol + off >= d.left + w {
                d.left = match show.sidescroll {
                    0 => ccol.saturating_sub(w / 2),
                    s => (ccol + off + 1 - w).max(d.left + s),
                };
            }
        }
        let sel = d.ed.selection_range();
        let mut out: Vec<Line> = Vec::with_capacity(height);
        // a search in progress: its visible matches, found once per frame (Search; IncSearch for the
        // one the cursor is on)
        let (found, current) = match d.ed.isearch() {
            Some(s) if !s.query().is_empty() => {
                let current = s.current();
                let from = d.ed.buf.to_char(Pos { line: d.top.min(lines), col: 0 });
                let end = if d.ed.view_is_empty() { (d.top + height).min(lines) } else { lines };
                let to = if end >= lines { d.ed.buf.len_chars() } else { d.ed.buf.to_char(Pos { line: end, col: 0 }) };
                (d.ed.isearch_matches(from, to), current)
            }
            // hlsearch: the last search's matches stay marked (until :nohlsearch)
            _ if show.hlsearch => {
                let from = d.ed.buf.to_char(Pos { line: d.top.min(lines), col: 0 });
                let end = (d.top + height * 2).min(lines);
                let to = if end >= lines { d.ed.buf.len_chars() } else { d.ed.buf.to_char(Pos { line: end, col: 0 }) };
                (d.ed.hl_matches(from, to), None)
            }
            _ => (Vec::new(), None),
        };
        let mut next = d.top;
        let mut cursor_row = 0;
        // the agent's suggestions: what they replace is struck through, their lines drawn after it
        let hunks: Vec<zett_edit::suggest::Hunk> = d.ed.suggestions().to_vec();
        let mut virt: std::collections::VecDeque<Line> = std::collections::VecDeque::new();
        let mut queued_for = usize::MAX;
        for row in 0..height {
            if let Some(l) = virt.pop_front() {
                out.push(l);
                continue;
            }
            while next < lines && d.ed.is_hidden(next) {
                next += 1;
            }
            if queued_for != next {
                queued_for = next;
                for h in hunks.iter().filter(|h| h.end() == next) {
                    for (i, t) in h.new.iter().enumerate() {
                        let shown: String = t.replace('\t', &" ".repeat(tab())).chars().skip(d.left).take(width).collect();
                        let mut spans = vec![Span::styled(gut("+ ", gutter as usize), hl::hl("AgentAdd")), Span::styled(shown, hl::hl("AgentAdd"))];
                        if i == 0 {
                            spans.push(Span::styled(format!(" {} ", h.author), hl::hl("AgentTag")));
                        }
                        virt.push_back(Line::from(spans));
                    }
                }
                // a query block's results, after its closing fence
                for (_, rows) in qrows.iter().filter(|(after, _)| after + 1 == next && !d.ed.is_hidden(*after)) {
                    for r in rows {
                        let mut spans = vec![Span::raw(" ".repeat(gutter as usize))];
                        let mut room = width;
                        for (t, g) in r {
                            let t: String = t.chars().take(room).collect();
                            room = room.saturating_sub(t.chars().count());
                            spans.push(Span::styled(t, hl::hl(g)));
                        }
                        virt.push_back(Line::from(spans));
                    }
                }
                if let Some(l) = virt.pop_front() {
                    out.push(l);
                    continue;
                }
            }
            let ln = next;
            next += 1;
            if ln == c.line {
                // on a wrapped line: the row of it the cursor is on
                cursor_row = row + crow;
            }
            if ln >= lines {
                out.push(Line::from(if gutter > 0 { Span::styled(gut(&format!("{} ", show.fill.eob), gutter as usize), hl::hl("EndOfBuffer")) } else { Span::styled(show.fill.eob.to_string(), hl::hl("EndOfBuffer")) }));
                continue;
            }
            // the cursor line's number stands out with cursorline (cursorlineopt number or both)
            let num_style = if ln == c.line && cul && show.cul_number { hl::hl("CursorLineNr") } else { hl::hl("LineNr") };
            let shown = if relative && ln != c.line { ln.abs_diff(c.line) } else { ln + 1 };
            let line_text = d.ed.buf.line(ln);
            // diagnostics: a sign in the gutter, the ranges underlined (in screen columns)
            let sign = d.diags.iter().filter(|x| x.line == ln).map(|x| x.severity).min();
            let marks: Vec<(usize, usize, u8)> = d
                .diags
                .iter()
                .filter(|x| x.line <= ln && x.end_line >= ln)
                .map(|x| {
                    let s = if x.line == ln { screen_col(&line_text, x.col) } else { 0 };
                    let e = if x.end_line == ln { screen_col(&line_text, x.end_col) } else { usize::MAX };
                    (s, e.max(s + 1), x.severity)
                })
                .collect();
            // the gutter: fold column, sign column, number (gn spans before the text)
            let mut spans: Vec<Span<'static>> = Vec::new();
            if show.foldcolumn > 0 {
                let (depth, start) = d.ed.fold_mark(ln);
                spans.push(Span::styled(fold_cells(show.foldcolumn, depth, start, &show.fill), hl::hl("FoldColumn")));
            }
            if sign_w > 0 {
                spans.push(match sign {
                    Some(s) => Span::styled("● ", Style::default().fg(lsp::severity_color(s))),
                    None => Span::styled("  ", hl::hl("SignColumn")),
                });
            }
            if numbers {
                let num = gut(&format!("{shown} "), num_w);
                spans.push(match sign.filter(|_| show.signs == Signs::Number) {
                    Some(s) => Span::styled(format!("●{}", &num[num.char_indices().nth(1).map(|(i, _)| i).unwrap_or(num.len())..]), num_style.fg(lsp::severity_color(s))),
                    None => Span::styled(num, num_style),
                });
            } else if num_w > 0 {
                spans.push(match sign {
                    Some(s) => Span::styled("●", Style::default().fg(lsp::severity_color(s))),
                    None => Span::raw(" "),
                });
            }
            let gn = spans.len();
            // the selection on this line, in screen columns
            let sel_cols = sel.and_then(|(s, e)| {
                let ls = d.ed.buf.to_char(Pos { line: ln, col: 0 });
                let le = ls + line_text.chars().count();
                let (a, b) = (s.max(ls), e.min(le + 1));
                (a < b).then(|| (screen_col(&line_text, a - ls), screen_col(&line_text, b.min(le) - ls) + usize::from(b > le)))
            });
            // a block (CTRL-V): its columns on each of its lines
            let sel_cols = match d.ed.block() {
                Some((l1, l2, c1, c2)) if ln >= l1 && ln <= l2 => {
                    let n = line_text.chars().count();
                    let (a, b) = (c1.min(n), c2.saturating_add(1).min(n.max(c1 + 1)));
                    (b > a || c2 == usize::MAX).then(|| (screen_col(&line_text, a), screen_col(&line_text, b.min(n)) + usize::from(b > n)))
                }
                Some(_) => None,
                None => sel_cols,
            };
            // search matches on this line, in screen columns (true: the current one)
            let hits: Vec<(usize, usize, bool)> = if found.is_empty() {
                Vec::new()
            } else {
                let ls = d.ed.buf.to_char(Pos { line: ln, col: 0 });
                let le = ls + line_text.chars().count();
                found
                    .iter()
                    .filter(|(a, b)| *a < le && *b > ls)
                    .map(|&(a, b)| (screen_col(&line_text, a.max(ls) - ls), screen_col(&line_text, b.min(le) - ls), current == Some((a, b))))
                    .collect()
            };
            let mut x = 0usize;
            // a line a suggestion would replace
            let replaced = hunks.iter().find(|h| h.at <= ln && ln < h.end());
            // list: where leading blanks end and trailing ones begin (screen columns)
            let body = line_text.trim_end_matches('\n');
            let trail = if list { screen_col(&line_text, body.trim_end_matches([' ', '\t']).chars().count()) } else { usize::MAX };
            let lead = if list { screen_col(&line_text, body.chars().take_while(|c| *c == ' ' || *c == '\t').count()) } else { 0 };
            // showmatch: the bracket on this line that the one just typed pairs with
            let paren_col = paren.and_then(|p| {
                let ls = d.ed.buf.to_char(Pos { line: ln, col: 0 });
                (p >= ls && p < ls + body.chars().count()).then(|| screen_col(&line_text, p - ls))
            });
            let cursorcol = show.cursorcolumn && focused && ln != c.line;
            // spell: misspelled words (SpellBad), a sentence starting with a small letter (SpellCap)
            let (bad, cap) = if spell_on { spell_marks(&line_text, camel, capcheck, ln > 0 && ends_sentence(&d.ed.buf.line(ln - 1))) } else { (Vec::new(), Vec::new()) };
            for (st, text) in d.spans(ln, ss, &hl) {
                for ch in text.chars() {
                    if wrap || (x >= d.left && x < d.left + width) {
                        // synmaxcol: no syntax colours past it
                        let mut s = if x >= show.synmaxcol { Style::default() } else { st };
                        // listchars: tabs, trailing / leading / all spaces, non-breaking ones
                        let (ch, blank) = match ch {
                            TAB_FIRST => (lc.tab.map_or(' ', |t| t.0), true),
                            TAB_FILL => (lc.tab.map_or(' ', |t| t.1), true),
                            TAB_LAST => (lc.tab.map_or(' ', |t| t.2.unwrap_or(t.1)), true),
                            ' ' if x >= trail && lc.trail.is_some() => (lc.trail.unwrap(), true),
                            ' ' if x < lead && lc.lead.is_some() => (lc.lead.unwrap(), true),
                            ' ' if list && lc.space.is_some() => (lc.space.unwrap(), true),
                            '\u{a0}' | '\u{202f}' if list && lc.nbsp.is_some() => (lc.nbsp.unwrap(), true),
                            c => (c, false),
                        };
                        if blank {
                            s = s.patch(hl::hl("Whitespace"));
                        }
                        if colorcols.contains(&(x + 1)) {
                            s = s.patch(hl::hl("ColorColumn"));
                        }
                        if cursorcol && x == ccol {
                            s = s.patch(hl::hl("CursorColumn"));
                        }
                        if bad.iter().any(|&(a, b)| x >= a && x < b) {
                            s = s.add_modifier(Modifier::UNDERLINED).underline_color(Color::Indexed(1));
                        } else if cap.iter().any(|&(a, b)| x >= a && x < b) {
                            s = s.add_modifier(Modifier::UNDERLINED).underline_color(Color::Indexed(4));
                        }
                        if paren_col == Some(x) {
                            s = s.patch(hl::hl("MatchParen"));
                        }
                        if replaced.is_some() {
                            s = s.patch(hl::hl("AgentDelete"));
                        }
                        if sel_cols.is_some_and(|(a, b)| x >= a && x < b) {
                            s = s.add_modifier(Modifier::REVERSED);
                        }
                        if let Some(h) = hits.iter().find(|h| x >= h.0 && x < h.1) {
                            s = s.patch(hl::hl(if h.2 { "IncSearch" } else { "Search" }));
                        }
                        if let Some(&(_, _, sev)) = marks.iter().filter(|m| x >= m.0 && x < m.1).min_by_key(|m| m.2) {
                            s = s.add_modifier(Modifier::UNDERLINED).underline_color(lsp::severity_color(sev));
                        }
                        spans.push(Span::styled(ch.to_string(), s));
                    }
                    x += 1;
                }
            }
            // listchars precedes / extends: text off the window's left / right edge (nowrap)
            if !wrap && spans.len() > gn {
                if let Some(p) = lc.precedes.filter(|_| d.left > 0) {
                    spans[gn] = Span::styled(p.to_string(), hl::hl("NonText"));
                }
                if let Some(e) = lc.extends.filter(|_| x > d.left + width) {
                    let last = spans.len() - 1;
                    spans[last] = Span::styled(e.to_string(), hl::hl("NonText"));
                }
            }
            // the line end: listchars eol, or one cell for a selection over the line break
            let at_end = sel_cols.is_some_and(|(a, b)| x >= a && x < b);
            let end_shown = wrap || (x >= d.left && x < d.left + width);
            if let Some(e) = lc.eol.filter(|_| end_shown) {
                let s = if at_end { hl::hl("NonText").add_modifier(Modifier::REVERSED) } else { hl::hl("NonText") };
                spans.push(Span::styled(e.to_string(), s));
                x += 1;
            } else if at_end && end_shown {
                spans.push(Span::styled(" ", Style::default().add_modifier(Modifier::REVERSED)));
            }
            let cells = spans.len() - gn;
            // colorcolumn and cursorcolumn past the line's end: the marked columns still show
            if !wrap {
                let mut cols: Vec<(usize, &str)> = colorcols.iter().map(|&cc| (cc - 1, "ColorColumn")).collect();
                if cursorcol {
                    cols.push((ccol, "CursorColumn"));
                }
                cols.sort();
                cols.dedup_by_key(|c| c.0);
                let start = x.max(d.left);
                let mut at = start;
                for (col, g) in cols.into_iter().filter(|&(col, _)| col >= start && col < d.left + width) {
                    spans.push(Span::raw(" ".repeat(col - at)));
                    spans.push(Span::styled(" ", hl::hl(g)));
                    at = col + 1;
                }
            }
            // a suggestion that only deletes: its tag on the first line it would delete
            if let Some(h) = replaced.filter(|h| h.new.is_empty() && h.at == ln) {
                spans.push(Span::styled(format!(" {}: delete ", h.author), hl::hl("AgentTag")));
            }
            // a folded section: what it hides, after its heading
            if d.ed.is_fold_start(ln) {
                spans.push(Span::styled(" …", hl::hl("Folded")));
            }
            // the current line stands out, and fades over the lines around it (CursorLine, CursorLineGlow…)
            // a section just found pulses: strong, softer, then the glow, over a second and a half
            let pulse = self.flash.filter(|f| f.0 == di && ln >= f.1 && ln < f.2).map(|f| f.3.elapsed().as_millis()).and_then(|ms| match ms {
                0..=350 => Some("IncSearch"),
                351..=900 => Some("Visual"),
                901..=1500 => Some("CursorLineGlow"),
                _ => None,
            });
            let band = match ln.abs_diff(c.line) {
                _ if pulse.is_some() => pulse,
                _ if !focused || !cul || !show.cul_line => None,
                0 => Some("CursorLine"),
                1 => Some("CursorLineGlow"),
                2 => Some("CursorLineGlow2"),
                _ => None,
            };
            let style = band.map(hl::hl).filter(|s| s.bg.is_some());
            let finish = |mut spans: Vec<Span<'static>>| {
                match style {
                    Some(s) => {
                        let used: usize = spans.iter().map(|s| s.width()).sum();
                        spans.push(Span::raw(" ".repeat((a.width as usize).saturating_sub(used))));
                        Line::from(spans).style(s)
                    }
                    None => Line::from(spans),
                }
            };
            // wrap: the line's cells cut into rows; the first goes here, the rest are drawn next
            let cont = cont_of(&line_text);
            let wstarts = if wrap { wrap_starts(&line_text, width, words, cont) } else { vec![0] };
            if wstarts.len() > 1 {
                let mut rest = spans.split_off(gn);
                let extras = rest.split_off(cells.min(rest.len()));
                let mut chunks: Vec<Vec<Span<'static>>> = Vec::new();
                for (k, &s0) in wstarts.iter().enumerate().rev() {
                    let part = rest.split_off(s0.min(rest.len()));
                    chunks.push(part);
                    if k == 0 {
                        break;
                    }
                }
                chunks.reverse();
                let n = chunks.len();
                for (k, mut part) in chunks.into_iter().enumerate() {
                    if k + 1 == n {
                        part.extend(extras.iter().cloned());
                    }
                    if k == 0 {
                        spans.extend(part);
                        out.push(finish(std::mem::take(&mut spans)));
                    } else {
                        let mut row = vec![Span::raw(" ".repeat(gutter as usize))];
                        // showbreak, then the breakindent
                        let mark: String = sbr.chars().take(cont).collect();
                        let n = mark.chars().count();
                        row.push(Span::styled(mark, hl::hl("NonText")));
                        row.push(Span::raw(" ".repeat(cont - n)));
                        row.extend(part);
                        virt.push_back(finish(row));
                    }
                }
            } else {
                out.push(finish(spans));
            }
        }
        f.render_widget(Paragraph::new(out), a);
        if focused && self.cmdline.is_none() && self.minibuf.is_none() && self.popup.is_none() {
            let x = a.x + gutter + (ccol.saturating_sub(d.left + starts[crow]) + if crow > 0 { cur_cont } else { 0 }) as u16;
            let y = a.y + cursor_row as u16;
            self.cursor_xy = (x, y);
            f.set_cursor_position(Position { x: x.min(a.right().saturating_sub(1)), y });
        }
    }

    /// The bottom row (`a`): messages, keys, errors; and the `:` line at `line_at` (in a box when `boxed`).
    pub(crate) fn draw_cmd(&mut self, f: &mut Frame, a: Rect, line_at: Rect, boxed: bool) {
        if self.travel.is_some() {
            return self.draw_travel(f, a);
        }
        if let Some(l) = &self.cmdline {
            let p = self.prompt.clone().unwrap_or_else(|| ":".into());
            if boxed {
                // a floating line: a rounded box, its prompt as the title
                let outer = Rect { x: line_at.x - 1, y: line_at.y - 1, width: line_at.width + 2, height: 3 };
                f.render_widget(Clear, outer);
                let title = if p == ":" { " Command ".to_string() } else { format!(" {} ", p.trim_end_matches([':', ' '])) };
                f.render_widget(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(hl("FloatBorder")).title(Span::styled(title, hl("Prompt"))), outer);
            } else if line_at != a {
                f.render_widget(Clear, line_at);
            }
            let shown_p = if boxed && p == ":" { "› ".to_string() } else { p.clone() };
            // the end of a long line stays in view
            let room = (line_at.width as usize).saturating_sub(shown_p.chars().count() + 1);
            let skip = l.chars().count().saturating_sub(room);
            let tail: String = l.chars().skip(skip).collect();
            f.render_widget(Paragraph::new(Line::from(vec![Span::styled(shown_p.clone(), hl("Prompt")), Span::raw(tail.clone())])), line_at);
            f.set_cursor_position(Position { x: line_at.x + (shown_p.chars().count() + tail.chars().count()) as u16, y: line_at.y });
            if line_at == a {
                return;
            }
        }
        // a search in progress: "I-search: query", Emacs's echo area
        if let Some(s) = self.docs.get(self.cur).and_then(|d| d.ed.isearch()) {
            let p = s.prompt();
            let (label, query) = p.split_once(": ").unwrap_or((p.as_str(), ""));
            let style = if label.starts_with("Failing") { hl("Invalid") } else { hl("Prompt") };
            f.render_widget(Paragraph::new(Line::from(vec![Span::styled(format!("{label}: "), style), Span::raw(query.to_string())])), a);
            return;
        }
        if let Some(s) = self.agent_status() {
            f.render_widget(Paragraph::new(s).style(hl("MsgArea")), a);
            return;
        }
        if let Some((m, at)) = &self.msg {
            if at.elapsed() < Duration::from_secs(6) {
                f.render_widget(Paragraph::new(m.as_str()).style(hl("MsgArea")), a);
                return;
            }
            self.msg = None;
        }
        // showmode: -- INSERT -- and the like, and the macro being recorded (Vim's last line)
        if let Some(d) = self.docs.get(self.cur).filter(|d| !d.ed.is_emacs() && self.flag(self.cur, "showmode")) {
            let mode = match d.ed.mode() {
                Mode::Insert if d.ed.replacing() => "-- REPLACE --",
                Mode::Insert => "-- INSERT --",
                Mode::Visual => "-- VISUAL --",
                Mode::VisualLine => "-- VISUAL LINE --",
                Mode::VisualBlock => "-- VISUAL BLOCK --",
                Mode::Normal => "",
            };
            let rec = d.ed.recording().map(|r| format!("recording @{r}")).unwrap_or_default();
            if !mode.is_empty() || !rec.is_empty() {
                let gap = if mode.is_empty() || rec.is_empty() { "" } else { " " };
                f.render_widget(Paragraph::new(format!("{mode}{gap}{rec}")).style(hl("ModeMsg")), a);
                return;
            }
        }
        if !self.keyseq.is_empty() {
            let shown: Vec<String> = self.keyseq.iter().map(|k| if k == "<Space>" { "SPC".into() } else { k.clone() }).collect();
            f.render_widget(Paragraph::new(shown.join(" ")).style(hl("KeySeq")), a);
            return;
        }
        // the error on this line (as Doom shows it), else what a language server is doing
        if let Some((sev, m)) = self.line_diag() {
            f.render_widget(Paragraph::new(m).style(Style::default().fg(lsp::severity_color(sev))), a);
        } else if !self.lsp_status.is_empty() {
            f.render_widget(Paragraph::new(self.lsp_status.as_str()).style(hl("StatusLineDim")), a);
        }
    }

    pub(crate) fn draw_menu(&self, f: &mut Frame, body: Rect) {
        let n = self.menu.len();
        let cols = (body.width as usize / 28).clamp(1, 4);
        let rows = n.div_ceil(cols);
        let h = (rows as u16 + 2).min(body.height);
        let a = Rect { x: body.x, y: body.bottom().saturating_sub(h), width: body.width, height: h };
        f.render_widget(Clear, a);
        let title: Vec<String> = self.keyseq.iter().map(|k| if k == "<Space>" { "SPC".into() } else { k.clone() }).collect();
        let block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(hl("FloatBorderDim")).title(format!(" {} … ", title.join(" ")));
        let inner = block.inner(a);
        f.render_widget(block, a);
        let colw = inner.width as usize / cols;
        let mut lines: Vec<Line> = Vec::new();
        for r in 0..rows {
            let mut spans = Vec::new();
            for c in 0..cols {
                let Some(m) = self.menu.get(c * rows + r) else { continue };
                let key = if m.key == "<Space>" { "SPC".to_string() } else { m.key.clone() };
                let desc = if m.group { format!("+{}", m.desc) } else { m.desc.clone() };
                let cell = format!(" {key:>3}  {desc}");
                let cell: String = cell.chars().take(colw.saturating_sub(1)).collect();
                let pad = colw.saturating_sub(cell.chars().count());
                spans.push(Span::styled(format!(" {key:>3} "), hl("WhichKeyKey")));
                let d: String = format!(" {desc}").chars().take(colw.saturating_sub(6)).collect();
                let dpad = colw.saturating_sub(5 + d.chars().count());
                spans.push(Span::styled(d, if m.group { hl("WhichKeyGroup") } else { Style::default() }));
                spans.push(Span::raw(" ".repeat(dpad.min(pad + 5))));
            }
            lines.push(Line::from(spans));
        }
        f.render_widget(Paragraph::new(lines), inner);
    }

}
