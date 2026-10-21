//! Highlight groups, as Neovim has them: every colour zett draws has a name `zett.hl` and
//! `:hi` change. Colours are the terminal's 16 (so the theme follows), `#rrggbb`, or names.

use super::Engine;
use mlua::{Table, Value};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HlColor {
    /// a terminal palette colour (0–255)
    Index(u8),
    Rgb(u8, u8, u8),
    /// the terminal's own foreground or background
    Default,
    /// the theme's background lifted N% toward its foreground (`+12`): a tint that follows the theme
    Lift(u8),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hl {
    pub fg: Option<HlColor>,
    pub bg: Option<HlColor>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
    pub strikethrough: bool,
    /// take another group's look instead
    pub link: Option<String>,
}

/// zett's groups and how they look until a config says otherwise (`:hi` syntax).
pub const DEFAULTS: &[(&str, &str)] = &[
    // syntax
    ("Comment", "fg=8 italic"),
    ("String", "fg=2"),
    ("Number", "fg=3"),
    ("Constant", "fg=11"),
    ("Keyword", "fg=5"),
    ("Operator", "fg=6"),
    ("Type", "fg=3"),
    ("Function", "fg=4"),
    ("Tag", "fg=1"),
    ("Attribute", "fg=3"),
    ("Macro", "fg=6"),
    ("Parameter", "fg=9"),
    ("Special", "fg=1 italic"),
    ("Heading", "fg=4 bold"),
    ("Bold", "bold"),
    ("Italic", "italic"),
    ("Raw", "fg=2"),
    ("Link", "fg=6 underline"),
    ("ListMarker", "fg=5"),
    ("Quote", "fg=8 italic"),
    ("Invalid", "fg=1 underline"),
    // the editor
    ("LineNr", "fg=8"),
    ("CursorLineNr", "fg=3 bold"),
    // the line the cursor is on, and a soft fade on the lines around it
    ("CursorLine", "bg=+13"),
    ("CursorLineGlow", "bg=+7"),
    ("CursorLineGlow2", "bg=+3"),
    // incremental search: every match, and the one the cursor is on
    ("Search", "fg=0 bg=3"),
    ("IncSearch", "fg=0 bg=5 bold"),
    ("NonText", "fg=8"),
    ("WinSeparator", "fg=8"),
    ("MsgArea", "fg=7"),
    ("KeySeq", "fg=4"),
    ("Prompt", "fg=4"),
    ("StatusLine", "bg=0"),
    ("StatusLineNC", ""),
    ("ModeNormal", "fg=0 bg=4 bold"),
    ("ModeInsert", "fg=0 bg=2 bold"),
    ("ModeVisual", "fg=0 bg=5 bold"),
    ("StatusLineDim", "fg=8"),
    ("StatusLineFile", "fg=4"),
    ("StatusLineBranch", "fg=5"),
    ("Modified", "fg=1"),
    ("TabLineSel", "fg=0 bg=4 bold"),
    ("TabLine", "fg=8"),
    ("DiagnosticError", "fg=1"),
    ("DiagnosticWarn", "fg=3"),
    ("DiagnosticInfo", "fg=4"),
    ("DiagnosticHint", "fg=8"),
    ("DiagnosticOk", "fg=2"),
    ("FloatBorder", "fg=4"),
    ("FloatBorderDim", "fg=8"),
    ("WhichKeyKey", "fg=3 bold"),
    ("WhichKeyGroup", "fg=4"),
    ("PickerPrompt", "fg=4"),
    ("PickerSel", "bg=0 bold"),
    ("PickerDir", "fg=8"),
    ("PickerMatch", "fg=4 bold"),
    ("PickerKey", "fg=3"),
    ("PickerNote", "fg=8"),
    ("HelpKey", "fg=3"),
    ("Pmenu", "bg=0"),
    ("Folded", "fg=8 bold"),
    ("AgentAdd", "fg=2"),
    ("AgentDelete", "fg=8 strikethrough"),
    ("AgentTag", "fg=0 bg=6 bold"),
    ("QueryHead", "fg=8 italic"),
    ("QueryPath", "fg=4"),
    ("QueryHit", "fg=7"),
    ("RefBar", "fg=5"),
    ("RefHead", "fg=5 bold"),
    ("RefText", "fg=7"),
    ("PmenuSel", "fg=0 bg=4"),
    ("PmenuKind", "fg=5"),
    ("PmenuExtra", "fg=8"),
    ("SuggestName", "fg=6"),
    ("SuggestArgs", "fg=8"),
    ("SuggestSel", "bg=0"),
    ("SuggestExample", "fg=2"),
    ("DashboardHeader", "fg=4 bold"),
    ("DashboardKey", "fg=3"),
    ("DashboardFooter", "fg=8"),
    // panels and plugins
    ("Title", "fg=4 bold"),
    ("TitleNC", "fg=8"),
    ("PanelSel", "bg=0 bold"),
    ("Directory", "fg=4"),
    ("File", ""),
    ("Current", "fg=3 bold"),
    ("Accent", "fg=6"),
    ("Muted", "fg=8"),
    ("Added", "fg=2"),
    ("Removed", "fg=1"),
    ("Changed", "fg=3"),
    ("Error", "fg=1"),
    ("Warn", "fg=3"),
    ("GitAdded", "link=Added"),
    ("GitModified", "link=Changed"),
    ("GitDeleted", "link=Removed"),
    ("GitRenamed", "link=Changed"),
    ("GitUntracked", "fg=5"),
    ("GitConflict", "link=Error"),
    ("ExplorerIndent", "link=Muted"),
    ("ExplorerExpander", "link=Muted"),
];

fn color(s: &str) -> Option<HlColor> {
    let s = s.trim().to_lowercase();
    if let Some(p) = s.strip_prefix('+') {
        return p.trim_end_matches('%').parse::<u8>().ok().map(|p| HlColor::Lift(p.min(100)));
    }
    if let Some(h) = s.strip_prefix('#') {
        let v = u32::from_str_radix(h, 16).ok()?;
        return (h.len() == 6).then(|| HlColor::Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8));
    }
    if let Ok(n) = s.parse::<u8>() {
        return Some(HlColor::Index(n));
    }
    let names = ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"];
    let (bright, base) = match s.strip_prefix("bright_").or_else(|| s.strip_prefix("bright")) {
        Some(b) => (8, b),
        None => (0, s.as_str()),
    };
    if let Some(i) = names.iter().position(|n| *n == base) {
        return Some(HlColor::Index(i as u8 + bright));
    }
    match base {
        "gray" | "grey" => Some(HlColor::Index(8)),
        "none" | "default" | "reset" => Some(HlColor::Default),
        _ => None,
    }
}

