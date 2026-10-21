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
    /// spellfile: where zg and zw write (None: the language's .add file)
    file: Option<PathBuf>,
}

fn dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{home}/.local/share"))).join("zett/spell")
}

static S: OnceLock<Mutex<Spell>> = OnceLock::new();
/// spelllang and spellfile as last set: the checker loads them when first used, not before
static WANT: Mutex<(String, String)> = Mutex::new((String::new(), String::new()));

fn spell() -> &'static Mutex<Spell> {
    S.get_or_init(|| {
        let (lang, file) = WANT.lock().unwrap().clone();
        let lang = if lang.is_empty() { std::env::var("ZETT_SPELLLANG").unwrap_or_else(|_| "en_US".into()) } else { lang };
        let mut s = Spell::load(&lang);
        if !file.is_empty() {
            s.file = Some(PathBuf::from(file));
            s.good.clear();
            s.read_add();
        }
        Mutex::new(s)
    })
}

/// Works with the spell checker (loaded on first use).
pub fn with<R>(f: impl FnOnce(&mut Spell) -> R) -> R {
    f(&mut spell().lock().unwrap())
}

impl Spell {
    /// The dictionaries of `langs` ("en_us,de": Vim's names; hunspell's en_US is found too).
    pub fn load(langs: &str) -> Spell {
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
        for lang in langs.split(',').map(str::trim).filter(|l| !l.is_empty()) {
            let hun = match lang.split_once('_') {
                Some((a, b)) => format!("{a}_{}", b.to_uppercase()),
                None => lang.to_string(),
            };
            let mut names = vec![lang.to_string()];
            if hun != lang {
                names.push(hun);
            }
            for n in names {
                for p in [dir().join(format!("{n}.dic")), PathBuf::from(format!("/usr/share/hunspell/{n}.dic")), dir().join(format!("{n}.words"))] {
                    read(p, &mut dict);
                }
            }
            if lang.starts_with("en") {
                read(PathBuf::from("/usr/share/dict/words"), &mut dict);
            }
        }
        let mut s = Spell { lang: langs.to_string(), dict, good: HashSet::new(), wrong: HashSet::new(), session_good: HashSet::new(), session_wrong: HashSet::new(), file: None };
        s.read_add();
        s
    }

    /// The user's own good and wrong words (zg zw), from the add file.
    fn read_add(&mut self) {
        if let Ok(t) = std::fs::read_to_string(self.add_file()) {
            for l in t.lines().map(str::trim).filter(|l| !l.is_empty()) {
                match l.strip_suffix("/!") {
                    Some(w) => self.wrong.insert(w.to_lowercase()),
                    None => self.good.insert(l.to_lowercase()),
                };
            }
        }
    }

    /// A spell checker over these words alone (tests).
    pub fn from_words(words: &[&str]) -> Spell {
        Spell { lang: "test".into(), dict: words.iter().map(|w| w.to_lowercase()).collect(), good: HashSet::new(), wrong: HashSet::new(), session_good: HashSet::new(), session_wrong: HashSet::new(), file: None }
    }

    fn add_file(&self) -> PathBuf {
        self.file.clone().unwrap_or_else(|| dir().join(format!("{}.add", self.lang.split([',', '_']).next().unwrap_or("en"))))
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
    fn bad_words_of_a_line() {
        let s = Spell::from_words(&["the", "quick", "fox", "file", "name"]);
        assert_eq!(s.bad_in("the qiuck fox", false), vec![(4, 9)]);
        assert_eq!(s.bad_in("x2 foo_bar fileName", false), vec![(11, 19)], "names with digits or _ are not words");
        assert!(s.bad_in("fileName", true).is_empty(), "spelloptions=camel checks each part");
    }

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


/// spelllang: the languages to check against ("en_us,de"); loaded again only when they change, and
/// not at all until spelling is used.
pub fn set_lang(langs: &str) {
    let langs = langs.trim();
    {
        let mut w = WANT.lock().unwrap();
        if w.0 == langs {
            return;
        }
        w.0 = langs.to_string();
    }
    if let Some(m) = S.get() {
        let mut s = m.lock().unwrap();
        let file = s.file.clone();
        *s = Spell::load(if langs.is_empty() { "en_US" } else { langs });
        s.file = file;
        s.good.clear();
        s.read_add();
    }
}

/// spellfile: where zg and zw keep words (empty: ~/.local/share/zett/spell/LANG.add).
pub fn set_file(path: &str) {
    let path = path.split(',').next().unwrap_or("").trim();
    {
        let mut w = WANT.lock().unwrap();
        if w.1 == path {
            return;
        }
        w.1 = path.to_string();
    }
    if let Some(m) = S.get() {
        let mut s = m.lock().unwrap();
        s.file = (!path.is_empty()).then(|| PathBuf::from(path));
        s.good.clear();
        s.wrong.clear();
        s.read_add();
    }
}
impl Spell {
    /// The misspelled words of a line, as character columns (start, end); with `camel`
    /// (spelloptions=camel) each part of a CamelCase word is checked alone.
    pub fn bad_in(&self, line: &str, camel: bool) -> Vec<(usize, usize)> {
        let cs: Vec<char> = line.chars().collect();
        let mut out = Vec::new();
        let mut i = 0;
        while i < cs.len() {
            if !cs[i].is_alphabetic() {
                i += 1;
                continue;
            }
            let a = i;
            while i < cs.len() && (cs[i].is_alphabetic() || (cs[i] == '\'' && cs.get(i + 1).is_some_and(|c| c.is_alphabetic()))) {
                i += 1;
            }
            // a word glued to digits or an underscore is a name, not a word
            if cs.get(i).is_some_and(|c| c.is_ascii_digit() || *c == '_') || (a > 0 && (cs[a - 1].is_ascii_digit() || cs[a - 1] == '_')) {
                continue;
            }
            let mut parts = vec![(a, i)];
            if camel {
                parts = Vec::new();
                let mut s = a;
                for j in a + 1..i {
                    if cs[j].is_uppercase() && cs[j - 1].is_lowercase() {
                        parts.push((s, j));
                        s = j;
                    }
                }
                parts.push((s, i));
            }
            for (x, y) in parts {
                if !self.ok(&cs[x..y].iter().collect::<String>()) {
                    out.push((x, y));
                }
            }
        }
        out
    }
}
