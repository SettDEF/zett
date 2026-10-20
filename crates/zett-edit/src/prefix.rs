//! The numeric argument (Emacs's prefix argument): `C-u` (4, `C-u C-u` 16), `C-u 12`, `M-5`,
//! `M-- ` / `C-u -` (negative). Typed before a command, which then runs that many times (or,
//! negative, the other way). The keys only point at the commands below; this is the state.

/// The commands that build an argument (M-x, C-h f); keys point at them in the keymap.
pub const COMMANDS: &[(&str, &str)] = &[
    ("universal-argument", "Begin a numeric argument for the next command (4; again: ×4; digits: that number)"),
    ("digit-argument", "Part of a numeric argument: the digit of the key that ran it (M-0 … M-9)"),
    ("negative-argument", "Begin a negative numeric argument for the next command (M--)"),
];

pub fn is_command(name: &str) -> bool {
    COMMANDS.iter().any(|(n, _)| *n == name)
}

/// An argument being typed. Empty: none (the command's default, 1).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrefixArg {
    active: bool,
    /// C-u pressed this many times (before any digit)
    times: u32,
    digits: Option<i64>,
    negative: bool,
}

impl PrefixArg {
    /// Typing one: plain digits and `-` go to it instead of the buffer.
    pub fn active(&self) -> bool {
        self.active
    }

    pub fn universal(&mut self) {
        self.active = true;
        if self.digits.is_none() && !self.negative {
            self.times += 1;
        }
    }

    pub fn digit(&mut self, d: u32) {
        self.active = true;
        let d = d.min(9) as i64;
        self.digits = Some(self.digits.unwrap_or(0).saturating_mul(10).saturating_add(d));
    }

    /// `-` right after C-u (or M--): the argument is negative.
    pub fn minus(&mut self) {
        self.active = true;
        if self.digits.is_none() {
            self.negative = !self.negative;
        }
    }

    /// A `-` typed now belongs to the argument (before its digits), not to the buffer.
    pub fn accepts_minus(&self) -> bool {
        self.active && self.digits.is_none()
    }

    /// Takes the value for the command about to run (and clears it): None without an argument.
    pub fn take(&mut self) -> Option<i64> {
        let v = self.value();
        *self = PrefixArg::default();
        v
    }

    /// What it is now: digits if typed, else 4 per C-u; `-` alone is -1.
    pub fn value(&self) -> Option<i64> {
        if !self.active {
            return None;
        }
        let n = match self.digits {
            Some(d) => d,
            // `C-u -` is -1, as in Emacs: the minus replaces the 4
            None if self.times > 0 && !self.negative => 4i64.saturating_pow(self.times),
            None => 1,
        };
        Some(if self.negative { -n } else { n })
    }

    /// The echo area while it is typed: `C-u 12-`, `C-u C-u-`, `C-u --`.
    pub fn prompt(&self) -> String {
        let mut s = "C-u".to_string();
        if self.digits.is_none() {
            for _ in 1..self.times {
                s.push_str(" C-u");
            }
        }
        if self.negative {
            s.push_str(" -");
        }
        if let Some(d) = self.digits {
            s.push_str(&format!(" {d}"));
        }
        s.push('-');
        s
    }
}

/// A command with a negative argument runs its opposite (backward-word for forward-word …);
/// None: it has no opposite and takes the count as it is.
pub fn opposite(name: &str) -> Option<&'static str> {
    const PAIRS: &[(&str, &str)] = &[
        ("forward-char", "backward-char"),
        ("next-line", "previous-line"),
        ("forward-word", "backward-word"),
        ("forward-paragraph", "backward-paragraph"),
        ("delete-char", "delete-backward-char"),
        ("kill-word", "backward-kill-word"),
    ];
    PAIRS.iter().find_map(|(a, b)| if *a == name { Some(*b) } else if *b == name { Some(*a) } else { None })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_u_is_four_and_multiplies() {
        let mut a = PrefixArg::default();
        assert_eq!(a.value(), None);
        a.universal();
        assert_eq!((a.value(), a.prompt().as_str()), (Some(4), "C-u-"));
        a.universal();
        assert_eq!((a.value(), a.prompt().as_str()), (Some(16), "C-u C-u-"));
        assert_eq!(a.take(), Some(16));
        assert_eq!(a.take(), None, "taking clears it");
    }

    #[test]
    fn digits_replace_the_four() {
        let mut a = PrefixArg::default();
        a.universal();
        a.digit(1);
        a.digit(2);
        assert_eq!((a.value(), a.prompt().as_str()), (Some(12), "C-u 12-"));
        // M-5 alone starts one too
        let mut m = PrefixArg::default();
        m.digit(5);
        assert_eq!(m.value(), Some(5));
    }

    #[test]
    fn minus_makes_it_negative() {
        let mut a = PrefixArg::default();
        a.minus();
        assert_eq!(a.value(), Some(-1));
        a.digit(3);
        assert_eq!((a.value(), a.prompt().as_str()), (Some(-3), "C-u - 3-"));
        let mut u = PrefixArg::default();
        u.universal();
        u.minus();
        assert_eq!(u.value(), Some(-1), "C-u - is -1");
    }

    #[test]
    fn opposites_both_ways() {
        assert_eq!(opposite("forward-word"), Some("backward-word"));
        assert_eq!(opposite("backward-char"), Some("forward-char"));
        assert_eq!(opposite("yank"), None);
    }
}
