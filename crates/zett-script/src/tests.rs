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

#[test]
fn vim_api_reads_and_writes_the_buffer_and_options() {
    let (d, f) = ws("vim-api");
    let mut e = Engine::new(Some(f));
    e.set_options(vec![("tabstop".into(), vec!["ts".into()], "int".into(), "8".into()), ("number".into(), vec!["nu".into()], "bool".into(), "false".into())]);
    e.load_config(&d.join("no-config"));
    assert!(e.errors.is_empty(), "{:?}", e.errors);
    let mut ed = zett_edit::Editor::from_str("one\ntwo\nthree\n");
    let run = |e: &mut Engine, ed: &mut zett_edit::Editor, code: &str| e.lend_buffer(ed, "/tmp/x.txt", &d, |e| e.eval(code)).unwrap();
    // lines: Neovim's indexing (from 0, end not included, -1 the end)
    assert_eq!(run(&mut e, &mut ed, "table.concat(vim.api.nvim_buf_get_lines(0, 0, -1, false), ',')").0, "one,two,three");
    run(&mut e, &mut ed, "vim.api.nvim_buf_set_lines(0, 1, 2, false, { 'TWO', 'two and a half' })");
    assert_eq!(ed.text(), "one\nTWO\ntwo and a half\nthree\n");
    run(&mut e, &mut ed, "vim.api.nvim_buf_set_lines(0, -1, -1, false, { 'four' })");
    assert_eq!(ed.text(), "one\nTWO\ntwo and a half\nthree\nfour\n");
    run(&mut e, &mut ed, "vim.api.nvim_buf_set_lines(0, 0, 1, false, {})");
    assert_eq!(ed.text(), "TWO\ntwo and a half\nthree\nfour\n");
    run(&mut e, &mut ed, "vim.fn.setline(1, 'two'); vim.fn.append(0, 'zero')");
    assert_eq!(ed.text(), "zero\ntwo\ntwo and a half\nthree\nfour\n");
    assert_eq!(run(&mut e, &mut ed, "vim.fn.line('$')").0, "5");
    // options: short names, defaults by kind, a set one read back; :set goes to the editor too
    assert_eq!(run(&mut e, &mut ed, "vim.o.ts").0, "8");
    assert_eq!(run(&mut e, &mut ed, "vim.o.number").0, "false");
    let (_, fx) = run(&mut e, &mut ed, "vim.opt.tabstop = 4");
    assert!(fx.contains(&Effect::Ui("ex".into(), "set tabstop=4".into())), "{fx:?}");
    assert_eq!(run(&mut e, &mut ed, "vim.o.tabstop").0, "4");
    let (_, fx) = run(&mut e, &mut ed, "vim.keymap.set('n', '<leader>w', ':w<CR>', { silent = true })");
    assert!(fx.contains(&Effect::Ui("ex".into(), "nnoremap <silent> <leader>w :w<CR>".into())), "{fx:?}");
    // the plain functions plugins lean on
    assert_eq!(run(&mut e, &mut ed, "vim.fn.fnamemodify('/a/b/c.txt', ':t:r')").0, "c");
    assert_eq!(run(&mut e, &mut ed, "vim.fn.substitute('a-b-c', '-', '+', 'g')").0, "a+b+c");
    assert_eq!(run(&mut e, &mut ed, "vim.json.encode(vim.json.decode('{\"a\":[1,2,{\"b\":true}]}'))").0, "{\"a\":[1,2,{\"b\":true}]}");
    assert_eq!(run(&mut e, &mut ed, "vim.tbl_deep_extend('force', { a = { b = 1, c = 2 } }, { a = { c = 3 } }).a.c").0, "3");
    assert_eq!(run(&mut e, &mut ed, "#vim.split('a,,b', ',', { plain = true })").0, "3");
    assert_eq!(run(&mut e, &mut ed, "vim.fn.has('nvim')").0, "1");
}

#[test]
fn vim_patterns_become_lua_patterns() {
    let (d, f) = ws("vim-pat");
    let mut e = Engine::new(Some(f));
    e.load_config(&d.join("no-config"));
    let ev = |e: &mut Engine, c: &str| e.eval(c).unwrap().0;
    assert_eq!(ev(&mut e, "vim.fn.substitute('a-b-c', '-', '+', 'g')"), "a+b+c");
    assert_eq!(ev(&mut e, "vim.fn.substitute('foo  bar', '\\\\s\\\\+', ' ', '')"), "foo bar");
    assert_eq!(ev(&mut e, "vim.fn.substitute('key=val', '\\\\(\\\\w\\\\+\\\\)=\\\\(\\\\w\\\\+\\\\)', '\\\\2:\\\\1', '')"), "val:key");
    assert_eq!(ev(&mut e, "vim.fn.matchstr('version 1.2.3', '\\\\d\\\\+\\\\.\\\\d\\\\+')"), "1.2");
    assert_eq!(ev(&mut e, "vim.fn.match('the cat', '\\\\<cat\\\\>')"), "4");
    assert_eq!(ev(&mut e, "vim.fn.substitute('a.b', '.', 'x', 'g')"), "xxx");
}

