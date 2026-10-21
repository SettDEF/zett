//! The editor's tests (keys in, text and cursor out).

#[cfg(test)]
mod tests {
    use crate::*;

    /// Runs keys on a text and returns (text, cursor as (line, col)) — what Vim would show.
    fn run(text: &str, keys: &str) -> (String, (usize, usize)) {
        let mut e = Editor::from_str(text);
        e.feed(keys);
        let c = e.cursor();
        (e.text(), (c.line, c.col))
    }

    #[test]
    fn insert_and_escape() {
        assert_eq!(run("", "ihello<Esc>"), ("hello".into(), (0, 4)));
        assert_eq!(run("ac", "ab<Esc>"), ("abc".into(), (0, 1)));
        assert_eq!(run("one", "A two<Esc>"), ("one two".into(), (0, 6)));
        assert_eq!(run("  x", "Iy<Esc>"), ("  yx".into(), (0, 2)));
        assert_eq!(run("a\nb", "onew<Esc>"), ("a\nnew\nb".into(), (1, 2)));
        assert_eq!(run("a\nb", "jOnew<Esc>"), ("a\nnew\nb".into(), (1, 2)));
    }

    #[test]
    fn motions() {
        assert_eq!(run("hello world foo", "w").1, (0, 6));
        assert_eq!(run("hello world foo", "2w").1, (0, 12));
        assert_eq!(run("hello world foo", "$b").1, (0, 12));
        assert_eq!(run("hello world", "e").1, (0, 4));
        assert_eq!(run("foo.bar baz", "w").1, (0, 3), "punctuation is its own word");
        assert_eq!(run("abc\n  def", "j^").1, (1, 2));
        assert_eq!(run("a\nb\nc", "G").1, (2, 0));
        assert_eq!(run("a\nb\nc", "Ggg").1, (0, 0));
        assert_eq!(run("a\nb\nc", "2G").1, (1, 0));
        assert_eq!(run("abc", "$l").1, (0, 2), "normal mode stays on the last char");
    }

    #[test]
    fn column_is_kept_across_short_lines() {
        assert_eq!(run("abcdef\nx\nabcdef", "4ljj").1, (2, 4));
    }

    #[test]
    fn delete_and_change() {
        assert_eq!(run("hello world", "dw"), ("world".into(), (0, 0)));
        assert_eq!(run("hello world", "cwbye<Esc>"), ("bye world".into(), (0, 2)), "cw acts like ce");
        assert_eq!(run("one\ntwo", "$dw"), ("on\ntwo".into(), (0, 1)), "dw does not eat the line break");
        assert_eq!(run("a b c d", "d2w"), ("c d".into(), (0, 0)));
        assert_eq!(run("a b c d", "2dw"), ("c d".into(), (0, 0)));
        assert_eq!(run("hello", "x"), ("ello".into(), (0, 0)));
        assert_eq!(run("hello", "$X"), ("helo".into(), (0, 3)));
        assert_eq!(run("hello world", "wD"), ("hello ".into(), (0, 5)));
        assert_eq!(run("abc", "rx"), ("xbc".into(), (0, 0)));
    }

    #[test]
    fn linewise() {
        assert_eq!(run("a\nb\nc", "dd"), ("b\nc".into(), (0, 0)));
        assert_eq!(run("a\nb\nc", "Gdd"), ("a\nb".into(), (1, 0)), "the last line takes the break before it");
        assert_eq!(run("a\nb\nc", "2dd"), ("c".into(), (0, 0)));
        assert_eq!(run("a\nb\nc", "dj"), ("c".into(), (0, 0)));
        assert_eq!(run("  a\nb", "ccx<Esc>"), ("  x\nb".into(), (0, 2)), "cc keeps the indent");
        assert_eq!(run("a\nb", "yyp"), ("a\na\nb".into(), (1, 0)));
        assert_eq!(run("a\nb", "jyyP"), ("a\nb\nb".into(), (1, 0)));
        assert_eq!(run("a\nb", "Gyyp"), ("a\nb\nb".into(), (2, 0)));
    }

    #[test]
    fn yank_put_chars() {
        assert_eq!(run("ab", "ylp"), ("aab".into(), (0, 1)));
        assert_eq!(run("hello world", "yw$p"), ("hello worldhello ".into(), (0, 16)));
    }

    #[test]
    fn join() {
        assert_eq!(run("a\n   b", "J"), ("a b".into(), (0, 1)));
        assert_eq!(run("a\nb\nc", "3J"), ("a b c".into(), (0, 3)));
    }

    #[test]
    fn undo_redo() {
        assert_eq!(run("abc", "xxu").0, "bc");
        assert_eq!(run("abc", "xxuu").0, "abc");
        assert_eq!(run("abc", "xxuu<C-r>").0, "bc");
        assert_eq!(run("", "ihello world<Esc>u").0, "", "an insert session is one change");
        assert_eq!(run("abc", "xu2<C-r>").0, "bc");
    }

    #[test]
    fn escape_cancels_pending() {
        assert_eq!(run("abc", "d<Esc>x"), ("bc".into(), (0, 0)));
        assert_eq!(run("abc", "3<Esc>x"), ("bc".into(), (0, 0)));
    }
}

#[cfg(test)]
mod comment_tests {
    use crate::*;

