//! The core settings, as Neovim's options and Emacs's variables: one table (name, Vim's short name,
//! kind, default, what it does). A value is looked up in the buffer (`:setlocal`), then its file type
//! (`zett.ft.markdown.wrap`), then `zett.opt` (`:set`, init.lua), then the default.

use super::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Kind {
    Bool,
    Int,
    Text,
}

pub(crate) struct Setting {
    pub name: &'static str,
    pub short: &'static [&'static str],
    pub kind: Kind,
    pub default: &'static str,
    pub about: &'static str,
}

const fn s(name: &'static str, short: &'static [&'static str], kind: Kind, default: &'static str, about: &'static str) -> Setting {
    Setting { name, short, kind, default, about }
}

pub(crate) const SETTINGS: &[Setting] = &[
    // zett pops up the language server's completions while typing (Neovim's default: off)
    s("autocomplete", &["ac"], Kind::Bool, "true", "completions come up while typing, after autocompletedelay"),
    s("autocompletedelay", &["acl"], Kind::Int, "120", "milliseconds after typing before the completions come up"),
    s("wrap", &[], Kind::Bool, "false", "long lines go on in the next rows instead of off the side"),
    s("linebreak", &["lbr"], Kind::Bool, "true", "wrap between words, not inside one"),
    s("cursorline", &["cul"], Kind::Bool, "true", "the line with the cursor stands out"),
    s("colorcolumn", &["cc"], Kind::Text, "", "columns to mark, e.g. 80 or 80,120"),
    s("list", &[], Kind::Bool, "false", "show tabs, spaces at line ends and more as listchars says"),
    s("listchars", &["lcs"], Kind::Text, "tab:→ ,trail:·,nbsp:+", "what list shows: tab:XY[Z] trail: lead: space: nbsp: eol: extends: precedes:"),
    s("signcolumn", &["scl"], Kind::Text, "number", "where error signs go: number (in the line numbers), yes, auto, no"),
    s("foldtext", &["fdt"], Kind::Text, "", "what a closed fold shows: empty (zett's) its first line as it is, with …; an expression (foldtext(), v:lua.F()) its result"),
    s("undofile", &["udf"], Kind::Bool, "true", "every saved file keeps its undo history (time travel, undo after a restart; zett's default)"),
    s("winborder", &[], Kind::Text, "rounded", "the lines around floating boxes: rounded (zett's), single, double, bold, solid, shadow, none"),
    s("splitbelow", &["sb"], Kind::Bool, "true", ":split puts the new window below (zett's default, Doom's; off: above, Vim's)"),
    s("splitright", &["spr"], Kind::Bool, "true", ":vsplit puts the new window on the right (zett's default; off: left, Vim's)"),
    s("statusline", &["stl"], Kind::Text, "", "the status line's format (%f %m %l %c %P %= %#Group# …; empty: zett's mode line)"),
    s("showmode", &["smd"], Kind::Bool, "false", "-- INSERT -- and the like on the last line (the status line shows the mode already)"),
    s("scrolloff", &["so"], Kind::Int, "3", "lines kept above and below the cursor"),
    s("tabstop", &["ts"], Kind::Int, "4", "columns a tab character takes"),
    s("shiftwidth", &["sw"], Kind::Int, "4", "columns one indent step takes"),
    s("expandtab", &["et"], Kind::Bool, "true", "Tab inserts spaces, to the next indent step"),
    s("trim_trailing_whitespace", &["trim"], Kind::Bool, "false", "saving removes spaces at line ends"),
    s("final_newline", &["eol", "fixendofline", "fixeol"], Kind::Bool, "true", "saving ends the file with a line break (Vim's fixendofline)"),
    s("autosave", &[], Kind::Int, "0", "save changed files after this many seconds (0: never)"),
    s("ignorecase", &["ic"], Kind::Bool, "true", "searches match either case"),
    s("smartcase", &["scs"], Kind::Bool, "true", "… unless the search has a capital (with ignorecase; Emacs's way)"),
    s("wrapscan", &["ws"], Kind::Bool, "true", "searches (n, N, /) go on from the other end"),
    s("cmdline", &[], Kind::Text, "bottom", "where : asks: bottom (Vim), top, center (a box, like noice.nvim), cursor (a box at the cursor)"),
    s("cmdwidth", &[], Kind::Int, "64", "the width of the : box (center, cursor)"),
    s("fill_column", &["textwidth", "tw"], Kind::Int, "80", "where M-q (fill-paragraph) wraps"),
];

/// A setting by its name or short name: zett's own first, then the rest of Neovim's (nvim_options.rs).
pub(crate) fn find(name: &str) -> Option<&'static Setting> {
    let is = |s: &&Setting| s.name == name || s.short.contains(&name);
    SETTINGS.iter().find(is).or_else(|| crate::nvim_options::NVIM.iter().find(is))
}

