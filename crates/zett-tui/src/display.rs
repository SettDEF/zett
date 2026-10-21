//! The display options a window is drawn with (cursorcolumn, cursorlineopt, listchars, showbreak,
//! breakindent, numberwidth, signcolumn, foldcolumn, hlsearch, synmaxcol, sidescroll, fillchars),
//! read once per draw.

use super::*;

/// Tab cells as expand() writes them while list is on; the window draws listchars' tab in their place.
pub(crate) const TAB_FIRST: char = '\u{E000}';
pub(crate) const TAB_FILL: char = '\u{E001}';
pub(crate) const TAB_LAST: char = '\u{E002}';

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ListChars {
    pub tab: Option<(char, char, Option<char>)>,
    pub trail: Option<char>,
    pub lead: Option<char>,
    pub space: Option<char>,
    pub nbsp: Option<char>,
    pub eol: Option<char>,
    pub extends: Option<char>,
    pub precedes: Option<char>,
}

/// "tab:→ ,trail:·,eol:$" as its parts.
pub(crate) fn listchars(v: &str) -> ListChars {
    let mut l = ListChars::default();
    for part in v.split(',') {
        let Some((k, val)) = part.split_once(':') else { continue };
        let cs: Vec<char> = val.chars().collect();
        let one = cs.first().copied();
        match k {
            "tab" if cs.len() >= 2 => l.tab = Some((cs[0], cs[1], cs.get(2).copied())),
            "trail" => l.trail = one,
            "lead" => l.lead = one,
            "space" => l.space = one,
            "nbsp" => l.nbsp = one,
            "eol" => l.eol = one,
            "extends" => l.extends = one,
            "precedes" => l.precedes = one,
            _ => {}
        }
    }
    l
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Fill {
    pub eob: char,
    pub vert: char,
    pub foldopen: char,
    pub foldclose: char,
    pub foldsep: char,
}

/// fillchars ("eob: ,vert:|"), over Neovim's defaults.
pub(crate) fn fillchars(v: &str) -> Fill {
    let mut f = Fill { eob: '~', vert: '│', foldopen: '-', foldclose: '+', foldsep: '│' };
    for part in v.split(',') {
        let Some((k, val)) = part.split_once(':') else { continue };
        let Some(c) = val.chars().next() else { continue };
        match k {
            "eob" => f.eob = c,
            "vert" => f.vert = c,
            "foldopen" => f.foldopen = c,
            "foldclose" => f.foldclose = c,
            "foldsep" => f.foldsep = c,
            _ => {}
        }
    }
    f
}

/// Where diagnostics' signs go: in the number column (zett's), their own two columns always, only
/// when there are some, or nowhere.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Signs {
    Number,
    Yes,
    Auto,
    No,
}

pub(crate) struct Display {
    pub cursorcolumn: bool,
    pub cul_line: bool,
    pub cul_number: bool,
    pub list: Option<ListChars>,
    pub showbreak: String,
    pub breakindent: bool,
    pub numberwidth: usize,
    pub signs: Signs,
    pub foldcolumn: usize,
    pub hlsearch: bool,
    pub synmaxcol: usize,
    pub sidescroll: usize,
    pub sidescrolloff: usize,
    pub scrolljump: i64,
    pub fill: Fill,
}

