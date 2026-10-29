//! Drawing: the buffers, their text and line numbers, the tab line, the : line, the which-key menu.

thread_local! {
    /// each window's height in the last frame (splitkeep: a resize keeps the text in place)
    static WIN_H: std::cell::RefCell<std::collections::HashMap<usize, usize>> = std::cell::RefCell::new(std::collections::HashMap::new());
}

/// A split: the new window starts with the height its parent had (so it counts as resized too).
pub(crate) fn note_split(from: usize, new: usize) {
    WIN_H.with(|m| { let mut m = m.borrow_mut(); if let Some(h) = m.get(&from).copied() { m.insert(new, h); } });
}

use super::*;
use crate::display::{ends_sentence, fold_cells, gut, spell_marks, Signs, TAB_FILL, TAB_FIRST, TAB_LAST};

impl App {
    // ── drawing ──

    pub(crate) fn draw(&mut self, f: &mut Frame) {
        // the screen's real size as Neovim's columns / lines (plugins place their windows by them: fidget's bottom
        // right was column 80 of a wide terminal, in the middle); set only when it changes
        let a = f.area();
        if self.engine.opt("columns").as_deref() != Some(&a.width.to_string()) { let _ = self.engine.set_opt("columns", &a.width.to_string(), 'i'); }
        if self.engine.opt("lines").as_deref() != Some(&a.height.to_string()) { let _ = self.engine.set_opt("lines", &a.height.to_string(), 'i'); }
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
            _ => (self.tabs && self.docs.len() > 1) || self.tab_count() > 1,
        };
        // cmdheight: the rows under the windows (0: only while there is something to show)
        let busy = self.cmdline.is_some() || self.msg.is_some() || self.minibuf.is_some() || self.docs[cur].ed.isearch().is_some() || !self.keyseq.is_empty() || self.agent.is_some();
        // laststatus 3: one status line for the focused window, under all of them
        let laststatus = self.num(cur, "laststatus");
        // with 0 and a status line at the bottom, the : line and messages take that line's place
        // (no row is added, the windows never move); without one, a row comes while busy
        let over_status = self.num(cur, "cmdheight") == 0 && laststatus != 0;
        let cmd_rows = match self.num(cur, "cmdheight") {
            0 if busy && !over_status => 1,
            n => n.min(10) as u16,
        };
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
        let cmd = if over_status && busy {
            let at = if global > 0 { gline } else { Rect { y: body.bottom().saturating_sub(1), height: 1, ..body } };
            Rect { y: at.y, height: 1, ..f.area() }
        } else {
            cmd
        };
        if tabs {
            // tabline: its own format, else zett's buffer tabs
            let fmt = self.setting(cur, "tabline");
            if fmt.is_empty() && self.tab_count() > 1 {
                // tab pages, from two on
                self.draw_tab_pages(f, tabline);
            } else if fmt.is_empty() {
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
        // winheight / winwidth: the focused window at least that many lines / columns (placed again if it grew)
        if let Some(&(_, r)) = placed.iter().find(|(i, _)| *i == self.focus) {
            let (wh, ww) = (self.num(cur, "winheight") + 1, self.num(cur, "winwidth"));
            let mut grew = false;
            if (r.height as usize) < wh && placed.len() > 1 {
                grew |= self.tree.resize(self.focus, false, (wh - r.height as usize) as i32, body);
            }
            if (r.width as usize) < ww && placed.len() > 1 {
                grew |= self.tree.resize(self.focus, true, (ww - r.width as usize) as i32, body);
            }
            if grew {
                placed.clear();
                seps.clear();
                self.tree.place(body, &mut placed, &mut seps);
                // winminheight / winminwidth: no other window smaller than that; what is missing goes back
                let (mh, mw) = (self.num(cur, "winminheight") + 1, self.num(cur, "winminwidth").max(1));
                for step in 1..=4i32 {
                let short_h = placed.iter().filter(|(i, _)| *i != self.focus).map(|(_, x)| mh.saturating_sub(x.height as usize)).max().unwrap_or(0) as i32;
                let short_w = placed.iter().filter(|(i, _)| *i != self.focus).map(|(_, x)| mw.saturating_sub(x.width as usize)).max().unwrap_or(0) as i32;
                // rounding can swallow a small step: each try gives back more
                let (short_h, short_w) = (if short_h > 0 { short_h + step - 1 } else { 0 }, if short_w > 0 { short_w + step - 1 } else { 0 });
                if short_h == 0 && short_w == 0 {
                    break;
                }
                {
                    if short_h > 0 {
                        self.tree.resize(self.focus, false, -short_h, body);
                    }
                    if short_w > 0 {
                        self.tree.resize(self.focus, true, -short_w, body);
                    }
                    placed.clear();
                    seps.clear();
                    self.tree.place(body, &mut placed, &mut seps);
                }
                }
            }
        }
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
        // the focused window first: scrollbind and cursorbind windows follow it in the same frame
        let mut placed = placed;
        placed.sort_by_key(|(id, _)| *id != self.focus);
        let mut follow: Option<(isize, isize, usize)> = None;
        // diff mode: the focused window's buffer, top line and cursor line; the others follow through the diff
        let mut follow_diff: Option<(usize, usize, usize)> = None;
        for (id, r) in placed {
            let Some(mut w) = self.wins.get(id).and_then(|w| w.clone()) else { continue };
            let focused = id == self.focus;
            let di = w.doc.min(self.docs.len() - 1);
            let in_diff = w.diff && !focused;
            // another window on this buffer: its own cursor while it is drawn (put back after)
            let real = self.docs[di].ed.cursor_state();
            if !focused {
                if let Some(c) = w.cursor {
                    self.docs[di].ed.set_cursor(c);
                }
            }
            if let Some((fdoc, ftop, fline)) = follow_diff.filter(|_| in_diff) {
                let blocks = self.diff_blocks(fdoc, di);
                w.top = crate::diff::map_line(&blocks, ftop);
                let col = self.docs[di].ed.cursor().col;
                let l = crate::diff::map_line(&blocks, fline).min(self.docs[di].ed.buf.len_lines().saturating_sub(1));
                self.docs[di].ed.set_cursor(Pos { line: l, col });
            }
            if let Some((dv, dh, line)) = follow.filter(|_| !focused && !in_diff) {
                if self.flag(di, "scrollbind") {
                    let last = self.docs[di].ed.buf.len_lines().saturating_sub(1) as isize;
                    w.top = (w.top as isize + dv).clamp(0, last) as usize;
                    w.left = (w.left as isize + dh).max(0) as usize;
                    // its cursor comes into view with it, as in Vim
                    let c = self.docs[di].ed.cursor();
                    let h = (r.height as usize).saturating_sub(2);
                    let so = self.num(di, "scrolloff").min(h / 2);
                    let (lo, hi) = (if w.top == 0 { 0 } else { w.top + so }, (w.top + h).saturating_sub(so).max(w.top + so));
                    if c.line < lo || c.line > hi {
                        self.docs[di].ed.set_cursor(Pos { line: c.line.clamp(lo, hi), col: c.col });
                    }
                }
                if self.flag(di, "cursorbind") && di != self.cur {
                    let col = self.docs[di].ed.cursor().col;
                    let l = line.min(self.docs[di].ed.buf.len_lines().saturating_sub(1));
                    self.docs[di].ed.set_cursor(Pos { line: l, col });
                }
            }
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
                self.fold_texts(di);
                // statuscolumn for the lines this window may show (the view can still move a little)
                let (t, h) = (self.docs[di].top, text.height as usize);
                self.stc_lines(di, t.saturating_sub(h), t + 2 * h);
                self.ext_marks(di, t.saturating_sub(h), t + 2 * h);
                self.drawing_win = id;
                let winhl = self.setting(di, "winhighlight");
                crate::hl::set_winhighlight(&winhl);
                self.draw_text(f, text, di, focused);
                // Normal read as another group: the window takes its colours where nothing else set
                // them (Neovim fills a window with Normal)
                if winhl.split(',').any(|p| p.trim_start().starts_with("Normal:")) {
                    let n = hl::hl("Normal");
                    let buf = f.buffer_mut();
                    for y in text.top()..text.bottom() {
                        for x in text.left()..text.right() {
                            let c = &mut buf[(x, y)];
                            if let Some(bg) = n.bg.filter(|_| c.bg == Color::Reset) {
                                c.set_bg(bg);
                            }
                            if let Some(fg) = n.fg.filter(|_| c.fg == Color::Reset) {
                                c.set_fg(fg);
                            }
                        }
                    }
                }
                crate::hl::set_winhighlight("");
                if travelling {
                    std::mem::swap(&mut self.docs[di], &mut self.travel.as_mut().unwrap().preview);
                }
                if let Some(Some(w)) = self.wins.get_mut(id) {
                    w.top = self.docs[di].top;
                    w.left = self.docs[di].left;
                    w.cursor = Some(self.docs[di].ed.cursor());
                }
                if !focused {
                    self.docs[di].ed.set_cursor_state(real);
                }
                if focused && w.diff {
                    follow_diff = Some((di, self.docs[di].top, self.docs[di].ed.cursor().line));
                }
                // how far the focused window moved since the last frame (scrollopt: ver, hor)
                if focused && (self.flag(di, "scrollbind") || self.flag(di, "cursorbind")) {
                    let (top, left) = (self.docs[di].top, self.docs[di].left);
                    let so = self.setting(di, "scrollopt");
                    let (dv, dh) = match self.bind_prev {
                        Some((pid, pt, pl)) if pid == id => (top as isize - pt as isize, if so.contains("hor") { left as isize - pl as isize } else { 0 }),
                        _ => (0, 0),
                    };
                    let dv = if so.contains("ver") || so.is_empty() { dv } else { 0 };
                    follow = Some((dv, dh, self.docs[di].ed.cursor().line));
                    self.bind_prev = Some((id, top, left));
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
        // over the status line: that line cleared first, so none of it shows around the : line
        if over_status && busy {
            f.render_widget(ratatui::widgets::Clear, cmd);
            f.render_widget(ratatui::widgets::Block::default().style(hl("MsgArea")), cmd);
        }
        let (line_at, boxed, below) = self.cmd_place(body, cmd);
        self.draw_cmd(f, cmd, line_at, boxed);
        // showcmd: the keys of a command under way, at the right of the last line (showcmdloc=last)
        if self.flag(cur, "showcmd") && self.setting(cur, "showcmdloc") == "last" && self.cmdline.is_none() && cmd.height > 0 {
            let keys = self.docs[cur].ed.pending_keys();
            let ruler = !status && global == 0 && self.flag(cur, "ruler");
            let x = cmd.right().saturating_sub(11 + if ruler { 18 } else { 0 });
            if !keys.is_empty() && x > cmd.x {
                f.render_widget(Paragraph::new(keys).style(hl("MsgArea")), Rect { x, y: cmd.y, width: 10, height: 1 });
            }
        }
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
        // conceallevel / concealcursor: how extmarks' conceal shows (0: not at all)
        let (clevel, ccursor) = (self.num(di, "conceallevel"), self.setting(di, "concealcursor"));
        // H M L and gm need the lines and columns shown
        // a section just found: drawn by zash (a hand-drawn line under it) where it can, else it pulses
        let (flash, fx) = (self.flash, fx_draws());
        let mut fx_marks: Vec<FxMark> = Vec::new();
        // help pages: their headings drawn under (once, then kept), a found section circled
        let help = fx && self.docs[di].name.starts_with("help: ");
        let quiet = self.fx_quiet();
        // the headings (outside code blocks), as the outline reads them
        let heads: std::collections::HashSet<usize> = if help {
            let mut fence = false;
            (0..self.docs[di].ed.buf.len_lines())
                .filter(|&l| {
                    let t = self.docs[di].ed.buf.line(l);
                    if t.trim_start().starts_with("```") {
                        fence = !fence;
                        return false;
                    }
                    !fence && t.starts_with('#')
                })
                .collect()
        } else {
            Default::default()
        };
        // diff mode: filler lines and the changed lines' groups (diff.rs)
        let dview = self.diff_view(self.drawing_win, di);
        // the signs placed in it (:sign), by line
        let usigns = self.sign_rows(di);
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
        crate::set_vartabstop(&self.setting(di, "vartabstop"));
        let show = self.display(di);
        // :match: this window's pattern and its highlight group
        // :3match, :2match, :match in that order: where they meet, the lowest number is drawn last and wins
        let mpats: Vec<(Style, _)> = [3u8, 2, 1].iter().filter_map(|n| self.win_match.get(&(self.drawing_win, *n))).filter_map(|(g, p)| crate::ex::vim_regex(p, false).ok().map(|re| (hl::hl(g), re))).collect();
        // breakindentopt: shift:N more, min:N columns of text kept at least
        let briopt = self.setting(di, "breakindentopt");
        let bri_num = |k: &str, d: usize| briopt.split(',').find_map(|p| p.strip_prefix(k)?.parse::<isize>().ok()).unwrap_or(d as isize);
        let (bri_shift, bri_min) = (bri_num("shift:", 0), bri_num("min:", 20).max(1) as usize);
        let sbr_first = briopt.split(',').any(|p| p == "sbr");
        // linebreak: the characters a line may break after (breakat; ^I is a tab)
        let brk: Vec<char> = self.setting(di, "breakat").replace("^I", "\t").chars().chain([TAB_FIRST, TAB_FILL, TAB_LAST]).collect();
        let words = words.then_some(brk.as_slice());
        let incsearch = self.flag(di, "incsearch");
        // spell: only when on and a dictionary is there (the first use loads it)
        let spell_on = self.flag(di, "spell") && zett_edit::spell::with(|s| s.available());
        let camel = self.setting(di, "spelloptions").split(',').any(|o| o == "camel");
        let capcheck = !self.setting(di, "spellcapcheck").is_empty();
        let lc = show.list.clone().unwrap_or_default();
        // showmatch: in insert mode, right after a closing bracket, its pair stands out
        // for matchtime tenths of a second after the key
        let lasting = self.typed_at.elapsed() < Duration::from_millis(self.num(di, "matchtime") as u64 * 100);
        let paren = (lasting && self.docs[di].ed.mode() == zett_edit::Mode::Insert && self.flag(di, "showmatch")).then(|| self.docs[di].ed.match_before_cursor()).flatten();
        // a plugin's whole lines between the text (extmark virt_lines): (the line they stand before, rows)
        let vlines: Vec<(usize, Vec<Vec<(String, String)>>)> = self.extmarks.get(&di).map(|(_, v)| v.iter().filter(|m| !m.virt_lines.is_empty()).map(|m| (if m.virt_above { m.row } else { m.row + 1 }, m.virt_lines.clone())).collect()).unwrap_or_default();
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
            Signs::Auto if !d.diags.is_empty() || !usigns.is_empty() => 2,
            _ => 0,
        };
        let num_w = match (numbers, show.signs) {
            (true, Signs::Number) => (digits + 2).max(show.numberwidth),
            (true, _) => (digits + 1).max(show.numberwidth),
            (false, Signs::Number) => 1,
            (false, _) => 0,
        };
        // statuscolumn: as wide as it worked out to be
        let gutter = match self.stc.get(&di) {
            Some((_, w, _)) => *w as u16,
            None => (show.foldcolumn + sign_w + num_w) as u16,
        };
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
        let cont_of = |t: &str| (sbr_w + if show.breakindent { (screen_col(t, t.chars().take_while(|c| *c == ' ' || *c == '\t').count()) as isize + bri_shift).max(0) as usize } else { 0 }).min(width.saturating_sub(bri_min).max(sbr_w)).min(width / 2 + width / 4);
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
                    + qrows.iter().filter(|(after, _)| *after >= d.top && *after < c.line).map(|(_, r)| r.len()).sum::<usize>()
                    + vlines.iter().filter(|(at, _)| *at >= d.top && *at <= c.line).map(|(_, r)| r.len()).sum::<usize>()
                    + dview.as_ref().map_or(0, |v| v.fill.iter().filter(|(l, _)| **l > d.top && **l <= c.line).map(|(_, n)| *n).sum::<usize>());
                let rows: usize = (d.top..c.line).filter(|&l| !d.ed.is_hidden(l)).take(height * 2).map(|l| rows_of(d, l)).sum::<usize>() + extra + crow;
                if rows + margin < height || d.top >= c.line {
                    break;
                }
                // not past the margin above: in a short window with wrapped lines both margins cannot
                // hold, and the top went up and down on every other frame (flickered)
                let next = d.ed.step_shown(d.top, 1, false);
                if next <= d.top || next > above {
                    break;
                }
                d.top = next;
            }
        }
        // splitkeep screen / topline: when the window's height changed (a split, a close) the text stays
        // where it was and the cursor moves into view instead (cursor: the view follows the cursor)
        let resized = WIN_H.with(|m| m.borrow_mut().insert(self.drawing_win, height).is_some_and(|h| h != height));
        let spk = self.setting(di, "splitkeep");
        let dy = self.setting(di, "display");
        let d = &mut self.docs[di];
        if resized && d.top != was && (spk == "screen" || spk == "topline") {
            d.top = was;
            let lo = (was + margin).min(lines.saturating_sub(1));
            let hi = (was + height).saturating_sub(1 + margin).max(lo).min(lines.saturating_sub(1));
            let line = c.line.clamp(lo, hi);
            d.ed.set_cursor(zett_core::Pos { line, col: c.col.min(d.ed.buf.line_len(line).saturating_sub(1)) });
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
            Some(s) if !s.query().is_empty() && (d.ed.is_emacs() || incsearch) => {
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
                // diff mode: the lines only the other side has stand here as filler (fillchars diff)
                if let Some(n) = dview.as_ref().and_then(|v| v.fill.get(&next)) {
                    for _ in 0..*n {
                        virt.push_back(Line::from(vec![Span::raw(" ".repeat(gutter as usize)), Span::styled(show.fill.diff.to_string().repeat(width), hl::hl("DiffDelete"))]));
                    }
                }
                for h in hunks.iter().filter(|h| h.end() == next) {
                    for (i, t) in h.new.iter().enumerate() {
                        let shown: String = crate::expand(t).chars().skip(d.left).take(width).collect();
                        let mut spans = vec![Span::styled(gut("+ ", gutter as usize), hl::hl("AgentAdd")), Span::styled(shown, hl::hl("AgentAdd"))];
                        if i == 0 {
                            spans.push(Span::styled(format!(" {} ", h.author), hl::hl("AgentTag")));
                        }
                        virt.push_back(Line::from(spans));
                    }
                }
                for (_, rows) in vlines.iter().filter(|(at, _)| *at == next) {
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
            let shown_text = || -> String { d.ed.buf.line(ln).chars().filter(|c| *c != '\n').skip(d.left).take(a.width.saturating_sub(gutter) as usize).collect() };
            let flashed = flash.filter(|f| fx && f.0 == di && f.1 == ln);
            let head_key = format!("head:{}:{ln}", d.name);
            // a whole topic's title is drawn under by the flash: the same mark as its heading's
            // (two keys drew it twice, the second time with its own sound, when the flash ended)
            let title_flash = flashed.is_some_and(|fl| fl.1 == 0 && fl.2 == 1) && heads.contains(&ln) && d.left == 0;
            if let Some(fl) = flashed {
                // a whole topic's title: drawn under; a section jumped to: a loop round it
                let spec = if fl.1 == 0 && fl.2 == 1 { "underline;color=accent;draw=700ms;sound=draw;volume=0.4" } else { "scribble;color=accent;turns=1.15;wobble=0.08;draw=900ms;sound=draw;volume=0.45" };
                // a heading: its words, the # left out
                let t = shown_text();
                let skip = if heads.contains(&ln) && d.left == 0 { t.chars().take_while(|c| *c == '#' || *c == ' ').count() } else { 0 };
                let text: String = t.chars().skip(skip).collect::<String>().trim_end().to_string();
                let key = if title_flash { head_key.clone() } else { format!("found:{}:{ln}", d.name) };
                fx_marks.push(FxMark { x: a.x + gutter + skip as u16, y: a.y + out.len() as u16, text, key, since: fl.3, spec, transient: false });
            }
            // a heading's underline, a jumped-to one's too (with its loop, not 2 s after it)
            if help && d.left == 0 && heads.contains(&ln) && !title_flash {
                // a heading: the words drawn under, the # left out
                let t = shown_text();
                let hashes = t.chars().take_while(|c| *c == '#' || *c == ' ').count();
                let words: String = t.chars().skip(hashes).collect();
                if !words.trim().is_empty() {
                    fx_marks.push(FxMark { x: a.x + gutter + hashes as u16, y: a.y + out.len() as u16, text: words.trim_end().to_string(), key: head_key, since: Instant::now(), spec: "underline;color=accent;draw=600ms;sound=draw;volume=0.35", transient: false });
                }
            }
            // the cursor on a help page: a big loop round the `name` it is on, else round the heading it is on
            if help && fx && !quiet && d.left == 0 && ln == c.line {
                let t: Vec<char> = d.ed.buf.line(ln).chars().filter(|ch| *ch != '\n').collect();
                let around = crate::emacs::tick_span(&t, c.col).map(|(start, text)| (start, text)).or_else(|| {
                    heads.contains(&ln).then(|| {
                        let hashes = t.iter().take_while(|ch| **ch == '#' || **ch == ' ').count();
                        (hashes, t[hashes..].iter().collect::<String>().trim_end().to_string())
                    })
                });
                if let Some((start, text)) = around.filter(|(_, text)| !text.trim().is_empty()) {
                    let key = format!("cur:{}:{ln}:{start}", d.name);
                    fx_marks.push(FxMark { x: a.x + gutter + start as u16, y: a.y + out.len() as u16, text, key, since: Instant::now(), spec: crate::emacs::CIRCLE, transient: true });
                }
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
            // a placed sign (:sign) over a diagnostic when its priority is above 10, as Neovim
            let us = usigns.get(&ln).filter(|u| sign.is_none() || u.priority > 10);
            let num_style = match usigns.get(&ln).and_then(|u| u.numhl) {
                Some(h) => num_style.patch(h),
                None => num_style,
            };
            if sign_w > 0 {
                spans.push(match (us, sign) {
                    (Some(u), _) => Span::styled(u.text.clone(), u.texthl),
                    (None, Some(s)) => Span::styled("● ", Style::default().fg(lsp::severity_color(s))),
                    (None, None) => Span::styled("  ", hl::hl("SignColumn")),
                });
            }
            if numbers && show.signs == Signs::Number && us.is_some() {
                // signcolumn=number: the sign instead of the number, as Neovim draws it (no digits
                // left over: a 2-digit line lost its first one to the sign)
                let u = us.unwrap();
                let pad = num_w.saturating_sub(u.text.chars().count());
                spans.push(Span::styled(" ".repeat(pad.saturating_sub(1)), num_style));
                spans.push(Span::styled(u.text.clone(), u.texthl));
                spans.push(Span::styled(" ", num_style));
            } else if numbers {
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
            // statuscolumn: its pieces in place of the gutter (LineNr where they name no group)
            if let Some((w, m)) = self.stc.get(&di).map(|(_, w, m)| (*w, m)) {
                spans.clear();
                match m.get(&ln) {
                    Some(pieces) => spans.extend(pieces.iter().map(|(t, g)| Span::styled(t.clone(), g.as_ref().map_or(num_style, |g| num_style.patch(hl::hl(g)))))),
                    None => spans.push(Span::raw(" ".repeat(w))),
                }
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
            let mhits: Vec<Vec<(usize, usize)>> = mpats.iter().map(|(_, re)| re.find_iter(body).flatten().map(|m| (screen_col(&line_text, body[..m.start()].chars().count()), screen_col(&line_text, body[..m.end()].chars().count()))).collect()).collect();
            // extmarks' highlights on this line (a plugin's), in screen columns, lowest priority first
            let emarks: &[crate::extmarks::ExtMark] = self.extmarks.get(&di).map(|(_, v)| v.as_slice()).unwrap_or(&[]);
            let eranges: Vec<(usize, usize, Style)> = crate::extmarks::line_ranges(emarks, ln, &line_text).into_iter().map(|(a, b, g, _)| (screen_col(&line_text, a), screen_col(&line_text, b), hl::hl(&g))).collect();
            // concealed text (extmarks' conceal): the cursor's line shows it unless concealcursor has
            // the mode's letter
            let mode_letter = match d.ed.mode() {
                Mode::Insert => 'i',
                Mode::Visual | Mode::VisualLine | Mode::VisualBlock => 'v',
                _ => 'n',
            };
            let conceal_here = clevel > 0 && (ln != c.line || ccursor.contains(mode_letter));
            let cranges: Vec<(usize, usize, String)> = if conceal_here { crate::extmarks::conceal_ranges(emarks, ln, &line_text).into_iter().map(|(a, b, s)| (screen_col(&line_text, a), screen_col(&line_text, b), s)).collect() } else { Vec::new() };
            // diff mode: this line's group, and the part of it that differs (DiffText), in screen columns
            let dline = dview.as_ref().and_then(|v| v.lines.get(&ln)).copied();
            let dtext = dline.and_then(|(_, r)| r).map(|(p, q)| (screen_col(&line_text, p), screen_col(&line_text, q)));
            // spell: misspelled words (SpellBad), a sentence starting with a small letter (SpellCap)
            let (bad, cap) = if spell_on { spell_marks(&line_text, camel, capcheck, ln > 0 && ends_sentence(&d.ed.buf.line(ln - 1))) } else { (Vec::new(), Vec::new()) };
            for (st, text) in d.spans(ln, ss, &hl) {
                for ch in text.chars() {
                    // concealed: one character in its place (level 1: it or a blank, 2: it, 3: none)
                    if let Some(cr) = cranges.iter().find(|r| x >= r.0 && x < r.1) {
                        if x == cr.0 && (wrap || (x >= d.left && x < d.left + width)) {
                            let shown = match clevel {
                                1 if cr.2.is_empty() => " ".to_string(),
                                1 | 2 => cr.2.clone(),
                                _ => String::new(),
                            };
                            if !shown.is_empty() {
                                spans.push(Span::styled(shown, hl::hl("Conceal")));
                            }
                        }
                        x += 1;
                        continue;
                    }
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
                        if dtext.is_some_and(|(p, q)| x >= p && x < q) {
                            s = s.patch(hl::hl("DiffText"));
                        }
                        for ((ms, _), hits) in mpats.iter().zip(&mhits) {
                            if hits.iter().any(|&(a, b)| x >= a && x < b) {
                                s = s.patch(*ms);
                            }
                        }
                        for (a, b, es) in &eranges {
                            if x >= *a && x < *b {
                                s = s.patch(*es);
                            }
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
            // extmarks' virtual text: after the line (eol, overlay), or at the window's right edge
            for m in emarks.iter().filter(|m| m.row == ln && !m.virt.is_empty() && m.virt_pos != "inline") {
                let w: usize = m.virt.iter().map(|c| c.0.chars().count()).sum();
                if m.virt_pos == "right_align" {
                    let used: usize = spans[gn..].iter().map(|s| s.width()).sum();
                    spans.push(Span::raw(" ".repeat(width.saturating_sub(used + w))));
                } else {
                    spans.push(Span::raw(" "));
                }
                for (t, g) in &m.virt {
                    spans.push(Span::styled(t.clone(), if g.is_empty() { Style::default() } else { hl::hl(g) }));
                }
            }
            // a folded section: what it hides, after its heading; with a foldtext, its text in place
            // of the line, the rest of the row filled (fillchars fold:)
            if let Some(t) = self.fold_text.get(&di).and_then(|(_, m)| m.get(&ln)) {
                spans.truncate(gn);
                let n = Span::raw(t.as_str()).width();
                spans.push(Span::styled(t.clone(), hl::hl("Folded")));
                spans.push(Span::styled(show.fill.fold.to_string().repeat(width.saturating_sub(n)), hl::hl("Folded")));
            } else if d.ed.is_fold_start(ln) {
                spans.push(Span::styled(" …", hl::hl("Folded")));
            }
            // the current line stands out, and fades over the lines around it (CursorLine, CursorLineGlow…)
            // a section just found pulses: strong, softer, then the glow, over a second and a half
            let pulse = self.flash.filter(|f| !fx && f.0 == di && ln >= f.1 && ln < f.2).map(|f| f.3.elapsed().as_millis()).and_then(|ms| match ms {
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
            // a diff line's colour fills the row (the band's own look on top is only its modifiers)
            let style = match dline {
                Some((g, _)) => {
                    let mut s = hl::hl(g);
                    if let Some(b) = band.map(hl::hl) {
                        s.add_modifier |= b.add_modifier;
                    }
                    Some(s)
                }
                // a sign's linehl colours its whole line
                None => match usigns.get(&ln).and_then(|u| u.linehl).or_else(|| emarks.iter().filter(|m| !m.line_hl.is_empty() && m.row <= ln && ln <= m.end_row).max_by_key(|m| m.priority).map(|m| hl::hl(&m.line_hl))) {
                    Some(h) => Some(h),
                    None => band.map(hl::hl).filter(|s| s.bg.is_some()),
                },
            };
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
                        // the breakindent, then showbreak (breakindentopt sbr: showbreak first)
                        let mark: String = sbr.chars().take(cont).collect();
                        let n = mark.chars().count();
                        let (mark, pad) = (Span::styled(mark, hl::hl("NonText")), Span::raw(" ".repeat(cont - n)));
                        if sbr_first {
                            row.extend([mark, pad]);
                        } else {
                            row.extend([pad, mark]);
                        }
                        row.extend(part);
                        virt.push_back(finish(row));
                    }
                }
            } else {
                out.push(finish(spans));
            }
        }
        // display: the last line did not fit (wrapped rows left over): lastline puts @@@ at the end of
        // the last row, truncate the whole last row as @@@, neither shows "@" there (as Vim)
        if !virt.is_empty() && !out.is_empty() {
            let has = |f: &str| dy.split(',').any(|o| o == f);
            let at = hl::hl("NonText");
            let last = out.len() - 1;
            if has("truncate") {
                out[last] = Line::from(Span::styled("@@@", at));
            } else if has("lastline") {
                let w = a.width as usize;
                let text: String = out[last].spans.iter().map(|s| s.content.as_ref()).collect();
                let keep: String = text.chars().take(w.saturating_sub(3)).collect();
                let pad = w.saturating_sub(3).saturating_sub(keep.chars().count());
                out[last] = Line::from(vec![Span::raw(keep), Span::raw(" ".repeat(pad)), Span::styled("@@@", at)]);
            } else {
                out[last] = Line::from(Span::styled("@", at));
            }
        }
        f.render_widget(Paragraph::new(out), a);
        self.fx_marks.extend(fx_marks);
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
                crate::layer(f, outer);
                let title = if p == ":" { " Command ".to_string() } else { format!(" {} ", p.trim_end_matches([':', ' '])) };
                f.render_widget(Block::default().borders(Borders::ALL).border_set(crate::display::float_set()).border_style(hl("FloatBorder")).title(Span::styled(title, hl("Prompt"))), outer);
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
        crate::layer(f, a);
        let title: Vec<String> = self.keyseq.iter().map(|k| if k == "<Space>" { "SPC".into() } else { k.clone() }).collect();
        let block = Block::default().borders(Borders::ALL).border_set(crate::display::float_set()).border_style(hl("FloatBorderDim")).title(format!(" {} … ", title.join(" ")));
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

/// zash draws marks by hand (OSC 7733;fx) when zett runs right in it, or in a zeit that passes them
/// on to their cells (ZEIT_FX=1; an older zeit dropped them).
pub(crate) fn fx_draws() -> bool {
    static OK: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *OK.get_or_init(|| {
        let zash = std::env::var("TERM_PROGRAM").is_ok_and(|t| t == "zash");
        let zeit = std::env::var_os("ZEIT").is_some() && std::env::var_os("ZEIT_FX").is_none();
        let ok = zash && !zeit;
        if !ok {
            // said once, in the log: why help's found places pulse instead of being drawn
            let why = if !zash { "the terminal is not zash" } else { "inside a zeit that does not pass zash's effects on (no ZEIT_FX)" };
            let _ = std::fs::create_dir_all(crate::doom::state_dir());
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(crate::doom::state_dir().join("zett.log")) {
                use std::io::Write;
                let _ = writeln!(f, "help: found places pulse, not drawn: {why}");
            }
        }
        ok
    })
}

/// A mark for zash to draw over cells (OSC 7733;fx): where, the text it is drawn on, what it is
/// (kept by `key` across frames: drawn once, then shown drawn while it stays), when it began.
pub(crate) struct FxMark {
    pub x: u16,
    pub y: u16,
    pub text: String,
    pub key: String,
    pub since: Instant,
    pub spec: &'static str,
    /// there only while the mouse or the cursor is (plain cells back when it goes)
    pub transient: bool,
}

#[cfg(test)]
mod splitkeep_tests {
    fn after_split(spk: &str) -> (usize, usize) {
        let d = std::env::temp_dir().join(format!("zett-spk-{spk}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let f = d.join("a.txt");
        std::fs::write(&f, (1..=200).map(|i| format!("line {i}\n")).collect::<String>()).unwrap();
        let mut app = crate::App::new(Vec::new());
        app.open(&f);
        app.command(&format!("set splitkeep={spk} scrolloff=0"));
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 40)).unwrap();
        app.command("35");
        term.draw(|f| app.draw(f)).unwrap();
        app.command("split");
        term.draw(|f| app.draw(f)).unwrap();
        let _ = std::fs::remove_dir_all(&d);
        (app.docs[app.cur].top, app.docs[app.cur].ed.cursor().line)
    }
    #[test]
    fn splitkeep_topline_keeps_the_text_and_moves_the_cursor() {
        let (top, line) = after_split("topline");
        assert_eq!(top, 0, "the top line stayed");
        assert!(line < 20, "the cursor came into the smaller window: {line}");
        let (top, line) = after_split("cursor");
        assert_eq!(line, 34, "cursor: the cursor stays");
        assert!(top > 0, "and the view scrolled to it");
    }
}

#[cfg(test)]
mod display_tests {
    fn last_row(dy: &str) -> String {
        let mut app = crate::App::new(Vec::new());
        let text = format!("{}{}\n", "a\n".repeat(7), "b".repeat(400));
        app.docs[app.cur].ed = zett_edit::Editor::from_str(&text);
        app.command(&format!("set display={dy} nonumber norelativenumber laststatus=0 showtabline=0"));
        let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(40, 12)).unwrap();
        term.draw(|f| app.draw(f)).unwrap();
        let b = term.backend().buffer();
        let rows: Vec<String> = (0..b.area.height).map(|y| (0..b.area.width).map(|x| b[(x, y)].symbol().to_string()).collect::<String>()).collect();
        rows.into_iter().filter(|r| r.contains('b') || r.contains('@')).last().unwrap_or_default()
    }
    #[test]
    fn display_marks_a_last_line_that_does_not_fit() {
        assert!(last_row("lastline").trim_end().ends_with("@@@"), "{:?}", last_row("lastline"));
        assert!(last_row("truncate").starts_with("@@@"), "{:?}", last_row("truncate"));
        assert_eq!(last_row("").trim_end(), "@");
    }
}
