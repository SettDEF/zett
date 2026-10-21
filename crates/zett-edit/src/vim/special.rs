//! Keys beyond letters: page keys, F1 and Help, Insert, Undo, Ctrl and Shift with arrows, Home and
//! End, CTRL-Tab, and the mouse buttons (at the place the window gives with set_mouse).

use super::*;
use crate::keys::MouseButton;

impl Editor {
    /// The window says where the mouse is before it hands over a mouse key.
    pub fn set_mouse(&mut self, at: crate::Pos) {
        self.vim.mouse = self.buf.to_char(at).min(self.buf.len_chars());
    }

    /// A key that is not a character; false when it is none of these.
    pub(crate) fn vim_special(&mut self, k: Key) -> bool {
        let g = self.prefix == Some('g');
        let bracket = self.prefix.filter(|p| matches!(p, '[' | ']'));
        match (k.code, k.ctrl, k.shift) {
            (Code::PageUp, _, _) | (Code::Up, false, true) => {
                self.vim_ctrl('b');
            }
            (Code::PageDown, _, _) | (Code::Down, false, true) => {
                self.vim_ctrl('f');
            }
            (Code::Left, false, true) | (Code::Left, true, false) => self.normal_key(Key::ch(if k.ctrl { 'B' } else { 'b' })),
            (Code::Right, false, true) | (Code::Right, true, false) => self.normal_key(Key::ch(if k.ctrl { 'W' } else { 'w' })),
            (Code::Home, true, _) => {
                self.count.get_or_insert(1);
                self.prefix = Some('g');
                self.normal_key(Key::ch('g'));
            }
            (Code::End, true, _) => self.normal_key(Key::ch('G')),
            (Code::Tab, true, _) => self.vim.requests.push(Request::Alternate(None)),
            (Code::Insert, _, _) => self.normal_key(Key::ch('i')),
            (Code::Undo, _, _) => self.normal_key(Key::ch('u')),
            (Code::F(1), _, _) | (Code::Help, _, _) => self.vim.requests.push(Request::Ex("help".into())),
            (Code::Up | Code::Down | Code::Home | Code::End, false, false) if g => {
                // g<Up> g<Down> g<Home> g<End>: by screen lines
                self.prefix = None;
                let c = match k.code { Code::Up => 'k', Code::Down => 'j', Code::Home => '0', _ => '$' };
                return self.vim_g(c);
            }
            (Code::Mouse(b), ctrl, shift) => {
                self.prefix = None;
                let at = self.vim.mouse;
                match (b, ctrl, shift, g, bracket) {
                    (MouseButton::Middle, _, _, _, Some(p)) => {
                        // [<MiddleMouse> ]<MiddleMouse>: [p ]p at the mouse
                        self.cursor = at;
                        self.clamp();
                        return self.vim_bracket(p, 'p');
                    }
                    (MouseButton::Left, false, false, false, _) => {
                        if matches!(self.mode, Mode::Visual | Mode::VisualLine) {
                            self.leave_visual();
                        }
                        self.cursor = at;
                        self.clamp();
                        self.want_col = self.cursor().col;
                    }
                    (MouseButton::Right, false, false, false, _) => {
                        // extend a selection to the mouse (or start one)
                        if !matches!(self.mode, Mode::Visual | Mode::VisualLine) {
                            self.anchor = self.cursor;
                            self.mode = Mode::Visual;
                        }
                        self.cursor = at;
                    }
                    (MouseButton::Middle, false, false, false, _) => {
                        // put the selection (the "* register) at the mouse
                        self.cursor = at;
                        self.clamp();
                        self.vim_use_reg('*');
                        self.put(false, 1);
                    }
                    (MouseButton::Left, true, _, _, _) | (MouseButton::Left, _, _, true, _) => {
                        self.cursor = at;
                        self.clamp();
                        self.vim.requests.push(Request::Definition);
                    }
                    (MouseButton::Right, true, _, _, _) | (MouseButton::Right, _, _, true, _) => self.vim.requests.push(Request::Back),
                    (MouseButton::Left, _, true, _, _) | (MouseButton::Right, _, true, _, _) => {
                        self.cursor = at;
                        self.clamp();
                        self.vim_star(b == MouseButton::Left, true);
                    }
                    _ => {
                        self.vim_unknown();
                    }
                }
            }
            _ => return false,
        }
        true
    }
}
