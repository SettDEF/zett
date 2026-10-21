//! The tests of vim.rs's keys.

mod tests {
    use crate::Editor;

    fn ed(t: &str) -> Editor {
        let mut e = Editor::from_str(t);
        e.set_wrapscan(true);
        e
    }

    #[test]
    fn search_with_slash_n_and_star() {
        let mut e = ed("one two one three one");
        e.feed("/one<CR>");
        assert_eq!(e.cursor, 8);
        e.feed("n");
        assert_eq!(e.cursor, 18);
        e.feed("n");
        assert_eq!(e.cursor, 0, "wraps around");
        e.feed("N");
        assert_eq!(e.cursor, 18);
        e.feed("?two<CR>");
        assert_eq!(e.cursor, 4);
        let mut e = ed("foo foobar foo");
        e.feed("*");
        assert_eq!(e.cursor, 11, "* skips foobar: whole words");
        e.feed("d/bar<CR>");
        // from 11 nothing matches forward but it wraps to 7: the motion deletes backward to it
        assert!(e.text().len() < 14);
    }

    #[test]
    fn marks_registers_macros_and_dot() {
        let mut e = ed("alpha\nbeta\ngamma\n");
        e.feed("majj'a");
        assert_eq!(e.cursor().line, 0);
        e.feed("\"ayy");
        e.feed("j\"byy");
        e.feed("G\"ap");
        assert!(e.text().ends_with("gamma\nalpha\n"), "{:?}", e.text());
        e.feed("\"Ayy");
        assert_eq!(e.register('a').unwrap().0, "alpha\nalpha\n", "upper case appends");
        let mut e = ed("1\n2\n3\n");
        e.feed("qqA!<Esc>jq");
        e.feed("@q");
        assert_eq!(e.text(), "1!\n2!\n3\n");
        e.feed("@@");
        assert_eq!(e.text(), "1!\n2!\n3!\n");
        let mut e = ed("a b c d");
        e.feed("dw");
        e.feed("..");
        assert_eq!(e.text(), "d");
    }

    #[test]
    fn counting_case_shifts_and_lines() {
        let mut e = ed("x = 41 and 0x0f");
        e.feed("<C-a>");
        assert_eq!(e.text(), "x = 42 and 0x0f");
        e.feed("w5<C-x>");
        assert!(e.text().contains("0x0a"), "{:?}", e.text());
        let mut e = ed("Hello World");
        e.feed("g~w");
        assert_eq!(e.text(), "hELLO World");
        e.feed("gUU");
        assert_eq!(e.text(), "HELLO WORLD");
        e.feed("g??");
        assert_eq!(e.text(), "URYYB JBEYQ");
        let mut e = ed("a\nb\n");
        e.set_opt("shiftwidth", "4");
        e.feed(">>j>>");
        assert_eq!(e.text(), "    a\n    b\n");
        e.feed("<<");
        assert_eq!(e.text(), "    a\nb\n");
        e.feed("~");
        assert_eq!(e.text(), "    a\nB\n");
    }

    #[test]
    fn brackets_paragraphs_and_replace_mode() {
        let mut e = ed("f(a, (b))");
        e.feed("%");
        assert_eq!(e.cursor, 8);
        e.feed("%");
        assert_eq!(e.cursor, 1);
        let mut e = ed("a\nb\n\nc\n");
        e.feed("}");
        assert_eq!(e.cursor().line, 2);
        let mut e = ed("abcd");
        e.feed("RXY<Esc>");
        assert_eq!(e.text(), "XYcd");
        let mut e = ed("a b");
        e.feed("ZZ");
        assert_eq!(e.take_requests(), vec![crate::vim::Request::Ex("x".into())]);
    }
}

mod batch2 {
    use crate::vim::Request;
    use crate::Editor;

    fn lines(n: usize) -> Editor {
        let t: String = (1..=n).map(|i| format!("line {i}\n")).collect();
        let mut e = Editor::from_str(&t);
        e.set_screen(0, 10, 40);
        e
    }