impl App {
    pub(crate) fn display(&self, di: usize) -> Display {
        let culopt = self.setting(di, "cursorlineopt");
        let has = |w: &str| culopt.split(',').any(|p| p == w || p == "both" || (w == "line" && p == "screenline"));
        let fc = self.setting(di, "foldcolumn");
        // foldcolumn: N, or auto[:N] (shown only where folds are; zett shows it while any exist)
        let foldcolumn = match fc.strip_prefix("auto") {
            Some(n) => {
                let n = n.trim_start_matches(':').parse().unwrap_or(1);
                if self.docs[di].ed.has_folds() { n } else { 0 }
            }
            None => fc.parse().unwrap_or(0),
        };
        Display {
            cursorcolumn: self.flag(di, "cursorcolumn"),
            cul_line: has("line"),
            cul_number: has("number"),
            list: self.flag(di, "list").then(|| listchars(&self.setting(di, "listchars"))),
            showbreak: self.setting(di, "showbreak"),
            breakindent: self.flag(di, "breakindent"),
            numberwidth: self.num(di, "numberwidth").max(1),
            signs: match self.setting(di, "signcolumn").split(':').next().unwrap_or("") {
                "yes" => Signs::Yes,
                "auto" => Signs::Auto,
                "no" => Signs::No,
                _ => Signs::Number,
            },
            foldcolumn: foldcolumn.min(12),
            hlsearch: self.flag(di, "hlsearch"),
            synmaxcol: match self.num(di, "synmaxcol") {
                0 => usize::MAX,
                n => n,
            },
            sidescroll: self.num(di, "sidescroll"),
            sidescrolloff: self.num(di, "sidescrolloff"),
            scrolljump: self.setting(di, "scrolljump").parse().unwrap_or(1),
            fill: fillchars(&self.setting(di, "fillchars")),
        }
    }
}

/// The fold column's cells for a line: a mark where a fold starts (open or closed), a bar inside one.
pub(crate) fn fold_cells(width: usize, depth: usize, start: Option<bool>, f: &Fill) -> String {
    if width == 0 {
        return String::new();
    }
    let mut s = String::new();
    for i in 0..width {
        let c = if i + 1 == depth.min(width) {
            match start {
                Some(true) => f.foldclose,
                Some(false) => f.foldopen,
                None => f.foldsep,
            }
        } else if i < depth {
            f.foldsep
        } else {
            ' '
        };
        s.push(c);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_parse() {
        let l = listchars("tab:>-,trail:~,eol:$,nbsp:+");
        assert_eq!(l.tab, Some(('>', '-', None)));
        assert_eq!((l.trail, l.eol, l.nbsp, l.space), (Some('~'), Some('$'), Some('+'), None));
        assert_eq!(listchars("tab:<->").tab, Some(('<', '-', Some('>'))));
        let f = fillchars("eob: ,vert:|");
        assert_eq!((f.eob, f.vert, f.foldclose), (' ', '|', '+'));
        assert_eq!(fold_cells(2, 1, Some(true), &f), "+ ");
        assert_eq!(fold_cells(2, 2, None, &f), "││");
        assert_eq!(fold_cells(2, 0, None, &f), "  ");
        let gc = "n-v-c-sm:block,i-ci-ve:ver25,r-cr-o:hor20,a:blinkon0";
        assert_eq!(cursor_for(gc, "i"), Some(("ver", false)));
        assert_eq!(cursor_for(gc, "o"), Some(("hor", false)));
        assert_eq!(cursor_for("n:block-blinkon250", "n"), Some(("block", true)));
        assert_eq!(cursor_for("", "n"), None);
    }
}

/// `s` right-aligned in a gutter `w` columns wide (cut to fit).
pub(crate) fn gut(s: &str, w: usize) -> String {
    let n = s.chars().count();
    if n >= w { s.chars().skip(n - w).collect() } else { format!("{}{s}", " ".repeat(w - n)) }
}

/// The line ends a sentence (. ! ? and maybe a closing quote or bracket): the next starts big.
pub(crate) fn ends_sentence(line: &str) -> bool {
    let t = line.trim_end().trim_end_matches(['"', '\'', ')', ']']);
    t.ends_with(['.', '!', '?'])
}

/// Spelling marks in screen columns: misspelled words, and with `capcheck` words starting a
/// sentence (after . ! ?, or at the line start when the line before ended one) with a small letter.
pub(crate) fn spell_marks(line: &str, camel: bool, capcheck: bool, starts_sentence: bool) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
    let body = line.trim_end_matches('\n');
    let bad = zett_edit::spell::with(|s| s.bad_in(body, camel));
    let mut cap = Vec::new();
    if capcheck {
        let cs: Vec<char> = body.chars().collect();
        let mut after_end = starts_sentence;
        let mut i = 0;
        while i < cs.len() {
            let c = cs[i];
            if c.is_alphabetic() {
                let a = i;
                while i < cs.len() && cs[i].is_alphabetic() {
                    i += 1;
                }
                if after_end && c.is_lowercase() {
                    cap.push((a, i));
                }
                after_end = false;
                continue;
            }
            if matches!(c, '.' | '!' | '?') {
                after_end = cs.get(i + 1).is_none_or(|n| n.is_whitespace() || matches!(n, '"' | '\'' | ')' | ']'));
            } else if !c.is_whitespace() && !matches!(c, '"' | '\'' | ')' | ']') {
                after_end = false;
            }
            i += 1;
        }
    }
    let to_screen = |v: Vec<(usize, usize)>| v.into_iter().map(|(a, b)| (crate::screen_col(line, a), crate::screen_col(line, b))).collect();
    (to_screen(bad), to_screen(cap))
}

