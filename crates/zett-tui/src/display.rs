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
    }
}

/// `s` right-aligned in a gutter `w` columns wide (cut to fit).
pub(crate) fn gut(s: &str, w: usize) -> String {
    let n = s.chars().count();
    if n >= w { s.chars().skip(n - w).collect() } else { format!("{}{s}", " ".repeat(w - n)) }
}
