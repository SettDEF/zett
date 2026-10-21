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
    s("wrap", &[], Kind::Bool, "false", "long lines go on in the next rows instead of off the side"),
    s("linebreak", &["lbr"], Kind::Bool, "true", "wrap between words, not inside one"),
    s("cursorline", &["cul"], Kind::Bool, "true", "the line with the cursor stands out"),
    s("colorcolumn", &["cc"], Kind::Text, "", "columns to mark, e.g. 80 or 80,120"),
    s("list", &[], Kind::Bool, "false", "show tabs (→) and spaces at line ends (·)"),
    s("scrolloff", &["so"], Kind::Int, "3", "lines kept above and below the cursor"),
    s("tabstop", &["ts"], Kind::Int, "4", "columns a tab character takes"),
    s("shiftwidth", &["sw"], Kind::Int, "4", "columns one indent step takes"),
    s("expandtab", &["et"], Kind::Bool, "true", "Tab inserts spaces, to the next indent step"),
    s("trim_trailing_whitespace", &["trim"], Kind::Bool, "false", "saving removes spaces at line ends"),
    s("final_newline", &["eol"], Kind::Bool, "true", "saving ends the file with a line break"),
    s("autosave", &[], Kind::Int, "0", "save changed files after this many seconds (0: never)"),
    s("ignorecase", &["ic"], Kind::Bool, "true", "searches match either case"),
    s("smartcase", &["scs"], Kind::Bool, "true", "… unless the search has a capital (with ignorecase; Emacs's way)"),
    s("cmdline", &[], Kind::Text, "bottom", "where : asks: bottom (Vim), top, center (a box, like noice.nvim), cursor (a box at the cursor)"),
    s("cmdwidth", &[], Kind::Int, "64", "the width of the : box (center, cursor)"),
    s("fill_column", &["textwidth", "tw"], Kind::Int, "80", "where M-q (fill-paragraph) wraps"),
];

/// A setting by its name or short name.
pub(crate) fn find(name: &str) -> Option<&'static Setting> {
    SETTINGS.iter().find(|s| s.name == name || s.short.contains(&name))
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
        if let Some(v) = d.local.get(s.name) {
            return v.clone();
        }
        if let Some(v) = self.engine.ft_opt(&ft_key(&d.syntax), s.name) {
            return v;
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
        let arg = arg.trim();
        let (name, value) = match arg.split_once(['=', ' ']) {
            Some((n, v)) => (n.trim(), Some(v.trim())),
            None => (arg, None),
        };
        let ask = name.ends_with('?');
        let name = name.trim_end_matches('?');
        let (setting, negated) = match find(name) {
            Some(s) => (s, false),
            None => match name.strip_prefix("no").and_then(find) {
                Some(s) if s.kind == Kind::Bool => (s, true),
                _ => return false,
            },
        };
        let cur = self.cur;
        let value = if ask || (value.is_none() && setting.kind != Kind::Bool) {
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
        self.say(format!("{}{} = {value}", if local { "local " } else { "" }, setting.name));
        true
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
        }
    }

    /// `:set` alone: every core setting with its value here, in a list.
    pub(crate) fn settings_list(&mut self) {
        let cur = self.cur;
        let keys: Vec<String> = SETTINGS
            .iter()
            .map(|s| {
                let v = self.setting(cur, s.name);
                format!("{} {} = {v}", if v != s.default { "●" } else { " " }, s.name)
            })
            .collect();
        // one column for the names and values, as wide as the longest
        let w = keys.iter().map(|k| k.chars().count()).max().unwrap_or(0) + 2;
        let rows = keys.into_iter().zip(SETTINGS).map(|(k, s)| (format!("{k:<w$}"), s.about.to_string())).collect();
        self.popup = Some(Popup::Help("Settings (:set name=value · :setlocal · zett.ft.<type>)".into(), rows));
    }

    /// Before writing: what trim_trailing_whitespace and final_newline ask for.
    pub(crate) fn tidy_for_save(&mut self, di: usize) {
        let (trim, eol) = (self.flag(di, "trim_trailing_whitespace"), self.flag(di, "final_newline"));
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
    for s in SETTINGS {
        let kind = match s.kind {
            Kind::Bool => "on/off",
            Kind::Int => "number",
            Kind::Text => "text",
        };
        let short = if s.short.is_empty() { String::new() } else { format!(" · `{}`", s.short.join("`, `")) };
        let d = if s.default.is_empty() { "\"\"" } else { s.default };
        out.push_str(&format!("- `{}`{short} · {kind} · default `{d}`\n  {}\n", s.name, s.about));
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
