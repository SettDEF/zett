//! Vim's status line format, shared by statusline, rulerformat, tabline, winbar and titlestring:
//! %f %F %t %m %M %r %R %h %w %y %Y %n %l %L %c %v %V %p %P %b %B %o %O %{expr} %= %#Group# %1* %*,
//! with widths (%-10.20f, %03l) and %( %) groups that vanish when all their items are empty.

/// What the items say about a buffer and its window.
#[derive(Default)]
pub(crate) struct Ctx {
    pub name: String,
    pub full: String,
    pub modified: bool,
    pub modifiable: bool,
    pub readonly: bool,
    pub help: bool,
    pub ft: String,
    pub bufnr: usize,
    /// 1-based line and column (bytes), the screen column, the lines in the buffer
    pub line: usize,
    pub col: usize,
    pub vcol: usize,
    pub lines: usize,
    /// the first line shown and the window's height (for %P)
    pub top: usize,
    pub height: usize,
    pub ch: Option<char>,
    /// 1-based byte offset of the cursor in the file
    pub offset: usize,
    /// showcmd with showcmdloc=statusline: the keys under way (%S)
    pub showcmd: String,
}

/// A piece of the line: its text and the highlight group it is drawn in (None: the line's own).
pub(crate) type Piece = (String, Option<String>);

