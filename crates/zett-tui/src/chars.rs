//! Characters by name, as Emacs's C-x 8: insert-char (any name, or a code: 2192, U+2192, #x2192),
//! the emoji pickers and list (Emacs's groups), what the character at the cursor is called. The
//! tables come from Emacs itself: scripts/gen-charnames.sh.

use super::*;

const NAMES: &str = include_str!("data/charnames.txt");
const EMOJI: &str = include_str!("data/emoji.txt");

/// Every (name, character).
fn names() -> impl Iterator<Item = (&'static str, char)> {
    NAMES.lines().filter_map(|l| {
        let (n, hex) = l.split_once('\t')?;
        Some((n, char::from_u32(u32::from_str_radix(hex, 16).ok()?)?))
    })
}

/// A character's name (None for one the table leaves out, as CJK ideographs).
pub(crate) fn name_of(c: char) -> Option<&'static str> {
    static BY_CHAR: std::sync::OnceLock<std::collections::HashMap<char, &'static str>> = std::sync::OnceLock::new();
    BY_CHAR.get_or_init(|| names().map(|(n, c)| (c, n)).collect()).get(&c).copied()
}

/// What was typed at C-x 8 RET: a name from the list, or a code in hex (U+, #x) or decimal (#).
pub(crate) fn parse_char(s: &str) -> Option<char> {
    let t = s.trim();
    if let Some(c) = names().find(|(n, _)| n.eq_ignore_ascii_case(t)).map(|(_, c)| c) {
        return Some(c);
    }
    let code = if let Some(h) = t.strip_prefix("U+").or_else(|| t.strip_prefix("u+")).or_else(|| t.strip_prefix("#x")) {
        u32::from_str_radix(h, 16).ok()
    } else if let Some(d) = t.strip_prefix('#') {
        d.parse().ok()
    } else {
        u32::from_str_radix(t, 16).ok()
    };
    code.and_then(char::from_u32)
}

/// An emoji's name: its characters' names, the joiners and variation selectors left out.
fn emoji_name(e: &str) -> String {
    let parts: Vec<&str> = e.chars().filter(|c| !matches!(*c as u32, 0x200d | 0xfe0f | 0xfe0e)).filter_map(name_of).collect();
    parts.join(", ").to_lowercase()
}

/// Every emoji with its group: (emoji, "Smileys / smiling").
fn emoji() -> impl Iterator<Item = (&'static str, &'static str)> {
    EMOJI.lines().filter_map(|l| l.split_once('\t')).flat_map(|(g, list)| list.split(' ').filter(|e| !e.is_empty()).map(move |e| (e, g)))
}

fn recent_file() -> PathBuf {
    doom::state_dir().join("emoji-recent")
}

impl App {
    /// The candidates for the C-x 8 prompts: "chars" (every name) or "emoji" (or "emoji-recent").
    pub(crate) fn char_items(&self, what: &str) -> Vec<Cand> {
        match what {
            "chars" => names().map(|(n, c)| Cand::noted(n, format!("{c}  U+{:04X}", c as u32))).collect(),
            "emoji-recent" => std::fs::read_to_string(recent_file()).unwrap_or_default().lines().map(|e| Cand::noted(format!("{e}  {}", emoji_name(e)), "")).collect(),
            _ => emoji().map(|(e, g)| Cand::noted(format!("{e}  {}", emoji_name(e)), g)).collect(),
        }
    }

    /// Text at the cursor (one undo step), the cursor after it.
    fn insert_here(&mut self, s: &str) {
        let before = self.docs[self.cur].ed.cursor().line;
        let ed = &self.docs[self.cur].ed;
        let at = ed.buf.to_char(ed.cursor());
        self.doc().ed.replace_text(at, at, s);
        self.doc().touched(before);
        self.edit_at = Instant::now();
    }

    /// C-x 8 RET's answer.
    pub(crate) fn insert_char(&mut self, typed: &str) {
        match parse_char(typed) {
            Some(c) => self.insert_here(&c.to_string()),
            None => self.say(format!("Invalid character: {typed}")),
        }
    }

    /// An emoji picked (the line starts with it), and kept among the recent ones.
    pub(crate) fn insert_emoji(&mut self, line: &str) {
        let Some(e) = line.split_whitespace().next() else { return };
        self.insert_here(e);
        let mut recent: Vec<String> = std::fs::read_to_string(recent_file()).unwrap_or_default().lines().filter(|l| *l != e).map(String::from).collect();
        recent.insert(0, e.to_string());
        recent.truncate(40);
        let _ = std::fs::create_dir_all(doom::state_dir());
        let _ = std::fs::write(recent_file(), recent.join("\n") + "\n");
    }

    /// C-x 8 e l: every emoji by group, in a buffer of its own.
    pub(crate) fn emoji_list(&mut self) {
        let text: String = EMOJI.lines().filter_map(|l| l.split_once('\t')).map(|(g, list)| format!("{g}\n  {list}\n\n")).collect();
        self.vc_buffer("*Emoji*", text, "Plain Text");
    }

    /// C-x 8 e d (and C-u C-x =): the character at the cursor, by name and code.
    pub(crate) fn describe_char(&mut self) {
        let ed = &self.docs[self.cur].ed;
        let at = ed.buf.to_char(ed.cursor());
        let Some(c) = ed.buf.char_at(at) else { return self.say("End of buffer") };
        let name = name_of(c).map(str::to_string).unwrap_or_else(|| if c == '\n' { "LINE FEED".into() } else { "(no name)".into() });
        let utf8: Vec<String> = c.to_string().bytes().map(|b| format!("{b:02X}")).collect();
        self.say(format!("{c}  {name}  U+{:04X}  UTF-8 {}", c as u32, utf8.join(" ")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_codes() {
        assert_eq!(parse_char("rightwards arrow"), Some('→'));
        assert_eq!(parse_char("U+20AC"), Some('€'));
        assert_eq!(parse_char("#x2192"), Some('→'));
        assert_eq!(parse_char("#955"), Some('λ'));
        assert_eq!(name_of('€'), Some("EURO SIGN"));
        assert_eq!(emoji_name("👍🏽"), "thumbs up sign, emoji modifier fitzpatrick type-4");
        assert!(emoji().count() > 1500, "Emacs's emoji groups");
    }
}