fn show_color(c: &HlColor) -> String {
    match c {
        HlColor::Index(n) => n.to_string(),
        HlColor::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        HlColor::Default => "none".into(),
        HlColor::Lift(p) => format!("+{p}"),
    }
}

impl Hl {
    /// `fg=3 bg=#202020 bold italic link=Other` laid over this one (`clear` starts afresh).
    pub fn apply(&mut self, args: &str) -> Result<(), String> {
        for t in args.split_whitespace() {
            match t.split_once('=') {
                Some(("fg" | "guifg" | "ctermfg", v)) => self.fg = Some(color(v).ok_or_else(|| format!("not a colour: {v}"))?),
                Some(("bg" | "guibg" | "ctermbg", v)) => self.bg = Some(color(v).ok_or_else(|| format!("not a colour: {v}"))?),
                Some(("link", v)) => self.link = Some(v.to_string()),
                Some((k, _)) => return Err(format!("unknown: {k} (fg= bg= link=)")),
                None => match t {
                    "bold" => self.bold = true,
                    "italic" => self.italic = true,
                    "underline" => self.underline = true,
                    "reverse" => self.reverse = true,
                    "strikethrough" => self.strikethrough = true,
                    "nobold" => self.bold = false,
                    "noitalic" => self.italic = false,
                    "nounderline" => self.underline = false,
                    "noreverse" => self.reverse = false,
                    "nostrikethrough" => self.strikethrough = false,
                    "clear" | "NONE" => *self = Hl::default(),
                    o => return Err(format!("unknown: {o} (bold italic underline reverse strikethrough clear)")),
                },
            }
        }
        Ok(())
    }

    /// Back in `:hi` syntax.
    pub fn describe(&self) -> String {
        if let Some(l) = &self.link {
            return format!("link={l}");
        }
        let mut v = Vec::new();
        if let Some(c) = &self.fg {
            v.push(format!("fg={}", show_color(c)));
        }
        if let Some(c) = &self.bg {
            v.push(format!("bg={}", show_color(c)));
        }
        for (on, n) in [(self.bold, "bold"), (self.italic, "italic"), (self.underline, "underline"), (self.reverse, "reverse"), (self.strikethrough, "strikethrough")] {
            if on {
                v.push(n.into());
            }
        }
        if v.is_empty() { "(plain)".into() } else { v.join(" ") }
    }