/// The format laid out `width` columns wide: `%=` shares the free space, too long is cut at the end.
/// `eval` answers %{expr}.
pub(crate) fn render(fmt: &str, c: &Ctx, width: usize, eval: &mut dyn FnMut(&str) -> String) -> Vec<Piece> {
    // sections between %= marks; each a list of pieces
    let mut sections: Vec<Vec<Piece>> = vec![Vec::new()];
    let mut group: Option<String> = None;
    // %( … %): where the group began, and whether an item in it said anything
    let mut opened: Vec<(usize, usize, bool)> = Vec::new();
    let mut lit = String::new();
    let chars: Vec<char> = fmt.chars().collect();
    let mut i = 0;
    let flush = |lit: &mut String, sections: &mut Vec<Vec<Piece>>, group: &Option<String>| {
        if !lit.is_empty() {
            sections.last_mut().unwrap().push((std::mem::take(lit), group.clone()));
        }
    };
    while i < chars.len() {
        if chars[i] != '%' || i + 1 >= chars.len() {
            lit.push(chars[i]);
            i += 1;
            continue;
        }
        i += 1;
        // %-0N.M
        let left = chars.get(i) == Some(&'-');
        if left {
            i += 1;
        }
        let zero = chars.get(i) == Some(&'0');
        let mut minw = 0usize;
        while let Some(d) = chars.get(i).and_then(|c| c.to_digit(10)) {
            minw = minw * 10 + d as usize;
            i += 1;
        }
        let mut maxw = usize::MAX;
        if chars.get(i) == Some(&'.') {
            i += 1;
            maxw = 0;
            while let Some(d) = chars.get(i).and_then(|c| c.to_digit(10)) {
                maxw = maxw * 10 + d as usize;
                i += 1;
            }
        }
        let Some(&item) = chars.get(i) else { break };
        i += 1;
        let text: Option<String> = match item {
            '%' => {
                lit.push('%');
                continue;
            }
            '=' => {
                flush(&mut lit, &mut sections, &group);
                sections.push(Vec::new());
                continue;
            }
            '<' => continue,
            '#' => {
                // %#Group#
                let end = chars[i..].iter().position(|&c| c == '#').map(|p| i + p).unwrap_or(chars.len());
                flush(&mut lit, &mut sections, &group);
                let g: String = chars[i..end].iter().collect();
                group = (!g.is_empty()).then_some(g);
                i = end + 1;
                continue;
            }
            '*' => {
                // %N* User1..9, %* back to the line's own
                flush(&mut lit, &mut sections, &group);
                group = (minw > 0).then(|| format!("User{}", minw.min(9)));
                continue;
            }
            '(' => {
                flush(&mut lit, &mut sections, &group);
                opened.push((sections.len() - 1, sections.last().unwrap().len(), false));
                continue;
            }
            ')' => {
                flush(&mut lit, &mut sections, &group);
                if let Some((s, at, said)) = opened.pop() {
                    if !said && s == sections.len() - 1 {
                        sections[s].truncate(at);
                    }
                }
                continue;
            }
            '{' => {
                let end = chars[i..].iter().position(|&c| c == '}').map(|p| i + p).unwrap_or(chars.len());
                let expr: String = chars[i..end].iter().collect();
                i = end + 1;
                Some(eval(&expr))
            }
            'f' => Some(c.name.clone()),
            'F' => Some(c.full.clone()),
            't' => Some(std::path::Path::new(&c.full).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| c.name.clone())),
            'm' => Some(if c.modified { "[+]" } else if !c.modifiable { "[-]" } else { "" }.into()),
            'M' => Some(if c.modified { ",+" } else if !c.modifiable { ",-" } else { "" }.into()),
            'r' => Some(if c.readonly { "[RO]" } else { "" }.into()),
            'R' => Some(if c.readonly { ",RO" } else { "" }.into()),
            'h' => Some(if c.help { "[Help]" } else { "" }.into()),
            'H' => Some(if c.help { ",HLP" } else { "" }.into()),
            'w' | 'W' | 'q' | 'k' | 'a' | 'N' => Some(String::new()),
            'y' => Some(if c.ft.is_empty() { String::new() } else { format!("[{}]", c.ft) }),
            'Y' => Some(if c.ft.is_empty() { String::new() } else { format!(",{}", c.ft.to_uppercase()) }),
            'n' => Some(c.bufnr.to_string()),
            'l' => Some(c.line.to_string()),
            'L' => Some(c.lines.to_string()),
            'c' => Some(c.col.to_string()),
            'v' => Some(c.vcol.to_string()),
            'V' => Some(if c.vcol == c.col { String::new() } else { format!("-{}", c.vcol) }),
            'p' => Some((c.line * 100 / c.lines.max(1)).to_string()),
            'P' => Some(place(c)),
            'b' => Some(c.ch.map(|x| (x as u32).to_string()).unwrap_or_else(|| "0".into())),
            'B' => Some(c.ch.map(|x| format!("{:x}", x as u32)).unwrap_or_else(|| "0".into())),
            'o' => Some(c.offset.to_string()),
            'S' => Some(c.showcmd.clone()),
            'O' => Some(format!("{:x}", c.offset)),
            _ => None,
        };
        let Some(mut t) = text else { continue };
        if t.chars().count() > maxw {
            // too long: cut from the start, as Vim does, with < where it was cut
            let n = t.chars().count();
            t = std::iter::once('<').chain(t.chars().skip(n - maxw + 1)).collect();
        }
        let n = t.chars().count();
        if n < minw {
            let pad = minw - n;
            let fill = if zero && !left && t.chars().all(|c| c.is_ascii_digit()) { "0" } else { " " };
            t = if left { format!("{t}{}", " ".repeat(pad)) } else { format!("{}{t}", fill.repeat(pad)) };
        }
        if !t.trim().is_empty() {
            if let Some(o) = opened.last_mut() {
                o.2 = true;
            }
        }
        lit.push_str(&t);
    }
    flush(&mut lit, &mut sections, &group);
    lay_out(sections, width)
}

/// Sections side by side in `width` columns: the free space spread over the %= gaps.
fn lay_out(sections: Vec<Vec<Piece>>, width: usize) -> Vec<Piece> {
    let used: usize = sections.iter().flatten().map(|p| p.0.chars().count()).sum();
    let gaps = sections.len().saturating_sub(1);
    let free = width.saturating_sub(used);
    let mut out = Vec::new();
    for (k, s) in sections.into_iter().enumerate() {
        if k > 0 {
            let share = free / gaps + usize::from(k <= free % gaps);
            out.push((" ".repeat(share), None));
        }
        out.extend(s);
    }
    // cut what does not fit
    let mut room = width;
    let mut cut = Vec::new();
    for (t, g) in out {
        if room == 0 {
            break;
        }
        let t: String = t.chars().take(room).collect();
        room -= t.chars().count();
        cut.push((t, g));
    }
    cut
}

/// %P: Top, Bot, All, or how far down the window's top is.
fn place(c: &Ctx) -> String {
    let top = c.top;
    let last_shown = top + c.height;
    if top == 0 && last_shown >= c.lines {
        "All".into()
    } else if top == 0 {
        "Top".into()
    } else if last_shown >= c.lines {
        "Bot".into()
    } else {
        format!("{}%", top * 100 / c.lines.saturating_sub(c.height).max(1))
    }
}

