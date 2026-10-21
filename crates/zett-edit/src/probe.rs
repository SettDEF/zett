//! Does the Vim profile do something with a key? For the parity pages: each entry of Neovim's own
//! index of normal-mode keys is written out as keys (`d{motion}` → `dw`, `CTRL-W J` → `<C-w>J`)
//! and fed into an editor set up so most keys have something to act on (text around the cursor, a
//! yank, a change to undo, a search done). A key counts when anything changed: text, cursor, mode,
//! a pending operator, the register, the mark, undo, the view.

use crate::Editor;

const TEXT: &str = "alpha Beta gamma 7\n  delta (Epsilon 42) {zeta} \"eta\" 'theta' [iota] <tag>\nkappa lambda mu\nnu xi omicron\n\npi rho sigma\n";

/// Neovim's notation for a normal-mode entry → keys to feed. `tag` tells text objects (`v_ab`).
pub fn expand(written: &str, tag: &str) -> String {
    let mut w = written.trim().to_string();
    // optional register and count prefixes
    for p in ["[\"x]", "[count]"] {
        w = w.replace(p, "");
    }
    let subs = [
        ("{motion}", "w"),
        ("{char}", "a"),
        ("{char1}", "a"),
        ("{char2}", "b"),
        ("{filter}", "sort<CR>"),
        ("{pattern}", "beta"),
        ("{count}", "2"),
        ("{height}", "5"),
        ("{width}", "5"),
        ("{register}", "a"),
        ("{a-z}", "a"),
        ("{A-Za-z}", "a"),
        ("{a-zA-Z}", "a"),
        ("{0-9a-zA-Z\"}", "a"),
        ("{a-z0-9}", "a"),
        ("{a-zA-Z0-9}", "a"),
        ("{a-zA-Z0-9.%#:-\"}", "a"),
        ("{0-9a-z\".%#:-}", "a"),
    ];
    for (a, b) in subs {
        w = w.replace(a, b);
    }
    // any other {…} placeholder: a letter
    while let (Some(a), Some(b)) = (w.find('{'), w.find('}')) {
        if b < a {
            break;
        }
        w.replace_range(a..=b, "a");
    }
    // CTRL-X → <C-x>, words apart are keys apart
    let mut out = String::new();
    for part in w.split(' ').filter(|p| !p.is_empty()) {
        if let Some(k) = part.strip_prefix("CTRL-") {
            let (key, rest) = k.split_at(k.chars().next().map(|c| c.len_utf8()).unwrap_or(0));
            out.push_str(&format!("<C-{}>{rest}", key.to_lowercase()));
        } else if part == "<NL>" {
            out.push_str("<C-j>");
        } else {
            out.push_str(part);
        }
    }
    // a text object: after an operator
    if tag.starts_with("v_") {
        out = format!("d{out}");
    }
    out
}

impl Editor {
    /// Everything a key could change, as one comparable string.
    fn probe_state(&self) -> String {
        format!(
            "{}|{}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{}|{}|{:?}|{:?}|{}|{:?}|{:?}",
            self.buf.text(),
            self.cursor,
            self.mode,
            self.count,
            self.op,
            self.op_count,
            self.prefix,
            self.reg,
            self.undo.cur,
            self.undo.nodes.len(),
            self.last_find,
            self.mark,
            self.last_search,
            self.isearch.is_some(),
            self.view,
        ) + &self.vim_probe()
    }
}

/// The state after `keys`, from the prepared start; None when keys are still waiting (a prefix, an
/// operator, a count): those did nothing yet.
fn settled(keys: &[crate::keys::Key]) -> Option<String> {
    let mut e = Editor::from_str(TEXT);
    e.prepare();
    e.last_search = "beta".into();
    e.set_cursor(crate::Pos { line: 1, col: 10 });
    for k in keys {
        e.key(*k);
    }
    (e.prefix.is_none() && e.op.is_none() && e.count.is_none()).then(|| e.probe_state())
}

/// The state after `keys`, from the prepared start.
fn after(keys: &[crate::keys::Key]) -> String {
    let mut e = Editor::from_str(TEXT);
    e.prepare();
    e.last_search = "beta".into();
    e.set_cursor(crate::Pos { line: 1, col: 10 });
    for k in keys {
        e.key(*k);
    }
    e.probe_state()
}

/// Whether the keys do anything in the Vim profile: the first key must make the difference (fed
/// without it, what follows ends elsewhere), so `ma` with no marks is not counted because `a`
/// alone starts inserting.
pub fn handles(keys: &str) -> bool {
    let Some(ks) = crate::keys::parse_strict(keys) else { return false };
    if ks.is_empty() {
        return false;
    }
    // a count: only waits, so ask whether it was taken
    if !keys.is_empty() && keys.chars().all(|c| c.is_ascii_digit()) {
        let mut e = Editor::from_str(TEXT);
        e.feed(keys);
        return e.count.is_some();
    }
    // and something must have happened at all: a key swallowed by a prefix (`gX`) changes nothing
    let Some(full) = settled(&ks) else { return false };
    // different from every shorter tail: `guw` where g only swallows the u ends as `w` alone would
    (1..=ks.len()).all(|i| full != after(&ks[i..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neovims_notation_becomes_keys() {
        assert_eq!(expand("[\"x]d{motion}", "d"), "dw");
        assert_eq!(expand("CTRL-W J", "CTRL-W_J"), "<C-w>J");
        assert_eq!(expand("CTRL-A", "CTRL-A"), "<C-a>");
        assert_eq!(expand("ib", "v_ib"), "dib");
        assert_eq!(expand("m{A-Za-z}", "m"), "ma");
    }

    #[test]
    fn known_keys_count_and_nonsense_does_not() {
        for k in ["w", "dw", "x", "p", "u", "gg", "dib", "fa", "ciwX<Esc>", "j"] {
            assert!(handles(k), "{k} should do something");
        }
        assert!(!handles("<F12>"), "nothing is bound to F12");
        assert!(!handles("gX"), "an unknown g command does nothing");
        assert!(handles("ma"), "marks are kept");
        assert!(!handles("guw") || handles("gUw"), "g swallowing a key and w moving is not a command");
        assert!(handles("/beta<CR>"), "/ searches");
    }
}

impl Editor {
    /// Something for every key to act on: a change to undo, a yank to put, a macro to replay, a
    /// selection to bring back, an insert to return to, an f to repeat, a search to go on with.
    fn prepare(&mut self) {
        self.feed("jx");
        self.feed("qqlq");
        self.feed("vl<Esc>");
        self.feed("A!<Esc>");
        self.feed("0fa");
        self.feed("yiw");
    }
}
