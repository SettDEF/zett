//! Language servers in the editor: diagnostics, definitions, hover.

use super::*;
use zett_lsp::{Diag, Event, Location};

impl Doc {
    pub(crate) fn abs_path(&self) -> Option<PathBuf> {
        let p = self.ed.buf.path()?;
        Some(std::fs::canonicalize(p).unwrap_or(p.to_path_buf()))
    }

    pub(crate) fn text(&self) -> String {
        let b = &self.ed.buf;
        b.slice(0, usize::MAX)
    }
}

impl App {
    /// Tells the file's server it is open (starting the server when needed).
    pub(crate) fn lsp_open(&mut self, di: usize) {
        let Some(path) = self.docs[di].abs_path() else { return };
        let root = doom::project_of(&path).unwrap_or_else(|| path.parent().map(Path::to_path_buf).unwrap_or_default());
        let text = self.docs[di].text();
        let rev = self.docs[di].ed.buf.rev();
        if let Some((c, lang)) = self.lsp.for_file(&path, &root) {
            if !c.is_open(&path) {
                c.did_open(&path, lang, &text);
            }
            self.docs[di].lsp = true;
            self.docs[di].lsp_rev = rev;
        }
    }

    fn client_for(&mut self, di: usize) -> Option<&mut zett_lsp::Client> {
        let path = self.docs[di].abs_path()?;
        let root = doom::project_of(&path).unwrap_or_else(|| path.parent().map(Path::to_path_buf).unwrap_or_default());
        self.lsp.for_file(&path, &root).map(|(c, _)| c)
    }

    /// Each turn of the loop: what servers said, and text they have not seen yet.
    pub(crate) fn lsp_tick(&mut self) {
        for ev in self.lsp.events() {
            match ev {
                Event::Diagnostics(path, diags) => {
                    if let Some(d) = self.docs.iter_mut().find(|d| d.abs_path().as_deref() == Some(path.as_path())) {
                        d.diags = diags;
                    }
                }
                Event::Definition(locs) => match locs.first() {
                    Some(l) => self.goto_location(l.clone()),
                    None => self.say("no definition found"),
                },
                Event::Hover(text) => {
                    let rows: Vec<(String, String)> = text.lines().filter(|l| !l.starts_with("```")).map(|l| (String::new(), l.to_string())).collect();
                    if rows.iter().all(|r| r.1.trim().is_empty()) {
                        self.say("nothing to show here");
                    } else {
                        self.popup = Some(Popup::Help("Documentation".into(), rows));
                    }
                }
                Event::Status(s) => self.lsp_status = s,
                Event::Exited(name) => {
                    self.lsp.clients.retain(|_, c| c.name != name);
                    self.say(format!("{name} stopped"));
                }
            }
        }
        // text changed since the server last saw it, and the typing paused
        if self.edit_at.elapsed() < Duration::from_millis(250) {
            return;
        }
        for di in 0..self.docs.len() {
            let d = &self.docs[di];
            if !d.lsp || d.ed.buf.rev() == d.lsp_rev {
                continue;
            }
            let (Some(path), text, rev) = (d.abs_path(), d.text(), d.ed.buf.rev()) else { continue };
            if let Some(c) = self.client_for(di) {
                c.did_change(&path, &text);
            }
            self.docs[di].lsp_rev = rev;
        }
    }

    pub(crate) fn lsp_saved(&mut self, di: usize) {
        if !self.docs[di].lsp {
            return;
        }
        let Some(path) = self.docs[di].abs_path() else { return };
        // the server sees the saved text before the save itself
        let text = self.docs[di].text();
        let rev = self.docs[di].ed.buf.rev();
        if let Some(c) = self.client_for(di) {
            c.did_change(&path, &text);
            c.did_save(&path);
        }
        self.docs[di].lsp_rev = rev;
    }

    /// The cursor's position in the server's column units.
    fn lsp_pos(&mut self) -> Option<(PathBuf, usize, usize)> {
        let di = self.cur;
        let path = self.docs[di].abs_path()?;
        let c = self.docs[di].ed.cursor();
        let line = self.docs[di].ed.buf.line(c.line);
        let utf16 = self.client_for(di)?.utf16();
        Some((path, c.line, if utf16 { zett_lsp::to_utf16(&line, c.col) } else { c.col }))
    }

    pub(crate) fn lsp_definition(&mut self) {
        if !self.docs[self.cur].lsp {
            return self.say("no language server for this file");
        }
        // the server must see the text as it is now
        self.edit_at = Instant::now() - Duration::from_secs(1);
        self.lsp_tick();
        let Some((path, line, col)) = self.lsp_pos() else { return };
        if let Some(c) = self.client_for(self.cur) {
            c.definition(&path, line, col);
        }
    }

    pub(crate) fn lsp_hover(&mut self) {
        if !self.docs[self.cur].lsp {
            return self.say("no language server for this file");
        }
        self.edit_at = Instant::now() - Duration::from_secs(1);
        self.lsp_tick();
        let Some((path, line, col)) = self.lsp_pos() else { return };
        if let Some(c) = self.client_for(self.cur) {
            c.hover(&path, line, col);
        }
    }

    fn goto_location(&mut self, l: Location) {
        let utf16 = self.client_for(self.cur).map(|c| c.utf16()).unwrap_or(true);
        self.open(&l.path);
        let d = &self.docs[self.cur];
        let line = l.line.min(d.ed.buf.len_lines().saturating_sub(1));
        let text = d.ed.buf.line(line);
        let col = if utf16 { zett_lsp::from_utf16(&text, l.col) } else { l.col };
        self.doc().ed.set_cursor(Pos { line, col });
    }

    /// ]e and [e: the next or previous diagnostic in the buffer.
    pub(crate) fn next_diag(&mut self, forward: bool) {
        let d = &self.docs[self.cur];
        let c = d.ed.cursor();
        let mut ds: Vec<&Diag> = d.diags.iter().collect();
        ds.sort_by_key(|x| (x.line, x.col));
        let target = if forward {
            ds.iter().find(|x| (x.line, x.col) > (c.line, c.col)).or(ds.first())
        } else {
            ds.iter().rev().find(|x| (x.line, x.col) < (c.line, c.col)).or(ds.last())
        };
        match target.map(|x| (x.line, x.col, x.message.clone())) {
            Some((line, col, m)) => {
                self.doc().ed.set_cursor(Pos { line, col });
                self.say(m);
            }
            None => self.say("no errors or warnings here"),
        }
    }

    /// The diagnostic message on the cursor's line, for the bottom line.
    pub(crate) fn line_diag(&self) -> Option<(u8, String)> {
        let d = &self.docs[self.cur];
        let l = d.ed.cursor().line;
        d.diags.iter().filter(|x| x.line == l).min_by_key(|x| x.severity).map(|x| (x.severity, x.message.clone()))
    }
}

/// The colour of a diagnostic's severity.
pub(crate) fn severity_color(s: u8) -> Color {
    match s {
        1 => Color::Indexed(1),
        2 => Color::Indexed(3),
        3 => Color::Indexed(4),
        _ => Color::Indexed(8),
    }
}