/// Neovim options zett acts on outside this table (the window draws numbers, history is always kept).
const WORKS_ELSEWHERE: &[&str] = &[
    "number", "relativenumber",
    // the window's display options (display.rs, draw.rs)
    "cursorcolumn", "cursorlineopt", "showbreak", "breakindent", "numberwidth", "foldcolumn", "hlsearch", "synmaxcol",
    "sidescroll", "sidescrolloff", "fillchars", "showmatch",
    "timeout", "timeoutlen", "scrolljump",
    "gdefault", "history",
    "equalalways", "eadirection",
    "readonly", "modified", "modeline", "modelines", "modelineexpr",
    "mouse", "mousescroll", "mousetime", "mousemodel",
    "spell", "spellcapcheck",
    "filetype", "syntax", "guicursor", "termsync",
    "commentstring",
    // files, shells, tabs, windows, the title (commands.rs, files.rs, ex.rs, tabs.rs, doom.rs, lib.rs)
    "writeany", "patchmode", "warn", "tabclose", "winfixheight", "winfixwidth", "exrc", "titleold",
    "eventignore", "eventignorewin", "buftype", "buflisted", "updatetime", "isfname", "includeexpr",
    "confirm", "iminsert", "keymap", "verbose", "verbosefile", "secure",
    "inccommand", "foldexpr", "previewheight", "previewwindow",
    "undodir", "infercase", "complete", "completeopt", "dictionary", "wildchar", "wildcharm", "cmdwinheight", "winhighlight", "foldtext", "tabpagemax", "statuscolumn", "completefunc", "omnifunc", "thesaurus", "thesaurusfunc", "bufhidden", "pumborder", "packpath", "conceallevel", "concealcursor",
    "errorbells", "visualbell", "belloff", "matchtime", "showcmd", "showcmdloc", "operatorfunc",
    "pumheight", "pumwidth", "pummaxwidth",
    "cedit", "termguicolors", "messagesopt", "winfixbuf",
    "scrollbind", "cursorbind", "scrollopt",
    "winheight", "winwidth", "winminheight", "winminwidth",
    "breakat", "breakindentopt",
    "wildmenu", "wildmode", "wildignore", "wildignorecase", "wildoptions", "fileignorecase", "suffixes",
    "keywordprg",
    "vartabstop", "background", "sessionoptions", "viewoptions", "tags", "tagfunc", "findfunc", "quickfixtextfunc", "tagcase", "taglength", "tagrelative", "tagstack", "cdpath", "diff", "diffopt", "runtimepath", "viewdir", "include", "define", "path", "suffixesadd", "comments",
    "makeprg", "errorformat", "grepprg", "grepformat", "shell", "shellcmdflag", "shellquote", "shellxquote", "makeencoding",
    "errorfile", "switchbuf", "chistory", "lhistory",
    "maxmapdepth", "remap",
    "write", "backup", "writebackup", "backupdir", "backupext", "backupskip", "backupcopy", "fsync", "fileformat", "fileencoding",
    "bomb", "endofline", "endoffile", "fileformats", "fileencodings", "binary", "autoread", "autowrite", "autowriteall", "hidden", "autochdir",
    "loadplugins",
    "laststatus", "rulerformat", "ruler", "cmdheight", "showtabline", "tabline", "winbar", "title", "titlestring", "titlelen",
];

