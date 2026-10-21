//! zett's documentation: short topics, read with `:help` in the editor or `zett doc`.

/// (topic, one line about it, the text)
pub const TOPICS: &[(&str, &str, &str)] = &[
    ("index", "the topics", include_str!("../docs/index.md")),
    ("editor", "buffers, windows, finding, editing", include_str!("../docs/editor.md")),
    ("vim", "Vim mode: search, marks, registers, macros, folds, blocks, spelling, windows", include_str!("../docs/vim.md")),
    ("vim-keys", "every normal-mode key, Neovim's index with zett's state", include_str!("../docs/vim-keys.md")),
    ("emacs", "Emacs keys: moving, killing, M-%, rectangles, registers, macros, C-h", include_str!("../docs/emacs.md")),
    ("keys", "the SPC tree and changing keys", include_str!("../docs/keys.md")),
    ("lsp", "language servers: errors, completion, rename", include_str!("../docs/lsp.md")),
    ("config", "init.lua: options, keys, commands, hooks", include_str!("../docs/config.md")),
    ("plugins", "explorer, undo tree, settings, writing one", include_str!("../docs/plugins.md")),
    ("terminal", "zett with zeit and zash", include_str!("../docs/terminal.md")),
    ("cli", "zett on the command line", include_str!("../docs/cli.md")),
    ("query", "query blocks, references, backlinks, code that runs", include_str!("../docs/query.md")),
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

/// The `commands` topic, made from the running config (plugins and init.lua included).
pub fn commands_topic(e: &crate::Engine) -> String {
    let keys = e.keys_of("n");
    let emacs = e.keys_of("e");
    let mut cmds = e.commands();
    cmds.sort();
    let mut out = String::from("# Commands\n\nEvery command, with its keys. Run any by name: `SPC :` or `:NAME`.\n\n");
    for (name, desc, source) in cmds {
        let k = keys.get(&name).map(|v| v.iter().map(|k| format!("`{k}`")).collect::<Vec<_>>().join(" ")).unwrap_or_default();
        // and the Emacs profile's, as Emacs writes them (C-x o)
        let ek = emacs.get(&name).map(|v| v.iter().map(|k| format!("`{}`", emacs_style(k))).collect::<Vec<_>>().join(" ")).unwrap_or_default();
        let k = [k, ek].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ");
        let from = if source == "zett" { String::new() } else { format!("  ({source})") };
        out.push_str(&format!("- `{name}`  {desc}{}{from}\n", if k.is_empty() { String::new() } else { format!("  {k}") }));
    }
    out
}

/// Keys as Emacs writes them: `<C-x>o` is `C-x o`, `<A-f>` is `M-f`, `<CR>` is `RET`.
pub fn emacs_style(typed: &str) -> String {
    crate::keys(typed)
        .iter()
        .map(|k| {
            let inner = k.strip_prefix('<').and_then(|k| k.strip_suffix('>')).unwrap_or(k);
            let (mods, key) = inner.rsplit_once('-').filter(|(_, k)| !k.is_empty()).unwrap_or(("", inner));
            let key = match key {
                "lt" => "<",
                "gt" => ">",
                "Enter" | "CR" => "RET",
                "Space" => "SPC",
                "BS" => "DEL",
                "Tab" => "TAB",
                "Esc" => "ESC",
                k => k,
            };
            // the engine writes Alt as A-; Emacs says M-
            let mods: Vec<&str> = mods.split('-').filter(|m| !m.is_empty()).map(|m| if m == "A" { "M" } else { m }).collect();
            if mods.is_empty() { key.to_string() } else { format!("{}-{key}", mods.join("-")) }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
