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