    #[test]
    fn windows_scrolling_and_the_view() {
        let mut e = lines(100);
        e.feed("<C-w>v2<C-w>+<C-w><C-w><C-w>gf");
        assert_eq!(e.take_requests(), vec![Request::Window("v".into(), None), Request::Window("+".into(), Some(2)), Request::Window("C-w".into(), None), Request::Window("gf".into(), None)]);
        e.feed("<C-d>");
        assert_eq!(e.cursor().line, 5);
        assert_eq!(e.take_requests(), vec![Request::Top(5)]);
        e.set_screen(5, 10, 40);
        e.feed("<C-e><C-e>");
        assert_eq!(e.take_requests(), vec![Request::Top(6), Request::Top(7)]);
        e.feed("30Gzz");
        assert_eq!(e.take_requests(), vec![Request::Top(24)]);
        e.feed("zt");
        assert_eq!(e.take_requests(), vec![Request::Top(29)]);
    }

    #[test]
    fn the_jump_list_goes_back_and_forth() {
        let mut e = lines(100);
        e.feed("50G");
        e.feed("80G");
        e.feed("<C-o>");
        assert_eq!(e.cursor().line, 49);
        e.feed("<C-o>");
        assert_eq!(e.cursor().line, 0);
        e.feed("<C-i>");
        assert_eq!(e.cursor().line, 49);
        e.feed("<Tab>");
        assert_eq!(e.cursor().line, 79);
    }
}

mod objects {
    use crate::Editor;

    #[test]
    fn sentences_and_tags() {
        let mut e = Editor::from_str("One two. Three four. Five.");
        e.feed("fhdis");
        assert_eq!(e.text(), "One two.  Five.", "is keeps the blanks around it, as Vim");
        let mut e = Editor::from_str("One two. Three four. Five.");
        e.feed("fhdas");
        assert_eq!(e.text(), "One two. Five.");
        let mut e = Editor::from_str("<p>hi <b>there</b> you</p>");
        e.feed("fedit");
        assert_eq!(e.text(), "<p>hi <b></b> you</p>");
        e.feed("0fhdat");
        assert_eq!(e.text(), "");
    }
}

mod brackets {
    use crate::Editor;

    #[test]
    fn unmatched_sections_marks_and_puts() {
        let mut e = Editor::from_str("f(a, g(b), c)");
        e.feed("fb[(");
        assert_eq!(e.cursor, 6);
        e.feed("[(");
        assert_eq!(e.cursor, 1);
        e.feed("])");
        assert_eq!(e.cursor, 12);
        let mut e = Editor::from_str("int a;\n{\n  x;\n}\nint b;\n{\n  y;\n}\n");
        e.feed("]]");
        assert_eq!(e.cursor().line, 1);
        e.feed("]]");
        assert_eq!(e.cursor().line, 5);
        e.feed("[]");
        assert_eq!(e.cursor().line, 3);
        let mut e = Editor::from_str("a\nb\nc\nd\n");
        e.feed("majmbG['");
        assert_eq!(e.cursor().line, 1);
        let mut e = Editor::from_str("    x\ny\n");
        e.feed("jyyk]p");
        assert_eq!(e.text(), "    x\n    y\ny\n");
    }
}

mod line_undo_and_screen_lines {
    use crate::Editor;

    #[test]
    fn u_restores_the_line_q_replays_and_gj_follows_the_wrap() {
        let mut e = Editor::from_str("hello world\nnext\n");
        e.feed("xxwx");
        assert_eq!(e.text(), "llo orld\nnext\n");
        e.feed("U");
        assert_eq!(e.text(), "hello world\nnext\n");
        e.feed("U");
        assert_eq!(e.text(), "llo orld\nnext\n", "U again undoes the U");
        let mut e = Editor::from_str("a\nb\nc\n");
        e.feed("qwA;<Esc>jq");
        e.feed("Q");
        assert_eq!(e.text(), "a;\nb;\nc\n");
        let mut e = Editor::from_str(&format!("{}\nshort\n", "x".repeat(25)));
        e.set_screen(0, 10, 10);
        e.set_wrap(true);
        e.feed("gj");
        assert_eq!(e.cursor, 10, "a screen row down inside the long line");
        e.feed("gjgj");
        assert_eq!(e.cursor().line, 1);
        e.feed("ggg$");
        assert_eq!(e.cursor, 9, "the end of the first screen row");
    }
}