    #[test]
    fn gc_comments_and_uncomments() {
        let mut e = Editor::from_str("fn a() {\n    one();\n\n    two();\n}\n");
        e.toggle_comment(1, 3, "//");
        assert_eq!(e.buf.line(1).trim_end(), "    // one();");
        assert_eq!(e.buf.line(2).trim_end(), "");
        assert_eq!(e.buf.line(3).trim_end(), "    // two();");
        e.toggle_comment(1, 3, "//");
        assert_eq!(e.buf.line(1).trim_end(), "    one();");
        assert_eq!(e.buf.line(3).trim_end(), "    two();");
        // one undo step brings the comments back
        e.feed("u");
        assert_eq!(e.buf.line(1).trim_end(), "    // one();");
    }
}

#[cfg(test)]
mod apply_tests {
    use crate::*;

    #[test]
    fn edits_apply_as_one_change() {
        let mut e = Editor::from_str("let foo = 1;\nfoo + foo\n");
        e.apply_edits(vec![
            (Pos { line: 0, col: 4 }, Pos { line: 0, col: 7 }, "bar".into()),
            (Pos { line: 1, col: 0 }, Pos { line: 1, col: 3 }, "bar".into()),
            (Pos { line: 1, col: 6 }, Pos { line: 1, col: 9 }, "bar".into()),
        ]);
        assert_eq!(e.buf.line(0).trim_end(), "let bar = 1;");
        assert_eq!(e.buf.line(1).trim_end(), "bar + bar");
        e.feed("u");
        assert_eq!(e.buf.line(1).trim_end(), "foo + foo");
    }
}

#[cfg(test)]
mod undo_tree_tests {
    use crate::*;

    #[test]
    fn history_survives_a_restart() {
        let mut e = Editor::from_str("a\n");
        e.feed("Ab<Esc>");
        e.feed("Ac<Esc>");
        e.feed("u");
        e.feed("Aé ü<Esc>"); // a branch, and text that is more than one byte a character
        let saved = e.history_save();
        // the file reopened: the same text gets its whole tree back, branches included
        let mut again = Editor::from_str(&e.text());
        assert!(again.history_load(&saved));
        assert_eq!(again.undo_states().len(), 4);
        assert_eq!(again.text_at(2), "abc\n");
        again.feed("u");
        assert_eq!(again.text(), "ab\n");
        // changed elsewhere meanwhile: the old history is not forced onto it
        let mut other = Editor::from_str("something else\n");
        assert!(!other.history_load(&saved));
        assert_eq!(other.undo_states().len(), 1);
    }

    #[test]
    fn deleted_text_is_found_in_the_history() {
        let mut e = Editor::from_str("keep\nthe Secret plan\nend\n");
        e.feed("jdd");
        e.feed("Gonew line<Esc>");
        let saved = e.history_save();
        let hits = Editor::history_search(&saved, "secret", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].2, "the Secret plan");
        // the state before the change that removed it still has it
        let before = e.undo_states()[hits[0].0].0;
        assert!(e.text_at(before).contains("the Secret plan"));
        assert!(!e.text().contains("Secret"));
        assert!(Editor::history_search(&saved, "SECRET", 10).is_empty(), "capitals match exactly");
        assert!(Editor::history_search(b"not a history", "x", 10).is_empty());
    }

    #[test]
    fn the_emacs_history_includes_what_was_just_typed() {
        // modeless, the editor is always "inserting": the typing still open must count
        let mut e = Editor::from_str("x\nold\n");
        e.set_emacs(true);
        e.set_cursor(Pos { line: 1, col: 0 });
        e.exec("kill-line", 1);
        "new".chars().for_each(|c| e.self_insert(c));
        let states = e.undo_states().len();
        assert_eq!(states, 3, "the kill and the typing are two steps");
        assert_eq!(e.text_at(0), "x\nold\n", "the first state has none of the later typing");
        assert_eq!(e.text_at(1), "x\n\n");
    }

    #[test]
    fn one_section_comes_back_from_the_past() {
        let mut e = Editor::from_str("# A\nold a\n# B\nold b\n");
        e.set_emacs(true);
        e.set_outline_kind(outline::OutlineKind::Markdown);
        e.set_cursor(Pos { line: 1, col: 0 });
        e.exec("kill-line", 1);
        "new a".chars().for_each(|c| e.self_insert(c));
        e.set_cursor(Pos { line: 3, col: 0 });
        e.exec("kill-line", 1);
        "new b".chars().for_each(|c| e.self_insert(c));
        assert_eq!(e.text(), "# A\nnew a\n# B\nnew b\n");
        // A as it was at the start; B keeps what was done since
        assert!(e.restore_section(0, "# A"));
        assert_eq!(e.text(), "# A\nold a\n# B\nnew b\n");
        e.exec("undo", 1);
        assert_eq!(e.text(), "# A\nnew a\n# B\nnew b\n", "one undo step");
    }

    #[test]
    fn branches_are_kept() {
        let mut e = Editor::from_str("a\n");
        e.feed("Ab<Esc>"); // 1: ab
        e.feed("Ac<Esc>"); // 2: abc
        e.feed("u"); // back to 1
        e.feed("Ad<Esc>"); // 3: abd, a branch beside 2
        assert_eq!(e.text(), "abd\n");
        assert_eq!(e.undo_states().iter().map(|s| s.0).collect::<Vec<_>>(), vec![0, 0, 1, 1]);
        assert_eq!(e.text_at(2), "abc\n");
        e.undo_goto(2);
        assert_eq!(e.text(), "abc\n");
        e.undo_goto(0);
        assert_eq!(e.text(), "a\n");
        e.feed("<C-r><C-r>"); // redo follows the branch last visited
        assert_eq!(e.text(), "abc\n");
        e.feed("g-"); // in time: 2 → 1
        assert_eq!(e.text(), "ab\n");
        e.feed("g+g+"); // 1 → 2 → 3
        assert_eq!(e.text(), "abd\n");
    }
}