/// Options Neovim's terminal UI ignores (for GUIs, for Windows, or kept only for old scripts): zett
/// ignores them the same way (decided 2026-10-21) and the settings page says so.
pub(crate) const LIKE_NVIM_TUI: &[&str] = &[
    "cdhome", "guifont", "guifontwide", "guioptions", "guitablabel", "guitabtooltip", "linespace", "browsedir", "winaltkeys", "langmenu",
    "menuitems", "mousehide", "mouseshape", "opendevice", "shellslash", "completeslash", "termencoding", "ttyfast", "compatible",
    "edcompatible", "insertmode", "maxcombine", "encoding",
];

/// The setting does something in zett (zett's table, or acted on elsewhere); false: accepted and kept,
/// no effect yet.
pub(crate) fn works(s: &Setting) -> bool {
    SETTINGS.iter().any(|x| std::ptr::eq(x, s)) || WORKS_ELSEWHERE.contains(&s.name) || LIKE_NVIM_TUI.contains(&s.name) || zett_edit::vim::options::EDITOR_OPTIONS.contains(&s.name)
}

/// The file type as settings name it: "markdown", "plain_text", "rust".
pub(crate) fn ft_key(syntax: &str) -> String {
    syntax.to_lowercase().replace([' ', '-'], "_")
}

impl App {
    /// A setting's value for a buffer, as text ("true", "4", "80,120").
    pub(crate) fn setting(&self, di: usize, name: &str) -> String {
        let Some(s) = find(name) else { return String::new() };
        let d = &self.docs[di];
        if s.name == "modified" {
            return d.dirty().to_string();
        }
        if let Some(v) = d.local.get(s.name) {
            return v.clone();
        }
        if let Some(v) = self.engine.ft_opt(&ft_key(&d.syntax), s.name) {
            return v;
        }
        // termguicolors: on by itself when the terminal says it has 24-bit colour (COLORTERM), as
        // Neovim does; zash and zeit say so (before 2026-10-21 zett showed every colour rounded to 256)
        if s.name == "termguicolors" && self.engine.opt(s.name).is_none() {
            return std::env::var("COLORTERM").is_ok_and(|c| c == "truecolor" || c == "24bit").to_string();
        }
        self.engine.opt(s.name).unwrap_or_else(|| s.default.to_string())
    }
    pub(crate) fn flag(&self, di: usize, name: &str) -> bool {
        matches!(self.setting(di, name).as_str(), "true" | "1" | "on" | "yes")
    }
    pub(crate) fn num(&self, di: usize, name: &str) -> usize {
        self.setting(di, name).parse::<f64>().map(|v| v.max(0.0) as usize).unwrap_or(0)
    }