mod folds {
    use crate::Editor;

    #[test]
    fn manual_folds_close_open_nest_and_move_with_the_text() {
        let mut e = Editor::from_str("a\nb\nc\nd\ne\nf\n");
        e.feed("jzfj");
        assert!(e.is_hidden(2) && !e.is_hidden(1) && !e.is_hidden(3), "zfj folds lines 2-3, shown as its first");
        e.feed("zo");
        assert!(!e.is_hidden(2));
        e.feed("zc");
        assert!(e.is_hidden(2));
        e.feed("za");
        assert!(!e.is_hidden(2));
        // a line added above moves the fold with its text
        e.feed("ggOnew<Esc>");
        e.feed("jjzc");
        assert!(e.is_hidden(3) && !e.is_hidden(2));
        e.feed("zR");
        assert!(!e.is_hidden(3));
        e.feed("zM");
        assert!(e.is_hidden(3));
        e.feed("zE");
        assert!(!e.is_hidden(3));
        // zF with a count, zd removes it, zj moves to the next one
        e.feed("gg3zF");
        assert!(e.is_hidden(1) && e.is_hidden(2) && !e.is_hidden(3));
        e.feed("zd");
        assert!(!e.is_hidden(1));
        e.feed("4GzfjggzRzj");
        assert_eq!(e.cursor().line, 3);
    }
}

mod block {
    use crate::Editor;

    #[test]
    fn a_block_yanks_deletes_inserts_and_puts() {
        let mut e = Editor::from_str("abcd\nefgh\nijkl\n");
        e.feed("l<C-v>jld");
        assert_eq!(e.text(), "ad\neh\nijkl\n", "two columns over two lines go");
        e.feed("u");
        e.feed("gg0<C-v>jjIX<Esc>");
        assert_eq!(e.text(), "Xabcd\nXefgh\nXijkl\n", "I types on every line");
        let mut e = Editor::from_str("ab\nabcdef\nabc\n");
        e.feed("<C-v>jj$A;<Esc>");
        assert_eq!(e.text(), "ab;\nabcdef;\nabc;\n", "$A appends at each line's end");
        let mut e = Editor::from_str("1234\n5678\n");
        e.feed("l<C-v>jly");
        e.feed("$p");
        assert_eq!(e.text(), "123423\n567867\n", "a block register puts its lines one under another");
        let mut e = Editor::from_str("aaaa\nbbbb\n");
        e.feed("<C-v>jlrx");
        assert_eq!(e.text(), "xxaa\nxxbb\n");
        e.feed("<C-v>jlU");
        assert_eq!(e.text(), "XXaa\nXXbb\n");
        let mut e = Editor::from_str("ab\ncd\n");
        e.feed("l<C-v>j>");
        assert_eq!(e.text(), "a    b\nc    d\n");
    }
}

mod options {
    use crate::Editor;

    fn with(text: &str, opts: &[(&str, &str)]) -> Editor {
        let mut e = Editor::from_str(text);
        for (k, v) in opts {
            e.set_opt(k, v);
        }
        e
    }

    #[test]
    fn indenting_backspace_and_wrapping() {
        let mut e = with("    a\n", &[("autoindent", "false")]);
        e.feed("ob<Esc>");
        assert_eq!(e.text(), "    a\nb\n", "noautoindent: the new line starts at column 0");
        let mut e = with("if x {\n", &[("smartindent", "true"), ("shiftwidth", "4")]);
        e.set_soft_tab(Some(4));
        e.feed("oy<Esc>");
        assert_eq!(e.text(), "if x {\n    y\n", "smartindent: one step in after {{");
        let mut e = with("", &[("softtabstop", "4")]);
        e.set_soft_tab(Some(4));
        e.feed("i<Tab><Tab><BS>x<Esc>");
        assert_eq!(e.text(), "    x", "softtabstop: Backspace takes a whole stop");
        let mut e = with("", &[("textwidth", "10"), ("formatoptions", "tcq")]);
        e.feed("ione two three four<Esc>");
        assert_eq!(e.text(), "one two\nthree four", "textwidth: typing wraps at the last blank");
        let mut e = with("Fine.\nNext\n", &[("joinspaces", "true")]);
        e.feed("J");
        assert_eq!(e.text(), "Fine.  Next\n");
        let mut e = with("// one\n// two\n", &[]);
        e.feed("J");
        assert_eq!(e.text(), "// one two\n", "formatoptions j: the comment leader goes");
    }

