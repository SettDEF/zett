use super::*;

fn ws(tag: &str) -> (PathBuf, Rc<RefCell<Folder>>) {
    let d = std::env::temp_dir().join(format!("zett-script-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("ideas.md"), "---\ntitle: Ideas\n---\n\nfirst\n").unwrap();
    let f = Folder::open(&d).unwrap();
    (d, Rc::new(RefCell::new(f)))
}

fn k(s: &str) -> Vec<String> {
    keys(s)
}

#[test]
fn key_notation() {
    assert_eq!(k("<leader>pn"), vec!["<leader>", "p", "n"]);
    assert_eq!(k("<c-S>x"), vec!["<C-s>", "x"]);
    assert_eq!(k("<space><CR>"), vec!["<Space>", "<Enter>"]);
    assert_eq!(k(" a"), vec!["<Space>", "a"]);
    assert_eq!(k("<"), vec!["<"]);
}

#[test]
fn leader_keys_wait_and_list_the_choices() {
    let (d, f) = ws("menu");
    let mut e = Engine::new(Some(f));
    e.load_config(&d.join("no-config"));
    assert!(e.errors.is_empty(), "{:?}", e.errors);
    // SPC alone: a menu with the "pages" group and the direct keys
    let Feed::Pending(menu) = e.feed("n", &k("<Space>")) else { panic!("SPC should wait") };
    let p = menu.iter().find(|m| m.key == "p").unwrap();
    assert!(p.group);
    assert_eq!(p.desc, "pages");
    assert!(menu.iter().any(|m| m.key == "x" && m.desc == "Capture" && !m.group));
    // SPC p p runs find-page: the host is asked to open quick open
    assert_eq!(e.feed("n", &k("<Space>pp")), Feed::Ran(vec![Effect::Ui("quickOpen".into(), String::new())]));
    // keys nobody mapped are left to the host
    assert_eq!(e.feed("n", &k("j")), Feed::None);
}

#[test]
fn init_lua_adds_commands_keys_and_hooks() {
    let (d, f) = ws("init");
    let conf = d.join("conf");
    std::fs::create_dir_all(conf.join("plugins/hello/lua/hello")).unwrap();
    std::fs::write(conf.join("plugins/hello/lua/hello/util.lua"), "return { greet = function(n) return 'hi ' .. n end }").unwrap();
    std::fs::write(conf.join("plugins/hello/init.lua"), r#"
        local u = require("hello.util")
        zett.command("hello", function() zett.notify(u.greet("there")) end, { desc = "Say hi" })
    "#).unwrap();
    std::fs::write(conf.join("init.lua"), r#"
        zett.opt.leader = ","
        zett.map("n", "<leader>h", "hello")
        zett.map("n", "<leader>d", function()
            zett.page.open("Ideas")
            zett.page.add("todo", "written by a key")
        end, { desc = "Add a task" })
        opened = 0
        zett.on("page.open", function(p) opened = opened + 1; last = p.title end)
    "#).unwrap();
    let mut e = Engine::new(Some(f.clone()));
    e.load_config(&conf);
    assert!(e.errors.is_empty(), "{:?}", e.errors);
    assert_eq!(e.plugins, vec!["hello"]);
    assert_eq!(e.leader(), ",");
    // ,h is also the start of the built-in ,h… help group: it waits, and runs on the timeout (as in Neovim)
    assert!(matches!(e.feed("n", &k(",h")), Feed::Pending(_)));
    assert_eq!(e.timeout("n", &k(",h")), Feed::Ran(vec![Effect::Notify("hi there".into())]));
    let Feed::Ran(fx) = e.feed("n", &k(",d")) else { panic!() };
    assert!(fx.iter().any(|x| matches!(x, Effect::Open(_))));
    // the page was saved to its file
    let md = std::fs::read_to_string(d.join("ideas.md")).unwrap();
    assert!(md.contains("- [ ] written by a key"), "{md}");
    // hooks
    let ideas = f.borrow().find("Ideas").unwrap();
    e.emit_page("page.open", ideas);
    let (out, _) = e.eval("last .. ' ' .. opened").unwrap();
    assert_eq!(out, "Ideas 1");
    // commands know where they came from
    assert!(e.commands().iter().any(|(n, _, s)| n == "hello" && s == "hello"));
    assert!(e.commands().iter().any(|(n, _, s)| n == "find-page" && s == "zett"));
}

#[test]
fn errors_are_collected_not_fatal() {
    let (d, f) = ws("err");
    let conf = d.join("conf");
    std::fs::create_dir_all(&conf).unwrap();
    std::fs::write(conf.join("init.lua"), "this is not lua").unwrap();
    let mut e = Engine::new(Some(f));
    e.load_config(&conf);
    assert_eq!(e.errors.len(), 1);
    assert!(e.errors[0].starts_with("init.lua"), "{:?}", e.errors);
    // the defaults still work
    assert!(matches!(e.feed("n", &k("<Space>pp")), Feed::Ran(_)));
    assert!(e.run("nope").is_err());
}

#[test]
fn lua_eval_and_page_api() {
    let (d, f) = ws("eval");
    let mut e = Engine::new(Some(f.clone()));
    e.load_config(&d.join("none"));
    assert_eq!(e.eval("1 + 2").unwrap().0, "3");
    let ideas = f.borrow().find("Ideas").unwrap();
    e.set_current(Some(ideas));
    e.eval("zett.page.add('h1', 'Top', 1)").unwrap();
    let (out, _) = e.eval("zett.page.current().blocks[1].kind .. ':' .. zett.page.current().blocks[2].text").unwrap();
    assert_eq!(out, "h1:first");
    assert!(e.eval("zett.page.add('nonsense', 'x')").is_err());
}

#[test]
fn emacs_keys_run_editing_commands_and_chords_wait() {
    let (d, f) = ws("emacs");
    let mut e = Engine::new(Some(f));
    e.set_frontend("editor");
    e.load_config(&d.join("no-config"));
    assert!(e.errors.is_empty(), "{:?}", e.errors);
    // a key on an editing command: the host runs it in the editor
    assert_eq!(e.feed("e", &k("<C-k>")), Feed::Ran(vec![Effect::Edit("kill-line".into(), 1)]));
    // M- is Alt; the host types <A-f>
    assert_eq!(e.feed("e", &["<A-f>".to_string()]), Feed::Ran(vec![Effect::Edit("forward-word".into(), 1)]));
    // the host types <CR> for Enter; the map says <CR> too: one spelling after normalizing
    assert_eq!(e.feed("e", &["<CR>".to_string()]), Feed::Ran(vec![Effect::Edit("newline".into(), 1)]));
    // C-x waits for the rest and lists it
    let Feed::Pending(menu) = e.feed("e", &k("<C-x>")) else { panic!("C-x should wait") };
    assert!(menu.iter().any(|m| m.desc.contains("save")), "{menu:?}");
    // unbound printable keys are the host's (it types them)
    assert_eq!(e.feed("e", &k("a")), Feed::None);
    // M-x lists editing commands next to the Lua ones
    let names: Vec<String> = e.commands().into_iter().map(|c| c.0).collect();
    assert!(names.contains(&"kill-line".to_string()) && names.contains(&"find-file".to_string()));
}

#[test]
fn isearch_plugin_binds_c_s_and_its_own_keymap() {
    let (d, f) = ws("isearch");
    let mut e = Engine::new(Some(f));
    e.set_frontend("editor");
    e.load_config(&d.join("no-config"));
    assert!(e.errors.is_empty(), "{:?}", e.errors);
    assert_eq!(e.feed("e", &k("<C-s>")), Feed::Ran(vec![Effect::Edit("isearch-forward".into(), 1)]));
    assert_eq!(e.feed("e", &k("<C-r>")), Feed::Ran(vec![Effect::Edit("isearch-backward".into(), 1)]));
    assert_eq!(e.feed("isearch", &k("<C-s>")), Feed::Ran(vec![Effect::Edit("isearch-repeat-forward".into(), 1)]));
    assert_eq!(e.feed("isearch", &["<CR>".to_string()]), Feed::Ran(vec![Effect::Edit("isearch-exit".into(), 1)]));
}

#[test]
fn numeric_argument_keys_and_zett_arg() {
    let (d, f) = ws("prefix-arg");
    let mut e = Engine::new(Some(f));
    e.set_frontend("editor");
    e.load_config(&d.join("no-config"));
    assert!(e.errors.is_empty(), "{:?}", e.errors);
    assert_eq!(e.feed("e", &k("<C-u>")), Feed::Ran(vec![Effect::Edit("universal-argument".into(), 1)]));
    assert_eq!(e.feed("e", &["<A-5>".to_string()]), Feed::Ran(vec![Effect::Edit("digit-argument".into(), 1)]));
    assert_eq!(e.feed("e", &["<A-->".to_string()]), Feed::Ran(vec![Effect::Edit("negative-argument".into(), 1)]));
    // a Lua command reads the argument the front end set
    e.set_arg(Some(7));
    let (out, _) = e.eval("return tostring(zett.arg)").unwrap();
    assert_eq!(out, "7");
    e.set_arg(None);
    let (out, _) = e.eval("return tostring(zett.arg)").unwrap();
    assert_eq!(out, "nil");
}

#[test]
fn every_emacs_key_points_at_a_command() {
    let (d, f) = ws("emacs-keys-real");
    let mut e = Engine::new(Some(f));
    e.set_frontend("editor");
    e.load_config(&d.join("no-config"));
    assert!(e.errors.is_empty(), "{:?}", e.errors);
    let have: std::collections::HashSet<String> = e.commands().into_iter().map(|c| c.0).collect();
    let src = include_str!("emacs.lua");
    let missing: Vec<&str> = src.lines().filter_map(|l| l.trim().strip_prefix("map(\"")).filter_map(|l| l.split("\", \"").nth(1)).filter_map(|r| r.split('"').next()).filter(|n| !have.contains(*n)).collect();
    assert!(missing.is_empty(), "keys on commands that do not exist: {missing:?}");
    // an Emacs name runs zett's command
    assert_eq!(e.feed("e", &k("<C-x>o")), Feed::Ran(vec![Effect::Ui("focusWin".into(), "next".into())]));
    assert_eq!(e.feed("e", &k("<C-M-f>")), Feed::Ran(vec![Effect::Edit("forward-sexp".into(), 1)]));
}