/// The pieces as plain text (titlestring, tests).
pub(crate) fn plain(p: &[Piece]) -> String {
    p.iter().map(|x| x.0.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Ctx {
        Ctx { name: "src/main.rs".into(), full: "/p/src/main.rs".into(), modified: true, modifiable: true, ft: "rust".into(), bufnr: 2, line: 7, col: 3, vcol: 5, lines: 70, height: 20, ..Default::default() }
    }

    #[test]
    fn items_widths_alignment_groups() {
        let mut no = |_: &str| String::new();
        let s = |f: &str, w: usize| plain(&render(f, &ctx(), w, &mut |_| "E".into()));
        assert_eq!(s("%f %m%r %y", 40), "src/main.rs [+] [rust]");
        assert_eq!(s("%t%=%l,%c%V %P", 24), "main.rs        7,3-5 Top");
        assert_eq!(s("%03l|%-4n|%5L", 40), "007|2   |   70");
        assert_eq!(s("%.5F", 40), "<n.rs");
        assert_eq!(s("a%( [%r]%)b", 40), "ab", "a group of empty items goes");
        assert_eq!(s("a%( [%m]%)b", 40), "a [[+]]b");
        assert_eq!(s("%{x}%%", 40), "E%");
        let p = render("%#Error#%m%*%f", &ctx(), 40, &mut no);
        assert_eq!(p[0], ("[+]".to_string(), Some("Error".to_string())));
        assert_eq!(p[1].1, None);
        assert_eq!(s("x%=y%=z", 7), "x  y  z");
        assert_eq!(s("%f", 4), "src/", "too long is cut");
    }
}

impl crate::App {
    /// What the format items say about buffer `di` in a window `height` rows high.
    pub(crate) fn status_ctx(&self, di: usize, height: usize) -> Ctx {
        let d = &self.docs[di];
        let c = d.ed.cursor();
        let line_text = d.ed.buf.line(c.line);
        let before: String = line_text.chars().take(c.col).collect();
        Ctx {
            name: d.name.strip_prefix(&format!("{}/", self.root.display())).unwrap_or(&d.name).to_string(),
            full: d.name.clone(),
            modified: d.dirty(),
            modifiable: self.setting(di, "modifiable") != "false",
            readonly: self.flag(di, "readonly"),
            help: d.name.starts_with("help:"),
            ft: crate::settings::ft_key(&d.syntax),
            bufnr: di + 1,
            line: c.line + 1,
            col: before.len() + 1,
            vcol: crate::screen_col(&line_text, c.col) + 1,
            lines: d.ed.buf.len_lines().max(1),
            top: d.top,
            height,
            ch: line_text.chars().nth(c.col).filter(|&ch| ch != '\n'),
            offset: d.ed.buf.to_char(c) + 1,
            showcmd: if self.flag(di, "showcmd") && self.setting(di, "showcmdloc") == "statusline" { d.ed.pending_keys() } else { String::new() },
        }
    }

    /// A format (statusline, rulerformat, tabline, winbar, titlestring) for buffer `di`, `width`
    /// columns wide; %{expr} is a Lua expression.
    pub(crate) fn fmt_line(&mut self, fmt: &str, di: usize, height: usize, width: usize) -> Vec<Piece> {
        let c = self.status_ctx(di, height);
        let engine = &mut self.engine;
        render(fmt, &c, width, &mut |e| engine.eval(&format!("return {e}")).map(|r| r.0).unwrap_or_default())
    }
}

/// The pieces as a line in area `a`: in `base`, each piece in its own group where it names one.
pub(crate) fn draw_pieces(f: &mut ratatui::Frame, a: ratatui::layout::Rect, pieces: Vec<Piece>, base: ratatui::style::Style) {
    use ratatui::text::{Line, Span};
    let spans: Vec<Span> = pieces.into_iter().map(|(t, g)| Span::styled(t, g.map_or(base, |g| base.patch(crate::hl::hl(&g))))).collect();
    f.render_widget(ratatui::widgets::Paragraph::new(Line::from(spans)).style(base), a);
}