    /// `:set NAME[=VALUE]`, `:set noNAME`, `:set NAME?`, `:setlocal …` for a core setting; false
    /// when it is not one (a plugin's setting then).
    pub(crate) fn set_core(&mut self, arg: &str, local: bool) -> bool {
        // the end as it is: "\ " at the end of a value is a space in it (set_args took the backslash)
        let arg = arg.trim_start();
        let (name, value) = match arg.find(['=', ':', ' ']) {
            Some(i) => (arg[..i].trim(), Some(arg[i + 1..].trim_start())),
            None => (arg, None),
        };
        // name+= -= ^=: add, remove, prepend (set_op)
        let (name, op) = match name.chars().last() {
            Some(c @ ('+' | '-' | '^')) if value.is_some() => (&name[..name.len() - 1], Some(c)),
            _ => (name, None),
        };
        let ask = name.ends_with('?');
        // name& (or name&vim): back to the default; name! and invname: switched
        let reset = name.contains('&');
        let name = name.split('&').next().unwrap_or("").trim_end_matches('?');
        let toggle = name.ends_with('!');
        let name = name.trim_end_matches('!');
        let (setting, negated, toggle) = match find(name) {
            Some(s) => (s, false, toggle),
            None => match name.strip_prefix("no").and_then(find) {
                Some(s) if s.kind == Kind::Bool => (s, true, false),
                _ => match name.strip_prefix("inv").and_then(find) {
                    Some(s) if s.kind == Kind::Bool => (s, false, true),
                    _ => return false,
                },
            },
        };
        let cur = self.cur;
        let value = if reset {
            setting.default.to_string()
        } else if toggle && setting.kind == Kind::Bool {
            (!self.flag(cur, setting.name)).to_string()
        } else if let (Some(op), Some(v)) = (op, value) {
            set_op(&self.setting(cur, setting.name), op, v, setting.kind)
        } else if ask || (value.is_none() && setting.kind != Kind::Bool) {
            let v = self.setting(cur, setting.name);
            self.say(format!("{} = {v} (default {}) · {}", setting.name, setting.default, setting.about));
            return true;
        } else if setting.kind == Kind::Bool {
            match value {
                Some(v) => (!matches!(v, "false" | "off" | "no" | "0") ^ negated).to_string(),
                None => (!negated).to_string(),
            }
        } else {
            value.unwrap_or("").to_string()
        };
        if setting.kind == Kind::Int && value.parse::<i64>().is_err() {
            self.say(format!("{} wants a number, not {value}", setting.name));
            return true;
        }
        // modified: the buffer's changed mark itself, not a kept value
        if setting.name == "modified" {
            self.docs[cur].ed.buf.set_dirty(value == "true");
            self.say(format!("modified = {value}"));
            return true;
        }
        let before = self.setting(cur, setting.name);
        if local {
            self.docs[cur].local.insert(setting.name.to_string(), value.clone());
        } else {
            let kind = match setting.kind { Kind::Bool => 'b', Kind::Int => 'i', Kind::Text => 't' };
            if let Err(e) = self.engine.set_opt(setting.name, &value, kind) {
                self.say(e);
                return true;
            }
            // a :set wins over the file type's value, as in Neovim's :set (global and this buffer)
            self.docs[cur].local.insert(setting.name.to_string(), value.clone());
        }
        self.apply_settings();
        // background: the colour scheme loads again (it picks its dark or light side), as in Neovim
        if setting.name == "background" && before != value && self.colors_name != "default" {
            let name = self.colors_name.clone();
            self.command(&format!("colorscheme {name}"));
        }
        // quiet as in Vim (plugins set options every frame); :help options says which ones act
        let _ = works;
        true
    }

    /// modeline: options from a `vim: set …:` line among the first and last `modelines` lines of
    /// buffer `di`, set for it alone; what could run code only with modelineexpr.
    pub(crate) fn apply_modeline(&mut self, di: usize) {
        if !self.flag(di, "modeline") {
            return;
        }
        let (n, lines) = (self.num(di, "modelines"), self.docs[di].ed.buf.len_lines());
        let mut at: Vec<usize> = (0..n.min(lines)).collect();
        at.extend((lines.saturating_sub(n)..lines).filter(|l| *l >= n));
        let expr = self.flag(di, "modelineexpr");
        let (keep_cur, keep_msg) = (self.cur, self.msg.take());
        self.cur = di;
        for l in at {
            let Some(opts) = modeline_opts(&self.docs[di].ed.buf.line(l)) else { continue };
            for a in set_args(&opts) {
                let name = a.split(['=', ':', '+', '-', '^', '!', '&']).next().unwrap_or("");
                if expr || !modeline_unsafe(name) {
                    self.set_core(&a, true);
                }
            }
        }
        self.cur = keep_cur;
        self.msg = keep_msg;
    }