    fn from_lua(t: &Table) -> mlua::Result<Hl> {
        let col = |k: &str| -> mlua::Result<Option<HlColor>> {
            Ok(match t.get::<Value>(k)? {
                Value::Integer(n) => Some(HlColor::Index(n.clamp(0, 255) as u8)),
                Value::Number(n) => Some(HlColor::Index(n.clamp(0.0, 255.0) as u8)),
                Value::String(s) => Some(color(&s.to_string_lossy()).ok_or_else(|| mlua::Error::runtime(format!("not a colour: {}", s.to_string_lossy())))?),
                _ => None,
            })
        };
        let flag = |k: &str| t.get::<Option<bool>>(k).map(|b| b.unwrap_or(false));
        Ok(Hl { fg: col("fg")?, bg: col("bg")?, bold: flag("bold")?, italic: flag("italic")?, underline: flag("underline")?, reverse: flag("reverse")?, strikethrough: flag("strikethrough")?, link: t.get("link")? })
    }
}

/// The groups as they start out.
pub fn defaults() -> HashMap<String, Hl> {
    DEFAULTS
        .iter()
        .map(|(n, spec)| {
            let mut h = Hl::default();
            let _ = h.apply(spec);
            (n.to_string(), h)
        })
        .collect()
}

impl Engine {
    /// Changes whenever a group does (the host redraws its colours then).
    pub fn hl_rev(&self) -> u64 {
        self.st.borrow().hl_rev
    }

    /// A group's look, links followed; None for a group nobody defined.
    pub fn highlight(&self, name: &str) -> Option<Hl> {
        let st = self.st.borrow();
        let mut name = name.to_string();
        for _ in 0..10 {
            let h = st.hl.get(&name)?;
            match &h.link {
                Some(l) => name = l.clone(),
                None => return Some(h.clone()),
            }
        }
        None
    }

    /// Every group: (name, as written, as it looks after links).
    pub fn highlights(&self) -> Vec<(String, String, String)> {
        let mut v: Vec<(String, String, String)> = {
            let st = self.st.borrow();
            st.hl.iter().map(|(n, h)| (n.clone(), h.describe(), String::new())).collect()
        };
        for e in v.iter_mut() {
            e.2 = self.highlight(&e.0).map(|h| h.describe()).unwrap_or_default();
        }
        v.sort();
        v
    }

    /// `:hi NAME ARGS`: laid over the group as it is (`link=` replaces it).
    pub fn set_highlight(&mut self, name: &str, args: &str) -> Result<String, String> {
        let mut st = self.st.borrow_mut();
        let mut h = st.hl.get(name).cloned().unwrap_or_default();
        if args.contains("link=") {
            h = Hl::default();
        } else {
            h.link = None;
        }
        h.apply(args)?;
        let shown = h.describe();
        st.hl.insert(name.to_string(), h);
        st.hl_rev += 1;
        Ok(format!("{name} {shown}"))
    }

    pub(crate) fn install_hl(&self, z: &Table) -> mlua::Result<()> {
        let lua = &self.lua;
        // zett.hl(name, { fg=, bg=, bold=, italic=, underline=, reverse=, link= }) sets a group
        // (as nvim_set_hl: the table is the whole look); zett.hl(name) gives it back.
        let st = self.st.clone();
        z.set("hl", lua.create_function(move |lua, (name, spec): (String, Option<Table>)| {
            match spec {
                Some(t) => {
                    let h = Hl::from_lua(&t)?;
                    let mut s = st.borrow_mut();
                    s.hl.insert(name, h);
                    s.hl_rev += 1;
                    Ok(Value::Nil)
                }
                None => {
                    let s = st.borrow();
                    let Some(h) = s.hl.get(&name) else { return Ok(Value::Nil) };
                    let t = lua.create_table()?;
                    let c = |c: &Option<HlColor>| c.as_ref().map(show_color);
                    t.set("fg", c(&h.fg))?;
                    t.set("bg", c(&h.bg))?;
                    t.set("bold", h.bold)?;
                    t.set("italic", h.italic)?;
                    t.set("underline", h.underline)?;
                    t.set("reverse", h.reverse)?;
                    t.set("strikethrough", h.strikethrough)?;
                    t.set("link", h.link.clone())?;
                    Ok(Value::Table(t))
                }
            }
        })?)?;
        let st = self.st.clone();
        z.set("hl_groups", lua.create_function(move |_, ()| {
            let mut v: Vec<String> = st.borrow().hl.keys().cloned().collect();
            v.sort();
            Ok(v)
        })?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs_parse_and_print() {
        let mut h = Hl::default();
        h.apply("fg=red bg=#102030 bold").unwrap();
        assert_eq!(h.fg, Some(HlColor::Index(1)));
        assert_eq!(h.describe(), "fg=1 bg=#102030 bold");
        assert!(h.apply("fg=nonsense").is_err());
        let d = defaults();
        assert_eq!(d["GitAdded"].link.as_deref(), Some("Added"));
    }
}
