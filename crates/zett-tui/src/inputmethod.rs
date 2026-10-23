//! Input methods (Emacs's C-\): what was just typed before the cursor becomes another character.
//! TeX (`\alpha` α, `\to` →, `^2` ²), german-prefix (`"a` ä), latin-1-prefix (`'e` é, `` `a `` à,
//! `~n` ñ, `,c` ç), german-postfix (`ae` ä, `oe` ö, `ue` ü, `sz` ß).

pub const METHODS: &[(&str, &str)] = &[
    ("TeX", "\\alpha α, \\to →, \\in ∈, ^2 ², _1 ₁: TeX's names for symbols"),
    ("german-prefix", "\"a ä, \"o ö, \"u ü, \"s ß (\"\" a plain \")"),
    ("german-postfix", "ae ä, oe ö, ue ü, sz ß (aee: a plain ae)"),
    ("latin-1-prefix", "'e é, `a à, ^o ô, \"u ü, ~n ñ, ,c ç, /o ø"),
];

const TEX: &[(&str, &str)] = &[
    ("alpha", "α"), ("beta", "β"), ("gamma", "γ"), ("delta", "δ"), ("epsilon", "ε"), ("varepsilon", "ε"), ("zeta", "ζ"),
    ("eta", "η"), ("theta", "θ"), ("vartheta", "ϑ"), ("iota", "ι"), ("kappa", "κ"), ("lambda", "λ"), ("mu", "μ"),
    ("nu", "ν"), ("xi", "ξ"), ("pi", "π"), ("varpi", "ϖ"), ("rho", "ρ"), ("sigma", "σ"), ("varsigma", "ς"), ("tau", "τ"),
    ("upsilon", "υ"), ("phi", "φ"), ("varphi", "ϕ"), ("chi", "χ"), ("psi", "ψ"), ("omega", "ω"),
    ("Gamma", "Γ"), ("Delta", "Δ"), ("Theta", "Θ"), ("Lambda", "Λ"), ("Xi", "Ξ"), ("Pi", "Π"), ("Sigma", "Σ"),
    ("Upsilon", "Υ"), ("Phi", "Φ"), ("Psi", "Ψ"), ("Omega", "Ω"),
    ("to", "→"), ("rightarrow", "→"), ("leftarrow", "←"), ("gets", "←"), ("uparrow", "↑"), ("downarrow", "↓"),
    ("leftrightarrow", "↔"), ("Rightarrow", "⇒"), ("Leftarrow", "⇐"), ("Leftrightarrow", "⇔"), ("mapsto", "↦"),
    ("in", "∈"), ("notin", "∉"), ("ni", "∋"), ("subset", "⊂"), ("supset", "⊃"), ("subseteq", "⊆"), ("supseteq", "⊇"),
    ("cup", "∪"), ("cap", "∩"), ("emptyset", "∅"), ("forall", "∀"), ("exists", "∃"), ("neg", "¬"), ("lnot", "¬"),
    ("wedge", "∧"), ("land", "∧"), ("vee", "∨"), ("lor", "∨"), ("infty", "∞"), ("partial", "∂"), ("nabla", "∇"),
    ("sum", "∑"), ("prod", "∏"), ("int", "∫"), ("oint", "∮"), ("sqrt", "√"), ("pm", "±"), ("mp", "∓"), ("times", "×"),
    ("div", "÷"), ("cdot", "·"), ("circ", "∘"), ("bullet", "•"), ("star", "⋆"), ("ast", "∗"), ("leq", "≤"), ("le", "≤"),
    ("geq", "≥"), ("ge", "≥"), ("neq", "≠"), ("ne", "≠"), ("approx", "≈"), ("equiv", "≡"), ("sim", "∼"), ("simeq", "≃"),
    ("cong", "≅"), ("propto", "∝"), ("ll", "≪"), ("gg", "≫"), ("perp", "⊥"), ("parallel", "∥"), ("angle", "∠"),
    ("degree", "°"), ("prime", "′"), ("dagger", "†"), ("ddagger", "‡"), ("ldots", "…"), ("dots", "…"), ("cdots", "⋯"),
    ("vdots", "⋮"), ("ddots", "⋱"), ("aleph", "ℵ"), ("hbar", "ℏ"), ("ell", "ℓ"), ("Re", "ℜ"), ("Im", "ℑ"), ("wp", "℘"),
    ("top", "⊤"), ("bot", "⊥"), ("vdash", "⊢"), ("models", "⊨"), ("oplus", "⊕"), ("otimes", "⊗"), ("langle", "⟨"),
    ("rangle", "⟩"), ("lceil", "⌈"), ("rceil", "⌉"), ("lfloor", "⌊"), ("rfloor", "⌋"), ("checkmark", "✓"),
    ("copyright", "©"), ("pounds", "£"), ("euro", "€"), ("S", "§"), ("P", "¶"), ("ss", "ß"), ("ae", "æ"), ("oe", "œ"),
    ("o", "ø"), ("O", "Ø"), ("aa", "å"), ("AA", "Å"), ("l", "ł"), ("L", "Ł"),
];

