//! Spelling: the words of hunspell dictionaries (/usr/share/hunspell/<lang>.dic, their stems with
//! the common English endings) or a plain word list (/usr/share/dict/words, a file in
//! ~/.local/share/zett/spell/), and the user's own lists as Vim keeps them
//! (~/.local/share/zett/spell/<lang>.add: a good word per line, a wrong one as "word/!").

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub struct Spell {
    pub lang: String,
    dict: HashSet<String>,
    good: HashSet<String>,
    wrong: HashSet<String>,
    // zG zW: for this session only
    session_good: HashSet<String>,
    session_wrong: HashSet<String>,
}

fn dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{home}/.local/share"))).join("zett/spell")
}

fn spell() -> &'static Mutex<Spell> {
    static S: OnceLock<Mutex<Spell>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Spell::load(&std::env::var("ZETT_SPELLLANG").unwrap_or_else(|_| "en_US".into()))))
}

/// Works with the spell checker (loaded on first use).
pub fn with<R>(f: impl FnOnce(&mut Spell) -> R) -> R {
    f(&mut spell().lock().unwrap())
}

impl Spell {
    pub fn load(lang: &str) -> Spell {
        let mut dict = HashSet::new();
        let read = |p: PathBuf, dict: &mut HashSet<String>| {
            if let Ok(t) = std::fs::read_to_string(&p) {
                // a .dic starts with the number of words; each is word/FLAGS
                for l in t.lines().skip(usize::from(p.extension().is_some_and(|e| e == "dic"))) {
                    let w = l.split('/').next().unwrap_or("").trim();
                    if !w.is_empty() {
                        dict.insert(w.to_lowercase());
                    }
                }
            }
        };
        for p in [dir().join(format!("{lang}.dic")), PathBuf::from(format!("/usr/share/hunspell/{lang}.dic")), dir().join(format!("{lang}.words")), PathBuf::from("/usr/share/dict/words")] {
            read(p, &mut dict);
        }
        let mut s = Spell { lang: lang.to_string(), dict, good: HashSet::new(), wrong: HashSet::new(), session_good: HashSet::new(), session_wrong: HashSet::new() };
        if let Ok(t) = std::fs::read_to_string(s.add_file()) {
            for l in t.lines().map(str::trim).filter(|l| !l.is_empty()) {
                match l.strip_suffix("/!") {
                    Some(w) => s.wrong.insert(w.to_lowercase()),
                    None => s.good.insert(l.to_lowercase()),
                };
            }
        }
        s
    }

    /// A spell checker over these words alone (tests).
    pub fn from_words(words: &[&str]) -> Spell {
        Spell { lang: "test".into(), dict: words.iter().map(|w| w.to_lowercase()).collect(), good: HashSet::new(), wrong: HashSet::new(), session_good: HashSet::new(), session_wrong: HashSet::new() }
    }

    fn add_file(&self) -> PathBuf {
        dir().join(format!("{}.add", self.lang.split('_').next().unwrap_or("en")))
    }

    /// There is a dictionary to check against.
    pub fn available(&self) -> bool {
        !self.dict.is_empty()
    }

    /// The word is spelled right (numbers, single letters and words with digits count as right).
    pub fn ok(&self, word: &str) -> bool {
        let w = word.to_lowercase();
        if w.chars().count() < 2 || w.chars().any(|c| c.is_ascii_digit()) {
            return true;
        }
        if self.wrong.contains(&w) || self.session_wrong.contains(&w) {
            return false;
        }
        if self.good.contains(&w) || self.session_good.contains(&w) || self.dict.contains(&w) {
            return true;
        }
        // a stem with an ending (hunspell lists stems)
        let stems = [("'s", ""), ("ies", "y"), ("ied", "y"), ("es", ""), ("s", ""), ("ed", ""), ("ed", "e"), ("d", ""), ("ing", ""), ("ing", "e"), ("ly", ""), ("er", ""), ("er", "e"), ("est", ""), ("est", "e"), ("ness", ""), ("ment", "")];
        stems.iter().any(|(end, add)| w.strip_suffix(end).is_some_and(|s| s.len() > 1 && self.dict.contains(&format!("{s}{add}"))))
    }

    /// Up to `n` words near `word`, closest first, in the word's case (Capitalised stays so).
    pub fn suggest(&self, word: &str, n: usize) -> Vec<String> {
        let w = word.to_lowercase();
        let len = w.chars().count();
        let mut found: Vec<(usize, &String)> = self.dict.iter().chain(self.good.iter()).filter(|d| d.chars().count().abs_diff(len) <= 2).filter_map(|d| {
            let k = distance(&w, d);
            (k <= 2).then_some((k, d))
        }).collect();
        found.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.len().abs_diff(len).cmp(&b.1.len().abs_diff(len))).then(a.1.cmp(b.1)));
        let cap = word.chars().next().is_some_and(char::is_uppercase);
        found.into_iter().take(n).map(|(_, d)| if cap { let mut c = d.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() } else { d.clone() }).collect()
    }

    /// zg zw (to the file), zG zW (this session), and their undo (zug zuw zuG zuW).
    pub fn mark(&mut self, word: &str, good: bool, session: bool, undo: bool) {
        let w = word.to_lowercase();
        let set = match (good, session) {
            (true, false) => &mut self.good,
            (false, false) => &mut self.wrong,
            (true, true) => &mut self.session_good,
            (false, true) => &mut self.session_wrong,
        };
        if undo {
            set.remove(&w);
        } else {
            set.insert(w.clone());
            // a word is not on both lists
            if good { self.wrong.remove(&w); self.session_wrong.remove(&w); } else { self.good.remove(&w); self.session_good.remove(&w); }
        }
        if !session && self.lang != "test" {
            let mut lines: Vec<String> = self.good.iter().cloned().collect();
            lines.extend(self.wrong.iter().map(|w| format!("{w}/!")));
            lines.sort();
            let _ = std::fs::create_dir_all(dir());
            let _ = std::fs::write(self.add_file(), lines.join("\n") + "\n");
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems_lists_and_suggestions() {
        let mut s = Spell::from_words(&["house", "walk", "happy", "city", "the", "quick"]);
        assert!(s.ok("houses") && s.ok("walked") && s.ok("walking") && s.ok("cities") && s.ok("The"));
        assert!(!s.ok("hous") && !s.ok("qiuck"));
        assert_eq!(s.suggest("qiuck", 3), vec!["quick"]);
        assert_eq!(s.suggest("Hous", 1), vec!["House"], "the case stays");
        s.mark("zett", true, true, false);
        assert!(s.ok("zett"));
        s.mark("zett", true, true, true);
        assert!(!s.ok("zett"));
        s.mark("walk", false, true, false);
        assert!(!s.ok("walk"), "marked wrong wins over the dictionary");
    }
}
