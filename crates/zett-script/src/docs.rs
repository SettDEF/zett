//! zett's documentation: short topics, read with `:help` in the editor or `zett doc`.

/// (topic, one line about it, the text)
pub const TOPICS: &[(&str, &str, &str)] = &[
    ("index", "the topics", include_str!("../docs/index.md")),
    ("editor", "buffers, windows, finding, editing", include_str!("../docs/editor.md")),
    ("keys", "the SPC tree and changing keys", include_str!("../docs/keys.md")),
    ("lsp", "language servers: errors, completion, rename", include_str!("../docs/lsp.md")),
    ("config", "init.lua: options, keys, commands, hooks", include_str!("../docs/config.md")),
    ("terminal", "zett with zeit and zash", include_str!("../docs/terminal.md")),
    ("cli", "zett on the command line", include_str!("../docs/cli.md")),
];

/// A topic by name or by its start (`ls` → lsp); `index` for nothing.
pub fn get(topic: &str) -> Option<(&'static str, &'static str)> {
    let t = topic.trim().to_lowercase();
    let t = if t.is_empty() { "index".to_string() } else { t };
    TOPICS.iter().find(|(n, _, _)| *n == t).or_else(|| TOPICS.iter().find(|(n, _, _)| n.starts_with(&t))).map(|(n, _, text)| (*n, *text))
}

#[cfg(test)]
mod tests {
    #[test]
    fn topics_resolve() {
        assert_eq!(super::get("").unwrap().0, "index");
        assert_eq!(super::get("ls").unwrap().0, "lsp");
        assert!(super::get("nothing-like-it").is_none());
        // every topic the index names exists
        for (n, _, _) in super::TOPICS {
            assert!(super::get(n).is_some());
        }
    }
}
