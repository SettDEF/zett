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