    #[test]
    fn motions_numbers_words_pairs_and_selection() {
        let mut e = with("abc\nabc\nabc\n", &[]);
        e.feed("llG");
        assert_eq!(e.cursor().col, 2, "nostartofline (Neovim's default): G keeps the column");
        let mut e = with("abc\nabc\n", &[("startofline", "true")]);
        e.feed("llG");
        assert_eq!(e.cursor().col, 0);
        let mut e = with("ab\ncd\n", &[("whichwrap", "h,l")]);
        e.feed("jlh");
        assert_eq!(e.cursor().line, 1, "inside the line h stays on it");
        e.feed("h");
        assert_eq!(e.cursor().line, 0, "whichwrap h: on to the line above");
        let mut e = with("abc", &[("tildeop", "true")]);
        e.feed("~w");
        assert_eq!(e.text(), "ABC");
        let mut e = with("0b101 and a", &[("nrformats", "bin,alpha")]);
        e.feed("<C-a>");
        assert_eq!(e.text(), "0b110 and a");
        e.feed("$<C-a>");
        assert_eq!(e.text(), "0b110 and b");
        let mut e = with("foo-bar baz", &[("iskeyword", "@,48-57,_,-")]);
        e.feed("w");
        assert_eq!(e.cursor, 8, "iskeyword with -: foo-bar is one word");
        let mut e = with("<a>", &[("matchpairs", "<:>")]);
        e.feed("%");
        assert_eq!(e.cursor, 2);
        let mut e = with("abcdef", &[("selection", "exclusive")]);
        e.feed("vlld");
        assert_eq!(e.text(), "cdef", "exclusive: the cursor's character stays");
        let mut e = with("a\nb\nc\nd\n", &[("modifiable", "false")]);
        e.feed("dd");
        assert_eq!(e.text(), "a\nb\nc\nd\n");
        assert!(e.take_requests().iter().any(|r| matches!(r, crate::vim::Request::Message(m) if m.starts_with("E21"))));
        let mut e = with("1\n2\n3\n4\n5\n", &[("report", "2")]);
        e.feed("3dd");
        assert!(e.take_requests().contains(&crate::vim::Request::Message("3 fewer lines".into())));
    }
}

mod clipboard {
    use crate::Editor;
    use std::sync::Mutex;

    static FAKE: Mutex<String> = Mutex::new(String::new());

    #[test]
    fn unnamedplus_yanks_and_puts_through_the_system_clipboard() {
        let _ = crate::vim::CLIPBOARD.set((|| Some(FAKE.lock().unwrap().clone()), |t| *FAKE.lock().unwrap() = t.to_string()));
        let mut e = Editor::from_str("one\ntwo\n");
        e.set_opt("clipboard", "unnamedplus");
        e.feed("yy");
        assert_eq!(*FAKE.lock().unwrap(), "one\n", "the yank reached the clipboard");
        *FAKE.lock().unwrap() = "from outside\n".into();
        e.feed("jp");
        assert_eq!(e.text(), "one\ntwo\nfrom outside\n", "the put came from it");
        e.feed("ggjll");
        let before = FAKE.lock().unwrap().clone();
        assert_eq!(before, "from outside\n", "moving about leaves it alone");
        let mut e = Editor::from_str("abc\n");
        e.feed("x");
        assert_eq!(*FAKE.lock().unwrap(), "from outside\n", "without the option it stays out of it");
    }
}

mod foldmethod {
    use crate::Editor;

    fn ed(text: &str, opts: &[(&str, &str)]) -> Editor {
        let mut e = Editor::from_str(text);
        for (k, v) in opts {
            e.set_opt(k, v);
        }
        e.refresh_view();
        e
    }