const SUP: &[(char, char)] = &[('0', '⁰'), ('1', '¹'), ('2', '²'), ('3', '³'), ('4', '⁴'), ('5', '⁵'), ('6', '⁶'), ('7', '⁷'), ('8', '⁸'), ('9', '⁹'), ('+', '⁺'), ('-', '⁻'), ('=', '⁼'), ('(', '⁽'), (')', '⁾'), ('n', 'ⁿ'), ('i', 'ⁱ')];
const SUB: &[(char, char)] = &[('0', '₀'), ('1', '₁'), ('2', '₂'), ('3', '₃'), ('4', '₄'), ('5', '₅'), ('6', '₆'), ('7', '₇'), ('8', '₈'), ('9', '₉'), ('+', '₊'), ('-', '₋'), ('=', '₌'), ('(', '₍'), (')', '₎')];

const GERMAN_PREFIX: &[(&str, &str)] = &[("\"a", "ä"), ("\"o", "ö"), ("\"u", "ü"), ("\"A", "Ä"), ("\"O", "Ö"), ("\"U", "Ü"), ("\"s", "ß"), ("\"\"", "\"")];
const GERMAN_POSTFIX: &[(&str, &str)] = &[("ae", "ä"), ("oe", "ö"), ("ue", "ü"), ("Ae", "Ä"), ("Oe", "Ö"), ("Ue", "Ü"), ("AE", "Ä"), ("OE", "Ö"), ("UE", "Ü"), ("sz", "ß")];

/// The vowels and what each accent makes of them (latin-1-prefix).
fn latin1(accent: char, c: char) -> Option<char> {
    let (from, to): (&str, &str) = match accent {
        '\'' => ("aeiouyAEIOUY", "áéíóúýÁÉÍÓÚÝ"),
        '`' => ("aeiouAEIOU", "àèìòùÀÈÌÒÙ"),
        '^' => ("aeiouAEIOU", "âêîôûÂÊÎÔÛ"),
        '"' => ("aeiouyAEIOU", "äëïöüÿÄËÏÖÜ"),
        '~' => ("anoANO", "ãñõÃÑÕ"),
        ',' => ("cC", "çÇ"),
        '/' => ("oOaAeE", "øØåÅæÆ"),
        _ => return None,
    };
    from.chars().position(|f| f == c).and_then(|i| to.chars().nth(i))
}

