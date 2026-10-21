//! Keys, and Vim's notation for them: `ihello<Esc>`, `<C-r>`, `<CR>`, `<lt>`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Code {
    Char(char),
    Esc,
    Enter,
    Backspace,
    Delete,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    /// F1 … F12
    F(u8),
    Help,
    Undo,
    /// a mouse button pressed where the window last said (Editor::set_mouse)
    Mouse(MouseButton),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: Code,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Key {
    pub const fn ch(c: char) -> Self {
        Self { code: Code::Char(c), ctrl: false, alt: false, shift: false }
    }
    pub const fn code(code: Code) -> Self {
        Self { code, ctrl: false, alt: false, shift: false }
    }
    pub const fn ctrl(c: char) -> Self {
        Self { code: Code::Char(c), ctrl: true, alt: false, shift: false }
    }
    /// The key in Vim notation (`x`, `<C-a>`, `<Esc>`), as macros keep it.
    pub fn notation(&self) -> String {
        let base = match self.code {
            Code::Char('<') => "lt".to_string(),
            Code::Char(c) if !self.ctrl && !self.alt => return c.to_string(),
            Code::Char(c) => c.to_string(),
            Code::Esc => "Esc".into(),
            Code::Enter => "CR".into(),
            Code::Backspace => "BS".into(),
            Code::Delete => "Del".into(),
            Code::Tab => "Tab".into(),
            Code::Left => "Left".into(),
            Code::Right => "Right".into(),
            Code::Up => "Up".into(),
            Code::Down => "Down".into(),
            Code::Home => "Home".into(),
            Code::End => "End".into(),
            Code::PageUp => "PageUp".into(),
            Code::PageDown => "PageDown".into(),
            Code::Insert => "Insert".into(),
            Code::F(n) => format!("F{n}"),
            Code::Help => "Help".into(),
            Code::Undo => "Undo".into(),
            Code::Mouse(MouseButton::Left) => "LeftMouse".into(),
            Code::Mouse(MouseButton::Middle) => "MiddleMouse".into(),
            Code::Mouse(MouseButton::Right) => "RightMouse".into(),
            #[allow(unreachable_patterns)]
            _ => "Nop".into(),
        };
        let m = format!("{}{}{}", if self.shift { "S-" } else { "" }, if self.ctrl { "C-" } else { "" }, if self.alt { "A-" } else { "" });
        format!("<{m}{base}>")
    }

    /// The plain character this key types, if any.
    pub fn char(&self) -> Option<char> {
        match self.code {
            Code::Char(c) if !self.ctrl && !self.alt => Some(c),
            _ => None,
        }
    }
}

/// Parses Vim key notation. Unknown `<…>` sequences are typed literally.
pub fn parse(s: &str) -> Vec<Key> {
    let mut out = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<' {
            if let Some(end) = chars[i + 1..].iter().position(|c| *c == '>') {
                let name: String = chars[i + 1..i + 1 + end].iter().collect();
                if let Some(k) = named(&name) {
                    out.push(k);
                    i += end + 2;
                    continue;
                }
            }
        }
        out.push(Key::ch(chars[i]));
        i += 1;
    }
    out
}

/// As `parse`, but None when a `<Name>` is not a key zett knows (F-keys, PageUp…): the parity probe
/// must not count a key it cannot even write.
pub fn parse_strict(s: &str) -> Option<Vec<Key>> {
    let mut rest = s;
    while let Some(a) = rest.find('<') {
        let after = &rest[a + 1..];
        match after.find('>') {
            Some(b) if b > 0 && after[..b].chars().all(|c| c.is_ascii_alphanumeric() || c == '-') => {
                named(&after[..b])?;
                rest = &after[b + 1..];
            }
            _ => rest = after,
        }
    }
    Some(parse(s))
}

fn named(name: &str) -> Option<Key> {
    let lower = name.to_ascii_lowercase();
    if lower == "s--" {
        return Some(Key { code: Code::Char('-'), ctrl: false, alt: false, shift: true });
    }
    let (mods, base) = match lower.rsplit_once('-') {
        Some((m, b)) if !b.is_empty() && m.chars().all(|c| "cams-".contains(c)) => (m.to_string(), b.to_string()),
        _ => (String::new(), lower.clone()),
    };
    let code = match base.as_str() {
        "esc" => Code::Esc,
        "cr" | "enter" | "return" => Code::Enter,
        "bs" | "backspace" => Code::Backspace,
        "del" | "delete" => Code::Delete,
        "tab" => Code::Tab,
        "left" => Code::Left,
        "right" => Code::Right,
        "up" => Code::Up,
        "down" => Code::Down,
        "home" => Code::Home,
        "end" => Code::End,
        "space" => Code::Char(' '),
        "lt" => Code::Char('<'),
        "pageup" => Code::PageUp,
        "pagedown" => Code::PageDown,
        "insert" | "ins" => Code::Insert,
        "help" => Code::Help,
        "undo" => Code::Undo,
        "nl" => return Some(Key { code: Code::Char('j'), ctrl: true, alt: false, shift: mods.contains('s') }),
        "leftmouse" => Code::Mouse(MouseButton::Left),
        "middlemouse" => Code::Mouse(MouseButton::Middle),
        "rightmouse" => Code::Mouse(MouseButton::Right),
        b if b.len() >= 2 && b.starts_with('f') && b[1..].parse::<u8>().is_ok_and(|n| (1..=12).contains(&n)) => Code::F(b[1..].parse().unwrap()),
        "bar" => Code::Char('|'),
        b if b.chars().count() == 1 && !mods.is_empty() => {
            // keep the case the user wrote for <C-R> vs <C-r>: Vim treats them the same
            Code::Char(name.chars().last().unwrap().to_ascii_lowercase())
        }
        _ => return None,
    };
    Some(Key { code, ctrl: mods.contains('c'), alt: mods.contains('a') || mods.contains('m'), shift: mods.contains('s') })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notation() {
        assert_eq!(parse("ix<Esc>"), vec![Key::ch('i'), Key::ch('x'), Key::code(Code::Esc)]);
        assert_eq!(parse("<C-r>"), vec![Key::ctrl('r')]);
        assert_eq!(parse("<C-R>"), vec![Key::ctrl('r')]);
        assert_eq!(parse("a<lt>b"), vec![Key::ch('a'), Key::ch('<'), Key::ch('b')]);
        assert_eq!(parse("<CR>"), vec![Key::code(Code::Enter)]);
        // not a key name: typed as is
        assert_eq!(parse("<x"), vec![Key::ch('<'), Key::ch('x')]);
        assert_eq!(parse("<foo>").len(), 5);
    }
}
