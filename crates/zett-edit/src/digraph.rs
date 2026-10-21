//! Digraphs: two characters for one (CTRL-K a: in insert mode and on the : line gives ä), Vim's
//! table (digraph_table.rs) and the ones :digraphs adds; 'digraph' lets {char1} <BS> {char2} do it too.

use std::cell::RefCell;

thread_local! {
    /// :digraphs {char1}{char2} {number}: these first
    static OWN: RefCell<Vec<(String, char)>> = const { RefCell::new(Vec::new()) };
}

fn find(pair: &str) -> Option<char> {
    OWN.with(|o| o.borrow().iter().find(|(p, _)| p == pair).map(|(_, c)| *c)).or_else(|| crate::digraph_table::TABLE.iter().find(|(p, _)| *p == pair).map(|(_, c)| *c))
}

/// The character `a` `b` stand for; Vim tries `b` `a` too, and else it is `b` itself.
pub fn lookup(a: char, b: char) -> char {
    find(&format!("{a}{b}")).or_else(|| find(&format!("{b}{a}"))).unwrap_or(b)
}

/// The digraph for `a` `b`, when there is one either way round ('digraph': a typed <BS> stays one).
pub fn known(a: char, b: char) -> Option<char> {
    find(&format!("{a}{b}")).or_else(|| find(&format!("{b}{a}")))
}

/// :digraphs {char1}{char2} {number} …: one or more of them; an error for what cannot be one.
pub fn define(arg: &str) -> Result<usize, String> {
    let words: Vec<&str> = arg.split_whitespace().collect();
    let mut n = 0;
    for w in words.chunks(2) {
        let [pair, num] = w else { return Err("E39: Number expected".into()) };
        if pair.chars().count() != 2 {
            return Err(format!("E104: Escape not allowed in digraph: {pair}"));
        }
        let code: u32 = num.parse().map_err(|_| "E39: Number expected".to_string())?;
        let c = char::from_u32(code).ok_or_else(|| format!("E39: Number expected: {num}"))?;
        OWN.with(|o| {
            let mut o = o.borrow_mut();
            o.retain(|(p, _)| p != pair);
            o.push((pair.to_string(), c));
        });
        n += 1;
    }
    Ok(n)
}

/// Every digraph for :digraphs: (the two characters, what they give, its number, added by :digraphs).
pub fn list() -> Vec<(String, char, u32, bool)> {
    let own: Vec<(String, char)> = OWN.with(|o| o.borrow().clone());
    let mut out: Vec<(String, char, u32, bool)> = crate::digraph_table::TABLE.iter().filter(|(p, _)| !own.iter().any(|(q, _)| q == p)).map(|(p, c)| (p.to_string(), *c, *c as u32, false)).collect();
    out.extend(own.into_iter().map(|(p, c)| (p, c, c as u32, true)));
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn vims_table_both_ways_and_own_ones() {
        assert_eq!(super::lookup('a', ':'), 'ä');
        assert_eq!(super::lookup(':', 'a'), 'ä', "the other way round too");
        assert_eq!(super::lookup('e', '\''), 'é');
        assert_eq!(super::lookup('E', 'u'), '€');
        assert_eq!(super::lookup('q', 'q'), 'q', "no digraph: the second character");
        assert_eq!(crate::digraph_table::TABLE.len(), 1366);
        assert_eq!(super::define("qq 8364 zz 9731").unwrap(), 2);
        assert_eq!(super::lookup('q', 'q'), '€');
        assert_eq!(super::lookup('z', 'z'), '☃');
        assert!(super::define("abc 65").is_err());
        assert!(super::list().iter().any(|(p, c, n, own)| p == "qq" && *c == '€' && *n == 8364 && *own));
    }
}
