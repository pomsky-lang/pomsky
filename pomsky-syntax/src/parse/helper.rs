use std::borrow::Cow;

use crate::error::{NumberError, ParseErrorKind};

pub(super) fn parse_quoted_text(input: &str) -> Result<Cow<'_, str>, ParseErrorKind> {
    Ok(match input.as_bytes()[0] {
        b'"' => {
            let mut s = strip_first_last(input);
            let mut buf = String::new();

            loop {
                let mut chars = s.chars();
                let char_len;
                match chars.next() {
                    Some('\\') => {
                        char_len = 1;
                        match chars.next() {
                            Some('\\') => {
                                buf.push('\\');
                                s = &s[1..];
                            }
                            Some('"') => {
                                buf.push('"');
                                s = &s[1..];
                            }
                            _ => {
                                return Err(ParseErrorKind::InvalidEscapeInStringAt(
                                    input.len() - s.len(),
                                ));
                            }
                        }
                    }
                    Some(c) => {
                        char_len = c.len_utf8();
                        buf.push(c)
                    }
                    None => break,
                }
                s = &s[char_len..];
            }
            Cow::Owned(buf)
        }
        _ => Cow::Borrowed(strip_first_last(input)),
    })
}

pub(super) fn strip_first_last(s: &str) -> &str {
    &s[1..s.len() - 1]
}

pub(super) fn parse_number(src: &str, radix: u8) -> Result<Vec<u8>, NumberError> {
    let mut digits = Vec::with_capacity(src.len());
    for c in src.bytes() {
        let n = match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'z' => c - b'a' + 10,
            b'A'..=b'Z' => c - b'A' + 10,
            _ => return Err(NumberError::InvalidDigit),
        };
        if n >= radix {
            return Err(NumberError::InvalidDigit);
        }
        digits.push(n);
    }
    Ok(digits)
}