#[test]
fn vimscript_runs_translated_to_lua() {
    let (d, f) = ws("vimscript");
    let mut e = Engine::new(Some(f));
    e.set_options(vec![("tabstop".into(), vec!["ts".into()], "int".into(), "8".into()), ("expandtab".into(), vec!["et".into()], "bool".into(), "false".into())]);
    e.load_config(&d.join("no-config"));
    assert!(e.errors.is_empty(), "{:?}", e.errors);
    let script = r#"
" a comment
let s:count = 0
function! s:Add(a, b) abort
  let s:count += 1
  return a:a + a:b
endfunction
function! Greet(name, ...)
  let extra = a:0 > 0 ? ' and ' . join(a:000, ', ') : ''
  return 'hi ' . a:name . extra
endfunction
let g:sum = 0
for i in range(1, 5)
  if i == 3
    continue
  endif
  let g:sum = s:Add(g:sum, i)
endfor
let g:calls = s:count
let g:greet = Greet('ann', 'bo', 'cy')
let d = {'x': 1, 'y': [10, 20, 30]}
let d.z = d.y[1] * 2
let g:z = d.z
let g:last = d.y[-1]
let g:slice = join(d.y[0:1], '-')
let g:doubled = join(map([1, 2, 3], 'v:val * 2'), ',')
let g:big = join(filter([1, 5, 9], {_, v -> v > 3}), ',')
let n = 0
while n < 10
  let n += 3
endwhile
let g:n = n
try
  throw 'oops'
catch /^oo/
  let g:caught = v:exception
finally
  let g:finally = 1
endtry
set ts=4 expandtab
let g:ts = &ts
let g:et = &et
let g:cat = 'a' . 1 .. 'b'
let g:cmp = ('abc' ==# 'abc') + ('abc' ==? 'ABC') + ('abc' =~ '^a')
let [g:p, g:q] = [7, 8]
function! Deferred()
  defer add(g:log, 'deferred')
  call add(g:log, 'body')
  return 5
endfunction
let g:log = []
let g:rv = Deferred()
let g:logs = join(g:log, ',')
"#;
    e.eval(&format!("vim._vimscript.run({script:?}, 'test.vim')")).unwrap();
    let g = |e: &mut Engine, k: &str| e.eval(&format!("tostring(vim.g.{k})")).unwrap().0;
    assert_eq!(g(&mut e, "sum"), "12", "1+2+4+5, 3 skipped by :continue");
    assert_eq!(g(&mut e, "calls"), "4");
    assert_eq!(g(&mut e, "greet"), "hi ann and bo, cy");
    assert_eq!(g(&mut e, "z"), "40");
    assert_eq!(g(&mut e, "last"), "30");
    assert_eq!(g(&mut e, "slice"), "10-20");
    assert_eq!(g(&mut e, "doubled"), "2,4,6");
    assert_eq!(g(&mut e, "big"), "5,9");
    assert_eq!(g(&mut e, "n"), "12");
    assert_eq!(g(&mut e, "caught"), "oops");
    assert_eq!(g(&mut e, "finally"), "1");
    assert_eq!(g(&mut e, "ts"), "4");
    assert_eq!(g(&mut e, "et"), "1");
    assert_eq!(g(&mut e, "cat"), "a1b");
    assert_eq!(g(&mut e, "cmp"), "3");
    assert_eq!(g(&mut e, "p") + &g(&mut e, "q"), "78");
    assert_eq!(g(&mut e, "rv"), "5");
    assert_eq!(g(&mut e, "logs"), "body,deferred");
    // an error is a Vim error, not a crash
    assert!(e.eval("vim._vimscript.run('echo NoSuchFunc()')").is_ok());
}

#[test]
fn jobstart_streams_and_msgpack_round_trips() {
    let (_d, f) = ws("jobstream");
    let mut e = Engine::new(Some(f));
    // the vim layer, with no user config (an empty folder)
    let cfg = std::env::temp_dir().join(format!("zett-jobstream-cfg-{}", std::process::id()));
    std::fs::create_dir_all(&cfg).unwrap();
    e.load_config(&cfg);
    // output arrives while the job runs, stdin can be written, on_exit comes last
    let out = e.eval(r#"(function()
        local got = {}
        local id = vim.fn.jobstart({ "sh", "-c", "echo one; sleep 0.3; read x; echo got $x" }, {
            on_stdout = function(_, d) got[#got + 1] = table.concat(d, "|") end,
            on_exit = function(_, c) got[#got + 1] = "exit " .. c end })
        vim.wait(200, function() return #got > 0 end)
        local early = got[1]
        vim.fn.chansend(id, "hi\n")
        local code = vim.fn.jobwait({ id }, 3000)[1]
        return early .. " / " .. code .. " / " .. table.concat(got, ";")
    end)()"#).unwrap().0;
    assert!(out.starts_with("one| / 0 / one|;got hi|;;exit 0"), "{out}");
    let m = e.eval(r#"vim.inspect(vim.fn.msgpackparse(vim.fn.msgpackdump({ { 1, "two", { a = 3.5 } }, -7, 70000 }, "B")))"#).unwrap().0;
    assert_eq!(m, r#"{ { 1, "two", { a = 3.5 } }, -7, 70000 }"#);
}

#[test]
fn neovims_lua_modules_are_there() {
    let (_d, f) = ws("nvimmods");
    let mut e = Engine::new(Some(f));
    let cfg = std::env::temp_dir().join(format!("zett-nvimmods-cfg-{}", std::process::id()));
    std::fs::create_dir_all(&cfg).unwrap();
    e.load_config(&cfg);
    let out = e.eval(r#"table.concat({
        vim.iter({ 1, 2, 3, 4 }):filter(function(x) return x % 2 == 0 end):map(function(x) return x * 10 end):join(","),
        tostring(vim.version.cmp("1.2.3", "1.10.0")),
        tostring(vim.version.parse("v2.4.1")),
        vim.uri.uri_from_fname("/a b"),
        vim.base64.encode("zett rocks!"),
        tostring(require("vim.iter") == vim.iter),
        vim.split("a,b", ",")[2],
    }, " ")"#).unwrap().0;
    assert_eq!(out, "20,40 -1 2.4.1 file:///a%20b emV0dCByb2NrcyE= true b");
}
