//! Highlight groups as terminal styles, cached until a group changes (zett.hl, :hi).

use ratatui::style::{Color, Modifier, Style};
use std::cell::RefCell;
use std::collections::HashMap;
use zett_script::hl::{Hl, HlColor};

thread_local! {
    static CACHE: RefCell<(u64, HashMap<String, Style>)> = RefCell::new((0, HashMap::new()));
}

pub(crate) fn color(c: HlColor) -> Color {
    match c {
        HlColor::Index(n) => Color::Indexed(n),
        HlColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
        HlColor::Default => Color::Reset,
        HlColor::Lift(p) => {
            let ((br, bg, bb), (fr, fg, fb)) = theme();
            let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * p as f32 / 100.0).round() as u8;
            Color::Rgb(mix(br, fr), mix(bg, fg), mix(bb, fb))
        }
    }
}

/// The theme's background and foreground, from tinct's terminal colours (what zash reads too).
fn theme() -> ((u8, u8, u8), (u8, u8, u8)) {
    static T: std::sync::OnceLock<((u8, u8, u8), (u8, u8, u8))> = std::sync::OnceLock::new();
    *T.get_or_init(|| {
        let home = std::env::var("HOME").unwrap_or_default();
        let src = std::fs::read_to_string(format!("{home}/.config/wezterm/tinct_colors.lua")).unwrap_or_default();
        let find = |key: &str| {
            src.lines().map(|l| l.trim()).find(|l| l.starts_with(key) && l[key.len()..].trim_start().starts_with('=')).and_then(|l| l.split('"').nth(1)).and_then(|h| {
                let v = u32::from_str_radix(h.trim_start_matches('#'), 16).ok()?;
                Some(((v >> 16) as u8, (v >> 8) as u8, v as u8))
            })
        };
        (find("background").unwrap_or((29, 32, 33)), find("foreground").unwrap_or((221, 199, 161)))
    })
}

pub(crate) fn style(h: &Hl) -> Style {
    let mut s = Style::default();
    if let Some(c) = h.fg {
        s = s.fg(color(c));
    }
    if let Some(c) = h.bg {
        s = s.bg(color(c));
    }
    for (on, m) in [(h.bold, Modifier::BOLD), (h.italic, Modifier::ITALIC), (h.underline, Modifier::UNDERLINED), (h.reverse, Modifier::REVERSED)] {
        if on {
            s = s.add_modifier(m);
        }
    }
    s
}

/// Takes the engine's groups when they changed; true when they did (the syntax theme follows).
pub(crate) fn sync(e: &zett_script::Engine) -> bool {
    let rev = e.hl_rev();
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.0 == rev {
            return false;
        }
        c.1 = e.highlights().into_iter().filter_map(|(n, _, _)| e.highlight(&n).map(|h| (n, style(&h)))).collect();
        c.0 = rev;
        true
    })
}

/// A group's style (the built-in look before any engine has been seen, as in tests).
pub(crate) fn hl(name: &str) -> Style {
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.1.is_empty() {
            let d = zett_script::hl::defaults();
            let resolve = |n: &str| {
                let mut n = n.to_string();
                for _ in 0..10 {
                    match d.get(&n) {
                        Some(h) if h.link.is_some() => n = h.link.clone().unwrap(),
                        Some(h) => return Some(style(h)),
                        None => return None,
                    }
                }
                None
            };
            c.1 = d.keys().filter_map(|n| resolve(n).map(|s| (n.clone(), s))).collect();
        }
        c.1.get(name).copied().unwrap_or_default()
    })
}

/// A group's foreground as a colour (for the syntax theme).
pub(crate) fn fg(name: &str) -> Option<Color> {
    hl(name).fg
}

/// A new engine (the config reloaded): its groups are taken at the next frame.
pub(crate) fn reset() {
    CACHE.with(|c| c.borrow_mut().0 = u64::MAX);
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_current_line_is_lifted() {
        assert!(matches!(super::hl("CursorLine").bg, Some(ratatui::style::Color::Rgb(..))));
    }
}
