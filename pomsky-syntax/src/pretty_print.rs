use std::fmt::{Debug, Display};

pub(crate) struct PrettyPrinter {
    buf: String,
    indent: usize,
}

impl PrettyPrinter {
    pub(crate) fn new() -> Self {
        PrettyPrinter { buf: String::new(), indent: 0 }
    }

    pub(crate) fn finish(self) -> String {
        self.buf
    }

    fn add_spaces(&mut self, n: usize) {
        self.buf.extend(std::iter::repeat(' ').take(n));
    }

    pub(crate) fn write(&mut self, s: &str) {
        self.extend(s.chars());
    }

    pub(crate) fn extend(&mut self, s: impl IntoIterator<Item = char>) {
        for c in s.into_iter() {
            self.buf.push(c);

            if c == '\n' {
                self.add_spaces(self.indent);
            }
        }
    }

    pub(crate) fn push_str(&mut self, s: &str) {
        debug_assert!(!s.contains('\n'));
        self.buf.push_str(s);
    }

    pub(crate) fn push(&mut self, c: char) {
        debug_assert!(c != '\n');
        self.buf.push(c);
    }

    pub(crate) fn write_fmt(&mut self, v: impl Display) {
        self.write(&format!("{v}"))
    }

    pub(crate) fn write_debug(&mut self, v: impl Debug) {
        self.write(&format!("{v:?}"))
    }

    pub(crate) fn increase_indentation(&mut self, n: usize) {
        self.indent += n;
    }

    pub(crate) fn decrease_indentation(&mut self, n: usize) {
        self.indent = self.indent.saturating_sub(n);
    }

    pub(crate) fn start_indentation(&mut self, paren: &str) {
        debug_assert!(!paren.contains('\n'));
        self.buf.push_str(paren);
        self.indent += 2;
        self.buf.push('\n');
        self.buf.extend(std::iter::repeat(' ').take(self.indent));
    }

    pub(crate) fn end_indentation(&mut self, paren: &str) {
        debug_assert!(!paren.contains('\n'));
        self.indent = self.indent.saturating_sub(2);
        self.buf.push('\n');
        self.buf.extend(std::iter::repeat(' ').take(self.indent));
        self.buf.push_str(paren);
    }

    pub(crate) fn pretty_print_char(&mut self, char: char) {
        use std::fmt::Write;

        // fast path first
        if char == ' ' || char.is_ascii_graphic() {
            self.buf.push('\'');
            self.buf.push(char);
            self.buf.push('\'');
        } else if char < '\u{1F}' {
            self.buf.push_str("U+");
            let _ = write!(self.buf, "{:X}", char as u32);
        } else {
            let prev_len = self.buf.len();
            let _ = write!(self.buf, "{:?}", char);
            let new_len = self.buf.len();

            if new_len - prev_len > char.len_utf8() + 2 {
                let _ = self.buf.drain(prev_len..);
                self.buf.push_str("U+");
                let _ = write!(self.buf, "{:X}", char as u32);
            }
        }
    }
}