    /// Hands the settings the editor itself acts on to each buffer's editor.
    pub(crate) fn apply_settings(&mut self) {
        for i in 0..self.docs.len() {
            let (et, sw, fill) = (self.flag(i, "expandtab"), self.num(i, "shiftwidth").max(1), self.num(i, "fill_column"));
            let d = &mut self.docs[i];
            d.ed.set_soft_tab(et.then_some(sw));
            d.ed.set_fill_column(fill.max(10));
            let (ic, scs) = (self.flag(i, "ignorecase"), self.flag(i, "smartcase"));
            self.docs[i].ed.set_case(ic, scs);
            let ws = self.flag(i, "wrapscan");
            self.docs[i].ed.set_wrapscan(ws);
            crate::display::WINBORDER.store(crate::display::winborder_code(&self.setting(i, "winborder")), std::sync::atomic::Ordering::Relaxed);
            if hl::TRUECOLOR.swap(self.flag(i, "termguicolors"), std::sync::atomic::Ordering::Relaxed) != self.flag(i, "termguicolors") {
                hl::invalidate();
            }
            // filetype and syntax: the highlighting follows them (:set ft=rust, :set syntax=off)
            let (ft, syn) = (self.setting(i, "filetype"), self.setting(i, "syntax"));
            let key = if syn.is_empty() || syn.eq_ignore_ascii_case("on") { ft } else { syn };
            let want = if key.eq_ignore_ascii_case("off") { Some("Plain Text".to_string()) } else { self.ss.find_syntax_by_token(&key).map(|s| s.name.clone()) };
            if let Some(w) = want.filter(|w| *w != self.docs[i].syntax && !key.is_empty()) {
                self.docs[i].syntax = w;
                self.docs[i].states.clear();
            }
            // every option the editor acts on (zett_edit::vim::options), with this buffer's value;
            // textwidth only once set (zett's fill_column default is for M-q, not for wrapping while typing)
            for &name in zett_edit::vim::options::EDITOR_OPTIONS {
                let Some(s) = find(name) else { continue };
                let v = if name == "textwidth" && !self.docs[i].local.contains_key(s.name) && self.engine.opt(s.name).is_none() { "0".to_string() } else { self.setting(i, s.name) };
                self.docs[i].ed.set_opt(name, &v);
            }
        }
    }

    /// `:set` alone: every core setting with its value here, in a list.
    pub(crate) fn settings_list(&mut self) {
        let cur = self.cur;
        let list = all();
        let keys: Vec<String> = list
            .iter()
            .map(|s| {
                let v = self.setting(cur, s.name);
                format!("{} {} = {v}", if v != s.default { "●" } else { " " }, s.name)
            })
            .collect();
        // one column for the names and values, as wide as the longest
        let w = keys.iter().map(|k| k.chars().count()).max().unwrap_or(0) + 2;
        let rows = keys.into_iter().zip(list).map(|(k, s)| (format!("{k:<w$}"), format!("{}{}", s.about, if works(s) { "" } else { "  ◐ kept, no effect yet" }))).collect();
        self.popup = Some(Popup::Help("Settings (:set name=value · :setlocal · zett.ft.<type>)".into(), rows));
    }

    /// Before writing: what trim_trailing_whitespace and final_newline ask for.
    pub(crate) fn tidy_for_save(&mut self, di: usize) {
        // fixendofline (final_newline) adds the last line break, not for binary; without it the
        // file keeps or loses it as endofline says
        let (trim, eol) = (self.flag(di, "trim_trailing_whitespace"), self.flag(di, "final_newline") && !self.flag(di, "binary"));
        let ed = &mut self.docs[di].ed;
        let n = ed.buf.len_lines();
        let mut edits = Vec::new();
        if trim {
            for l in 0..n {
                let t = ed.buf.line(l);
                let t = t.trim_end_matches('\n');
                let kept = t.trim_end_matches([' ', '\t']).chars().count();
                let all = t.chars().count();
                if kept < all {
                    edits.push((Pos { line: l, col: kept }, Pos { line: l, col: all }, String::new()));
                }
            }
        }
        if eol {
            let len = ed.buf.len_chars();
            if len > 0 && !ed.buf.slice(len - 1, len).ends_with('\n') {
                let last = n.saturating_sub(1);
                let end = Pos { line: last, col: ed.buf.line(last).trim_end_matches('\n').chars().count() };
                edits.push((end, end, "\n".into()));
            }
        }
        if !edits.is_empty() {
            let keep = ed.cursor();
            ed.apply_edits(edits);
            let last = ed.buf.len_lines().saturating_sub(1);
            ed.set_cursor(Pos { line: keep.line.min(last), col: keep.col.min(ed.buf.line_len(keep.line.min(last))) });
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_and_short_names() {
        assert_eq!(super::find("ts").map(|s| s.name), Some("tabstop"));
        assert_eq!(super::find("textwidth").map(|s| s.name), Some("fill_column"));
        assert!(super::find("nowrap").is_none(), "no… is read by set_core, not find");
        assert_eq!(super::ft_key("Plain Text"), "plain_text");
    }
}

/// `:help settings`: every core setting, as Neovim's options page reads.
pub(crate) fn settings_topic() -> String {
    let mut out = String::from(
        "# Settings\n\nAs Neovim's options and Emacs's variables. Set one with `:set name=value` (`:set noname` turns\n\
         one off, `:set name?` shows it, `:set` alone lists them), for this buffer only with `:setlocal`, in\n\
         init.lua with `zett.opt.name = value`, for a file type with `zett.ft.markdown = { wrap = true }`.\n\
         A buffer's value comes from :setlocal, then its file type, then zett.opt, then the default.\n\n",
    );
    for s in all() {
        let kind = match s.kind {
            Kind::Bool => "on/off",
            Kind::Int => "number",
            Kind::Text => "text",
        };
        let short = if s.short.is_empty() { String::new() } else { format!(" · `{}`", s.short.join("`, `")) };
        let d = if s.default.is_empty() { "\"\"" } else { s.default };
        let state = if LIKE_NVIM_TUI.contains(&s.name) {
            " · ignored in a terminal, as by Neovim's"
        } else if works(s) {
            ""
        } else {
            " · ◐ kept, no effect yet"
        };
        out.push_str(&format!("- `{}`{short} · {kind} · default `{d}`{state}\n  {}\n", s.name, s.about));
    }
    out
}

/// Edits between two names (the help's "maybe").
pub(crate) fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut prev = row[0];
        row[0] = i;
        for j in 1..=b.len() {
            let cur = row[j];
            row[j] = (row[j] + 1).min(row[j - 1] + 1).min(prev + usize::from(a[i - 1] != b[j - 1]));
            prev = cur;
        }
    }
    row[b.len()]
}