impl App {
    /// guicursor ("n-v-c-sm:block,i-ci-ve:ver25,r-cr-o:hor20"): the cursor for the mode now; None
    /// when guicursor is empty (the terminal's own cursor stays).
    pub(crate) fn cursor_shape(&self) -> Option<(&'static str, bool)> {
        let ed = &self.docs[self.cur].ed;
        let mode = if self.cmdline.is_some() || self.minibuf.is_some() {
            "c"
        } else {
            match ed.mode() {
                Mode::Insert if ed.replacing() => "r",
                Mode::Insert => "i",
                Mode::Normal if ed.pending_op() => "o",
                Mode::Normal => "n",
                _ => "v",
            }
        };
        cursor_for(&self.setting(self.cur, "guicursor"), mode)
    }
}

/// The guicursor parts for `mode` as a terminal cursor style.
pub(crate) fn cursor_for(gc: &str, mode: &str) -> Option<(&'static str, bool)> {
    if gc.trim().is_empty() {
        return None;
    }
    // "a:" parts apply to every mode (blinking, highlight); the mode's own part sets the shape
    let mut shape = "block";
    let mut blink = false;
    for part in gc.split(',') {
        let Some((modes, args)) = part.split_once(':') else { continue };
        if !modes.split('-').any(|m| m == mode || m == "a") {
            continue;
        }
        for a in args.split('-') {
            if a == "block" || a.starts_with("ver") || a.starts_with("hor") {
                shape = if a.starts_with("ver") { "ver" } else if a.starts_with("hor") { "hor" } else { "block" };
            } else if let Some(n) = a.strip_prefix("blinkon") {
                blink = n.parse::<u32>().is_ok_and(|n| n > 0);
            }
        }
    }
    Some((shape, blink))
}

/// A shape from cursor_for as the terminal's cursor style.
pub(crate) fn cursor_style((shape, blink): (&str, bool)) -> ratatui::crossterm::cursor::SetCursorStyle {
    use ratatui::crossterm::cursor::SetCursorStyle as C;
    match (shape, blink) {
        ("ver", true) => C::BlinkingBar,
        ("ver", false) => C::SteadyBar,
        ("hor", true) => C::BlinkingUnderScore,
        ("hor", false) => C::SteadyUnderScore,
        (_, true) => C::BlinkingBlock,
        _ => C::SteadyBlock,
    }
}


/// winborder as a number for the drawing code that has no settings at hand (float_set).
pub(crate) static WINBORDER: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(2);

/// winborder's name as its number: single 1, rounded 2, double 3, bold 4, none / solid 5 (spaces).
pub(crate) fn winborder_code(v: &str) -> u8 {
    match v.split(',').next().unwrap_or("") {
        "single" | "shadow" => 1,
        "double" => 3,
        "bold" => 4,
        "none" | "solid" => 5,
        _ => 2,
    }
}

/// The lines floating boxes are drawn with (winborder); none and solid keep the box's size with
/// a border of spaces.
pub(crate) fn float_set() -> ratatui::symbols::border::Set {
    use ratatui::symbols::border;
    match WINBORDER.load(std::sync::atomic::Ordering::Relaxed) {
        1 => border::PLAIN,
        3 => border::DOUBLE,
        4 => border::THICK,
        5 => border::Set { top_left: " ", top_right: " ", bottom_left: " ", bottom_right: " ", vertical_left: " ", vertical_right: " ", horizontal_top: " ", horizontal_bottom: " " },
        _ => border::ROUNDED,
    }
}