    #[test]
    fn indent_and_marker_folds_with_their_options() {
        let code = "fn a() {\n    x\n    if y {\n        z\n    }\n}\nfn b() {}\n";
        let e = ed(code, &[("foldmethod", "indent"), ("shiftwidth", "4")]);
        assert!(e.is_hidden(2) && e.is_hidden(4), "level 0: the indented lines are folded under their first");
        assert!(!e.is_hidden(1) && !e.is_hidden(0) && !e.is_hidden(5) && !e.is_hidden(6));
        let e = ed(code, &[("foldmethod", "indent"), ("shiftwidth", "4"), ("foldlevel", "1")]);
        assert!(!e.is_hidden(2) && !e.is_hidden(4), "foldlevel 1: only the deeper fold closes, and it is one line (foldminlines)");
        let mut e = ed(code, &[("foldmethod", "indent"), ("shiftwidth", "4")]);
        e.feed("jzo");
        e.refresh_view();
        assert!(!e.is_hidden(2), "zo opens a fold worked out from the text");
        e.feed("Gonew<Esc>");
        e.refresh_view();
        assert!(!e.is_hidden(2), "an edit elsewhere keeps it open");
        e.feed("zf");
        assert!(e.take_requests().iter().any(|r| matches!(r, crate::vim::Request::Message(m) if m.starts_with("E350"))));
        let m = "a {{{\nb\nc }}}\nd {{{1\ne\nf {{{2\ng\n";
        let e = ed(m, &[("foldmethod", "marker")]);
        assert!(e.is_hidden(1) && e.is_hidden(2) && !e.is_hidden(3), "a start marker to its end marker");
        assert!(e.is_hidden(4) && e.is_hidden(6), "a level-1 marker runs to the end, a level-2 one nests in it");
        let e = ed(m, &[("foldmethod", "marker"), ("foldenable", "false")]);
        assert!(!e.is_hidden(1), "nofoldenable shows everything");
        let mut e = ed(code, &[("foldmethod", "indent"), ("shiftwidth", "4"), ("foldclose", "all")]);
        e.feed("jzo");
        e.refresh_view();
        assert!(!e.is_hidden(2));
        e.feed("G");
        e.refresh_view();
        assert!(e.is_hidden(2), "foldclose=all: the fold the cursor left closes");
    }
}

mod shifting {
    use crate::Editor;

    fn ed(text: &str, opts: &[(&str, &str)]) -> Editor {
        let mut e = Editor::from_str(text);
        for (k, v) in opts {
            e.set_opt(k, v);
        }
        e
    }

    #[test]
    fn shiftwidth_shiftround_preserveindent_and_programs() {
        let mut e = ed("a\n\nb\n", &[("shiftwidth", "2")]);
        e.feed(">2j");
        assert_eq!(e.text(), "  a\n\n  b\n", "shiftwidth 2, the empty line left alone");
        let mut e = ed("   a\n", &[("shiftwidth", "4"), ("shiftround", "true")]);
        e.feed(">>");
        assert_eq!(e.text(), "    a\n", "shiftround: 3 goes to 4, not 7");
        e.feed(">><<");
        assert_eq!(e.text(), "    a\n");
        let mut e = ed("   a\n", &[("shiftwidth", "4")]);
        e.feed("<<");
        assert_eq!(e.text(), "a\n");
        let mut e = ed("\t a\n", &[("shiftwidth", "4"), ("tabstop", "8"), ("expandtab", "false"), ("preserveindent", "true")]);
        e.feed(">>");
        assert_eq!(e.text(), "\t     a\n", "preserveindent keeps the tab and adds at the end");
        let mut e = ed("a\n", &[("shiftwidth", "8"), ("tabstop", "8"), ("expandtab", "false")]);
        e.feed(">>");
        assert_eq!(e.text(), "\ta\n", "noexpandtab: a tab");
        let mut e = ed("b\na\n", &[("equalprg", "sort")]);
        e.feed("=j");
        assert!(e.take_requests().contains(&crate::vim::Request::Ex("1,2!sort".into())), "equalprg filters the lines");
        let mut e = ed("x y\n", &[("formatprg", "fmt -w 20")]);
        e.feed("gqq");
        assert!(e.take_requests().contains(&crate::vim::Request::Ex("1,1!fmt -w 20".into())));
    }
}