/// Every setting :set knows: zett's own, then the rest of Neovim's (not those zett's table already has).
pub(crate) fn all() -> Vec<&'static Setting> {
    let mine = |n: &Setting| SETTINGS.iter().any(|s| s.name == n.name || s.short.contains(&n.name) || n.short.iter().any(|x| s.short.contains(x) || s.name == *x));
    SETTINGS.iter().chain(crate::nvim_options::NVIM.iter().filter(|n| !mine(n))).collect()
}

/// `:set`'s arguments one by one: Vim's `:set a b=c d+=e` (a backslash keeps a space in a value), or
/// zett's `:set NAME VALUE` (two words, the second not an option).
pub(crate) fn set_args(arg: &str) -> Vec<String> {
    let (mut out, mut cur, mut esc) = (Vec::new(), String::new(), false);
    for c in arg.trim_start().chars() {
        if esc {
            cur.push(c);
            esc = false;
        } else if c == '\\' {
            esc = true;
        } else if c.is_whitespace() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    let names = |w: &str| {
        let n = w.split(['=', ':', '+', '-', '^', '?', '!', '&']).next().unwrap_or("");
        find(n).is_some() || n.strip_prefix("no").or_else(|| n.strip_prefix("inv")).and_then(find).is_some()
    };
    if out.len() == 2 && !out[0].contains(['=', ':']) && !out[1].contains(['=', '?', '!', '&']) && !names(&out[1]) {
        return vec![format!("{}={}", out[0], out[1])];
    }
    out
}

/// `+=` `-=` `^=` on a value: numbers add, subtract, multiply; comma lists add, remove, prepend an
/// item; other text (flag letters) appends, removes, prepends.
pub(crate) fn set_op(cur: &str, op: char, v: &str, kind: Kind) -> String {
    if kind == Kind::Int {
        let (a, b) = (cur.parse::<i64>().unwrap_or(0), v.parse::<i64>().unwrap_or(0));
        return match op {
            '+' => a + b,
            '-' => a - b,
            _ => a * b,
        }
        .to_string();
    }
    if cur.contains(',') || v.contains(',') || cur.contains(':') {
        let mut items: Vec<&str> = cur.split(',').filter(|s| !s.is_empty()).collect();
        let new: Vec<&str> = v.split(',').filter(|s| !s.is_empty()).collect();
        match op {
            '-' => items.retain(|i| !new.contains(i)),
            '+' => items.extend(new.iter().filter(|n| !cur.split(',').any(|c| c == **n))),
            _ => {
                items.retain(|i| !new.contains(i));
                items.splice(0..0, new);
            }
        }
        return items.join(",");
    }
    match op {
        '-' => cur.replace(v, ""),
        '+' if !cur.contains(v) => format!("{cur}{v}"),
        '+' => cur.to_string(),
        _ => format!("{v}{}", cur.replace(v, "")),
    }
}

#[cfg(test)]
mod set_tests {
    use super::*;

    #[test]
    fn arguments_and_operators() {
        assert_eq!(set_args("list number tw=80"), vec!["list", "number", "tw=80"]);
        assert_eq!(set_args("textwidth 80"), vec!["textwidth=80"], "zett's NAME VALUE");
        assert_eq!(set_args("showbreak=>\\ \\ "), vec!["showbreak=>  "]);
        assert_eq!(set_args("wrap nolist"), vec!["wrap", "nolist"]);
        assert_eq!(set_op("80", '+', "8", Kind::Int), "88");
        assert_eq!(set_op("tab:> ,trail:-", '+', "eol:$", Kind::Text), "tab:> ,trail:-,eol:$");
        assert_eq!(set_op("indent,eol,start", '-', "eol", Kind::Text), "indent,start");
        assert_eq!(set_op("bin,hex", '^', "alpha", Kind::Text), "alpha,bin,hex");
        assert_eq!(set_op("tcqj", '-', "t", Kind::Text), "cqj");
        assert_eq!(set_op("cqj", '+', "t", Kind::Text), "cqjt");
    }
}

/// A modeline's options (`vim: set ts=4 sw=4 :`, `vi: ts=4`, `ex: …`), or None when the line has none.
pub(crate) fn modeline_opts(line: &str) -> Option<String> {
    let at = ["vim:", "Vim:", "vi:", "ex:"].iter().filter_map(|m| {
        line.match_indices(m).find(|(i, _)| *i == 0 || line[..*i].ends_with(char::is_whitespace)).map(|(i, _)| (i, m.len()))
    }).min()?;
    let rest = line[at.0 + at.1..].trim_start();
    // second form: set … : (the options end at a colon not escaped)
    if let Some(r) = rest.strip_prefix("set ").or_else(|| rest.strip_prefix("se ")) {
        let mut out = String::new();
        let mut cs = r.chars().peekable();
        while let Some(c) = cs.next() {
            match c {
                '\\' if cs.peek() == Some(&':') => out.push(cs.next().unwrap()),
                ':' => return Some(out.trim().to_string()),
                c => out.push(c),
            }
        }
        return None;
    }
    // first form: options split by blanks or colons, to the line's end
    Some(rest.split([':', ' ', '\t']).filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" "))
}

