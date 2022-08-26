use crate::{parse::LexErrorMsg, span::Span};

use super::{
    micro_regex::{Capture, CharIs, Many0, Many1, MicroRegex},
    Token,
};

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

macro_rules! reserved_word_pattern {
    {} => (
        "let" | "lazy" | "greedy" | "range" | "base" | "atomic" | "enable" | "disable" |
        "if" | "else" | "recursion"
    );
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

                    if c == '^' => (1, Token::Caret);
                    if c == '$' => (1, Token::Dollar);
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
                        None => (input.len(), Token::ErrorMsg(LexErrorMsg::UnclosedString)),
                    };

                    if c == '"' => match find_unescaped_quote(&input[1..]) {
                        Some(len_inner) => (len_inner + 2, Token::String),
                        None => (input.len(), Token::ErrorMsg(LexErrorMsg::UnclosedString)),
                    };

                    if let Some((len, _)) = (
                        "U+", Many1(CharIs(|c| c.is_ascii_hexdigit())),
                    ).is_start(input) => (len, Token::CodePoint);

                    if let Some((len, _)) = (
                        Many1(CharIs(|c| c.is_ascii_digit()))
                    ).is_start(input) => (len, Token::Number);

                    if let Some((len, _)) = (
                        CharIs(|c| c.is_alphabetic() || c == '_'),
                        Many0(CharIs(|c| c.is_alphanumeric() || c == '_'))
                    ).is_start(input) => match &input[..len] {
                        reserved_word_pattern!() => (len, Token::ReservedName),
                        _ => (len, Token::Identifier),
                    };

                    if let Some((len, err)) = parse_special_group(input) => (len, Token::ErrorMsg(err));

                    if c == '(' => (1, Token::OpenParen);

                    if let Some((len, err)) = parse_backslash(input) => (len, Token::ErrorMsg(err));
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
                } else if let Some(next) = s[n + 1..].chars().next() {
                    s = &s[n + 1 + next.len_utf8()..];
                } else {
                    return None;
                }
            }
            None => return None,
        }
    }
}

fn parse_backslash(input: &str) -> Option<(usize, LexErrorMsg)> {
    let hex = CharIs(|c| c.is_ascii_hexdigit());

    let ident = Many1(CharIs(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '+' | '_')));

    let after_gk: [&dyn MicroRegex<Context = _>; 4] = [
        &('<', ident, '>'),
        &('{', ident, '}'),
        &('\'', ident, '\''),
        &(&["-", "+", ""][..], CharIs(|c| c.is_ascii_digit())),
    ];

    let after_p: [&dyn MicroRegex<Context = _>; 3] =
        [&CharIs(|c| c.is_ascii_alphanumeric()), &('{', ident, '}'), &("{^", ident, '}')];

    let after_backslash: [&dyn MicroRegex<Context = _>; 6] = [
        &(&["u{", "x{"][..], Many1(hex), '}').ctx(LexErrorMsg::BackslashUnicode),
        &('u', hex, hex, hex, hex).ctx(LexErrorMsg::BackslashU4),
        &('x', hex, hex).ctx(LexErrorMsg::BackslashX2),
        &(&['k', 'g'][..], &after_gk[..]).ctx(LexErrorMsg::BackslashGK),
        &(&['p', 'P'][..], &after_p[..]).ctx(LexErrorMsg::BackslashProperty),
        &CharIs(|_| true).ctx(LexErrorMsg::Backslash),
    ];

    Capture(('\\', &after_backslash[..])).is_start(input).map(|(len, (_, err))| (len, err))
}

fn parse_special_group(input: &str) -> Option<(usize, LexErrorMsg)> {
    let ident = Many1(CharIs(|c| c.is_ascii_alphanumeric() || c == '-' || c == '+'));

    let after_open: [&dyn MicroRegex<Context = _>; 14] = [
        &':'.ctx(LexErrorMsg::GroupNonCapturing),
        &'='.ctx(LexErrorMsg::GroupLookahead),
        &'!'.ctx(LexErrorMsg::GroupLookaheadNeg),
        &'>'.ctx(LexErrorMsg::GroupAtomic),
        &'('.ctx(LexErrorMsg::GroupConditional),
        &'|'.ctx(LexErrorMsg::GroupBranchReset),
        &"<=".ctx(LexErrorMsg::GroupLookbehind),
        &"<!".ctx(LexErrorMsg::GroupLookbehindNeg),
        &(&["P<", "<"][..], ident, '>').ctx(LexErrorMsg::GroupNamedCapture),
        &('\'', ident, '\'').ctx(LexErrorMsg::GroupNamedCapture),
        &("P=", ident, ')').ctx(LexErrorMsg::GroupPcreBackreference),
        &(&["P>", "&"][..]).ctx(LexErrorMsg::GroupSubroutineCall),
        &('#', Many0(CharIs(|c| c != ')')), ')').ctx(LexErrorMsg::GroupComment),
        &"".ctx(LexErrorMsg::GroupOther),
    ];

    Capture(("(?", &after_open[..])).is_start(input).map(|(len, (_, err))| (len, err))
}
