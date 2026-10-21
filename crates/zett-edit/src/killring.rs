//! Emacs's kill ring: every kill and copy, newest last. One ring is shared by every buffer of a
//! front end (`Editor::use_kill_ring`), so text killed in one buffer yanks in another.

use std::sync::{Arc, Mutex};

/// The most kills kept; the oldest goes first.
const KEEP: usize = 120;

#[derive(Debug, Default)]
pub struct KillRing {
    items: Vec<String>,
    /// goes up with every kill or copy, so a front end can tell when to copy to the clipboard
    kills: u64,
    /// the last killed rectangle (C-x r k, C-x r y): its lines, shared like the ring
    pub rect: Vec<String>,
}

/// A ring several editors hold at once.
pub type SharedKillRing = Arc<Mutex<KillRing>>;

impl KillRing {
    pub fn shared() -> SharedKillRing {
        Arc::new(Mutex::new(KillRing::default()))
    }

    /// A new kill, or (`append`) more of the last one: consecutive kills join, as in Emacs;
    /// `before` puts the text in front (a backward kill).
    pub fn push(&mut self, t: String, append: bool, before: bool) {
        match self.items.last_mut() {
            Some(last) if append => {
                if before {
                    last.insert_str(0, &t);
                } else {
                    last.push_str(&t);
                }
            }
            _ => {
                self.items.push(t);
                if self.items.len() > KEEP {
                    self.items.remove(0);
                }
            }
        }
        self.kills += 1;
    }

    /// Text copied outside zett: it joins the ring unless it already is the newest kill (Emacs'
    /// interprogram paste). Not counted as a kill, so it is not copied back out. True when added.
    pub fn offer(&mut self, text: String) -> bool {
        if text.is_empty() || self.last() == Some(text.as_str()) {
            return false;
        }
        self.items.push(text);
        if self.items.len() > KEEP {
            self.items.remove(0);
        }
        true
    }

    pub fn last(&self) -> Option<&str> {
        self.items.last().map(String::as_str)
    }

    pub fn get(&self, i: usize) -> Option<&str> {
        self.items.get(i).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn kill_count(&self) -> u64 {
        self.kills
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consecutive_kills_join() {
        let mut r = KillRing::default();
        r.push("one ".into(), false, false);
        r.push("two".into(), true, false);
        r.push("zero ".into(), true, true);
        assert_eq!(r.last(), Some("zero one two"));
        assert_eq!(r.len(), 1);
        assert_eq!(r.kill_count(), 3);
    }

    #[test]
    fn outside_text_joins_once_and_is_not_a_kill() {
        let mut r = KillRing::default();
        assert!(r.offer("copied".into()));
        assert!(!r.offer("copied".into()));
        assert_eq!(r.len(), 1);
        assert_eq!(r.kill_count(), 0);
    }

    #[test]
    fn keeps_the_newest() {
        let mut r = KillRing::default();
        for i in 0..KEEP + 5 {
            r.push(i.to_string(), false, false);
        }
        assert_eq!(r.len(), KEEP);
        assert_eq!(r.get(0), Some("5"));
    }
}
