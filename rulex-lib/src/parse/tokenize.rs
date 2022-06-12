use crate::{parse::ParseErrorMsg, span::Span};

use super::Token;

macro_rules! consume_chain {
    (
        $input:ident, $c:ident;
        if $cond:expr => $result:expr ; $($rest:tt)*
    ) => {
        if $cond {
            $result
        } else {
            consume_chain!($input, $c; $($rest)*)
        }
    };
    (
        $input:ident, $c:ident;
        if let $pat:pat = $test:expr => $result:expr ; $($rest:tt)*
    ) => {
        if let $pat = $test {
            $result
        } else {
            consume_chain!($input, $c; $($rest)*)
        }
    };
    (
        $input:ident, $c:ident;
    ) => {
        {
            (($c.len_utf8(), Token::Error))
        }
    }
}

pub(crate) fn tokenize(mut input: &str) -> Vec<(Token, Span)> {
    let mut result = vec![];
    let mut offset = 0;

    loop {
        let input_len = input.len();
        input = input.trim_start();
        while input.starts_with('#') {
            input = input.trim_start_matches(|c| c != '\n').trim_start();
        }
        offset += input_len - input.len();

        match input.chars().next() {
            None => break,
            Some(c) => {
                let (len, token) = consume_chain! {
                    input, c;

                    if input.starts_with("<%") => (2, Token::BStart);
                    if input.starts_with("%>") => (2, Token::BEnd);
                    if input.starts_with(">>") => (2, Token::LookAhead);
                    if input.starts_with("<<") => (2, Token::LookBehind);
                    if input.starts_with("::") => (2, Token::Backref);

                    if c == '%' => (1, Token::BWord);
                    if c == '*' => (1, Token::Star);
                    if c == '+' => (1, Token::Plus);
                    if c == '?' => (1, Token::QuestionMark);
                    if c == '|' => (1, Token::Pipe);
                    if c == ':' => (1, Token::Colon);
                    if c == ')' => (1, Token::CloseParen);
                    if c == '{' => (1, Token::OpenBrace);
                    if c == '}' => (1, Token::CloseBrace);
                    if c == ',' => (1, Token::Comma);
                    if c == '!' => (1, Token::Not);
                    if c == '[' => (1, Token::OpenBracket);
                    if c == '-' => (1, Token::Dash);
                    if c == ']' => (1, Token::CloseBracket);
                    if c == '.' => (1, Token::Dot);
                    if c == ';' => (1, Token::Semicolon);
                    if c == '=' => (1, Token::Equals);

                    if c == '\'' => match input[1..].find('\'') {
                        Some(len_inner) => (len_inner + 2, Token::String),
                        None => (input.len(), Token::ErrorMsg(ParseErrorMsg::UnclosedString)),
                    };

                    if c == '"' => match find_unescaped_quote(&input[1..]) {
                        Some(len_inner) => (len_inner + 2, Token::String),
                        None => (input.len(), Token::ErrorMsg(ParseErrorMsg::UnclosedString)),
                    };

                    if let Some(rest) = input.strip_prefix("U+") => {
                        match rest.find(|c: char| !c.is_ascii_hexdigit()) {
                            Some(0) => (1, Token::Error),
                            Some(len_inner) => (len_inner + 2, Token::CodePoint),
                            None => (input.len(), Token::CodePoint),
                        }
                    };

                    if matches!(c, '0'..='9') => (
                        input.find(|c: char| !matches!(c, '0'..='9')).unwrap_or(input.len()),
                        Token::Number,
                    );

                    if c.is_alphabetic() || c == '_' => (
                        input.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(input.len()),
                        Token::Identifier,
                    );

                    if c == '^' => (1, Token::ErrorMsg(ParseErrorMsg::Caret));
                    if c == '$' => (1, Token::ErrorMsg(ParseErrorMsg::Dollar));

                    if let Some(rest) = input.strip_prefix("(?") => (
                        match rest.chars().next() {
                            Some('<') => {
                                let name_len = rest.chars()
                                    .skip(1)
                                    .take_while(char::is_ascii_alphanumeric)
                                    .count();

                                if name_len > 0 && matches!(rest.chars().nth(1 + name_len), Some('>')) {
                                    4 + name_len
                                } else if let Some('=' | '!') = rest.chars().nth(1) {
                                    4
                                } else {
                                    3
                                }
                            }
                            Some('P') if matches!(rest.chars().nth(1), Some('<')) => {
                                let name_len = rest.chars()
                                    .skip(2)
                                    .take_while(char::is_ascii_alphanumeric)
                                    .count();

                                if name_len > 0 && matches!(rest.chars().nth(2 + name_len), Some('>')) {
                                    5 + name_len
                                } else {
                                    4
                                }
                            },
                            Some('>' | '!' | ':' | '=' | '(' | '|') => 3,
                            _ => 2,
                        },
                        Token::ErrorMsg(ParseErrorMsg::SpecialGroup),
                    );
                    if c == '(' => (1, Token::OpenParen);

                    if c == '\\' => {
                        if input.starts_with("\\u{") || input.starts_with("\\x{") {
                            match input[3..].find('}') {
                                Some(len) => (len + 4, Token::ErrorMsg(ParseErrorMsg::BackslashUnicode)),
                                None => (2, Token::ErrorMsg(ParseErrorMsg::Backslash)),
                            }
                        } else if let Some(rest) = input.strip_prefix("\\u") {
                            match rest.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(rest.len()) {
                                4.. => (6, Token::ErrorMsg(ParseErrorMsg::BackslashU4)),
                                _ => (2, Token::ErrorMsg(ParseErrorMsg::Backslash)),
                            }
                        } else if let Some(rest) = input.strip_prefix("\\x") {
                            match rest.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(rest.len()) {
                                2.. => (4, Token::ErrorMsg(ParseErrorMsg::BackslashX2)),
                                _ => (2, Token::ErrorMsg(ParseErrorMsg::Backslash)),
                            }
                        } else if let Some(rest) = input.strip_prefix("\\k<") {
                            match rest.find('>') {
                                Some(len) => (len + 4, Token::ErrorMsg(ParseErrorMsg::BackslashK)),
                                _ => (2, Token::ErrorMsg(ParseErrorMsg::Backslash)),
                            }
                        } else if input.len() >= 2 {
                            (2, Token::ErrorMsg(ParseErrorMsg::Backslash))
                        } else {
                            (1, Token::Error)
                        }
                    };
                };

                let start = offset;
                offset += len;
                input = &input[len..];
                result.push((token, Span::new(start, offset)));
            }
        }
    }

    result
}

fn find_unescaped_quote(input: &str) -> Option<usize> {
    let mut s = input;

    loop {
        match s.find(|c| c == '\\' || c == '"') {
            Some(n) => {
                if s.as_bytes()[n] == b'"' {
                    return Some(n + (input.len() - s.len()));
                } else if n + 2 <= s.len() {
                    s = &s[n + 2..];
                } else {
                    return None;
                }
            }
            None => return None,
        }
    }
}