/// An option a modeline may not set without modelineexpr: what runs code or programs.
pub(crate) fn modeline_unsafe(name: &str) -> bool {
    let n = find(name.trim_start_matches("no")).map_or(name, |s| s.name);
    n.ends_with("expr") || n.ends_with("func") || n.ends_with("prg") || n.starts_with("shell") || matches!(n, "statusline" | "tabline" | "winbar" | "titlestring" | "rulerformat" | "iconstring" | "foldtext" | "makeef" | "keywordprg" | "operatorfunc" | "patchmode" | "spellfile" | "backupdir" | "directory" | "undodir" | "viewdir")
}

#[cfg(test)]
mod modeline_tests {
    use super::*;

    #[test]
    fn both_forms_and_the_unsafe_ones() {
        assert_eq!(modeline_opts("// vim: set ts=4 sw=4 et :").as_deref(), Some("ts=4 sw=4 et"));
        assert_eq!(modeline_opts("# vim: ts=8:sw=8").as_deref(), Some("ts=8 sw=8"));
        assert_eq!(modeline_opts("/* vi: set tw=72: */").as_deref(), Some("tw=72"));
        assert_eq!(modeline_opts("no modeline here"), None);
        assert!(modeline_unsafe("statusline") && modeline_unsafe("foldexpr") && modeline_unsafe("makeprg") && modeline_unsafe("shell"));
        assert!(!modeline_unsafe("ts") && !modeline_unsafe("list"));
    }
}