/// What the text before the cursor (`before`, its end) becomes under `method`: how many characters
/// go and what takes their place. `next` is the character typed after a TeX name, which ends one
/// that is also the start of a longer one (`\in` then space: ∈ and the space kept).
pub fn compose(method: &str, before: &str) -> Option<(usize, String)> {
    let tail: Vec<char> = before.chars().rev().take(32).collect::<Vec<_>>().into_iter().rev().collect();
    let n = tail.len();
    match method {
        "TeX" => {
            // ^2 _1: super- and subscripts
            if n >= 2 {
                let (mark, c) = (tail[n - 2], tail[n - 1]);
                let table = match mark {
                    '^' => SUP,
                    '_' => SUB,
                    _ => &[][..],
                };
                if let Some((_, s)) = table.iter().find(|(k, _)| *k == c) {
                    return Some((2, s.to_string()));
                }
            }
            // \name: as soon as it is whole and no longer name starts with it; or ended by a non-letter
            let i = tail.iter().rposition(|c| *c == '\\')?;
            let word: String = tail[i + 1..].iter().collect();
            if word.is_empty() {
                return None;
            }
            let (name, ender) = match word.chars().last() {
                Some(c) if !c.is_alphabetic() => (word[..word.len() - c.len_utf8()].to_string(), Some(c)),
                _ => (word.clone(), None),
            };
            if !name.chars().all(char::is_alphabetic) {
                return None;
            }
            let hit = TEX.iter().find(|(k, _)| *k == name)?;
            let longer = TEX.iter().any(|(k, _)| k.len() > name.len() && k.starts_with(&name));
            match ender {
                Some(e) => Some((word.chars().count() + 1, format!("{}{e}", hit.1))),
                None if !longer => Some((name.chars().count() + 1, hit.1.to_string())),
                None => None,
            }
        }
        "german-prefix" | "german-postfix" => {
            let table = if method == "german-prefix" { GERMAN_PREFIX } else { GERMAN_POSTFIX };
            if n < 2 {
                return None;
            }
            // german-postfix: "aee" is a plain "ae" (the e doubled undoes it)
            if method == "german-postfix" && n >= 2 && tail[n - 1] == 'e' && "äöüÄÖÜ".contains(tail[n - 2]) {
                let plain = match tail[n - 2] {
                    'ä' => "ae", 'ö' => "oe", 'ü' => "ue", 'Ä' => "Ae", 'Ö' => "Oe", _ => "Ue",
                };
                return Some((2, plain.to_string()));
            }
            let two: String = tail[n - 2..].iter().collect();
            table.iter().find(|(k, _)| *k == two).map(|(_, v)| (2, v.to_string()))
        }
        "latin-1-prefix" => {
            if n < 2 {
                return None;
            }
            latin1(tail[n - 2], tail[n - 1]).map(|c| (2, c.to_string()))
        }
        _ => None,
    }
}

/// The method's table, for C-h I.
pub fn describe(method: &str) -> Vec<(String, String)> {
    match method {
        "TeX" => TEX.iter().map(|(k, v)| (format!("\\{k}"), v.to_string())).chain(SUP.iter().map(|(k, v)| (format!("^{k}"), v.to_string()))).chain(SUB.iter().map(|(k, v)| (format!("_{k}"), v.to_string()))).collect(),
        "german-prefix" => GERMAN_PREFIX.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        "german-postfix" => GERMAN_POSTFIX.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        "latin-1-prefix" => "'`^\"~,/".chars().flat_map(|a| "aeiouycnAEIOUYCN".chars().filter_map(move |c| latin1(a, c).map(|r| (format!("{a}{c}"), r.to_string())))).collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::compose;

    #[test]
    fn tex_names_turn_into_symbols_when_whole() {
        assert_eq!(compose("TeX", "a \\alpha"), Some((6, "α".into())));
        // \in is the start of \infty: it waits, and a space ends it
        assert_eq!(compose("TeX", "\\in"), None);
        assert_eq!(compose("TeX", "\\in "), Some((4, "∈ ".into())));
        assert_eq!(compose("TeX", "x^2"), Some((2, "²".into())));
        assert_eq!(compose("TeX", "\\nothing "), None);
    }

    #[test]
    fn prefix_and_postfix_methods() {
        assert_eq!(compose("german-prefix", "Gr\"u"), Some((2, "ü".into())));
        assert_eq!(compose("german-postfix", "Gruesse"), None);
        assert_eq!(compose("german-postfix", "Grue"), Some((2, "ü".into())));
        assert_eq!(compose("german-postfix", "Grüe"), Some((2, "ue".into())), "the e doubled undoes it");
        assert_eq!(compose("latin-1-prefix", "caf'e"), Some((2, "é".into())));
        assert_eq!(compose("latin-1-prefix", "plain"), None);
    }
}
