use crate::Span;

use super::{
    LexErrorMsg, Token,
    micro_regex::{Capture, CharIs, Many0, Many1, MicroRegex},
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
        "U" | "let" | "lazy" | "greedy" | "range" | "base" | "atomic" | "enable" | "disable" |
        "if" | "else" | "recursion" | "regex" | "test" | "call"
    );
}

static SINGLE_TOKEN_LOOKUP: [Option<Token>; 127] = const {
    let mut table = [const { None }; 127];
    table[b'^' as usize] = Some(Token::Caret);
    table[b'$' as usize] = Some(Token::Dollar);
    table[b'%' as usize] = Some(Token::Percent);
    table[b'<' as usize] = Some(Token::AngleLeft);
    table[b'>' as usize] = Some(Token::AngleRight);
    table[b'*' as usize] = Some(Token::Star);
    table[b'+' as usize] = Some(Token::Plus);
    table[b'?' as usize] = Some(Token::QuestionMark);
    table[b'|' as usize] = Some(Token::Pipe);
    table[b'&' as usize] = Some(Token::Ampersand);
    table[b':' as usize] = Some(Token::Colon);
    table[b')' as usize] = Some(Token::CloseParen);
    table[b'{' as usize] = Some(Token::OpenBrace);
    table[b'}' as usize] = Some(Token::CloseBrace);
    table[b',' as usize] = Some(Token::Comma);
    table[b'!' as usize] = Some(Token::Not);
    table[b'[' as usize] = Some(Token::OpenBracket);
    table[b']' as usize] = Some(Token::CloseBracket);
    table[b'-' as usize] = Some(Token::Dash);
    table[b'.' as usize] = Some(Token::Dot);
    table[b';' as usize] = Some(Token::Semicolon);
    table[b'=' as usize] = Some(Token::Equals);
    table
};

fn lookup_single(c: char) -> Option<Token> {
    let c = c as u32;
    if c < 128 { SINGLE_TOKEN_LOOKUP[c as usize] } else { None }
}

pub(crate) fn tokenize(mut input: &str) -> Vec<(Token, Span)> {
    let mut result = vec![];
    let mut offset = 0;

    while let Some((token, start, end)) = next_token(input) {
        result.push((token, Span::new(offset + start, offset + end)));
        input = &input[end..];
        offset += end;
    }

    result
}

pub fn next_token(mut input: &str) -> Option<(Token, usize, usize)> {
    let input_len = input.len();
    input = input.trim_start();
    while input.starts_with('#') {
        input = input.trim_start_matches(|c| c != '\n').trim_start();
    }
    let start = input_len - input.len();

    match input.chars().next() {
        None => None,
        Some(c) => {
            let (len, token) = consume_chain! {
                input, c;

                if input.starts_with(">>") => (2, Token::LookAhead);
                if input.starts_with("<<") => (2, Token::LookBehind);
                if input.starts_with("::") => (2, Token::DoubleColon);

                if let Some(token) = lookup_single(c) => (1, token);

                if c == '\'' => match input[1..].find('\'') {
                    Some(len_inner) => (len_inner + 2, Token::String),
                    None => (input.len(), Token::ErrorMsg(LexErrorMsg::UnclosedString)),
                };

                if c == '"' => match find_unescaped_quote(&input[1..]) {
                    Some(len_inner) => (len_inner + 2, Token::String),
                    None => (input.len(), Token::ErrorMsg(LexErrorMsg::UnclosedString)),
                };

                if let Some((len, _)) = (
                    'U',
                    Many0(CharIs(char::is_whitespace)),
                    '+',
                    Many0(CharIs(char::is_whitespace)),
                    Many1(CharIs(|c| c.is_alphanumeric() || c == '_')),
                ).is_start(input) => {
                    if input[1..len].trim_start_matches(|c: char| c == '+' || c.is_whitespace())
                        .contains(|c: char| !c.is_ascii_hexdigit()) {
                        (len, Token::ErrorMsg(LexErrorMsg::InvalidCodePoint))
                    } else {
                        (len, Token::CodePoint)
                    }
                };

                if let Some((len, _)) = (
                    Many1(CharIs(|c| c.is_ascii_digit()))
                ).is_start(input) => match (input.as_bytes(), len) {
                    ([b'0', ..], 2..) => (len, Token::ErrorMsg(LexErrorMsg::LeadingZero)),
                    _ => (len, Token::Number),
                };

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

            Some((token, start, start + len))
        }
    }
}

fn find_unescaped_quote(input: &str) -> Option<usize> {
    let mut s = input;

    loop {
        match s.find(['\\', '"']) {
            Some(n) => {
                if s.as_bytes()[n] == b'"' {
                    return Some(n + (input.len() - s.len()));
                }
                let next = s[n + 1..].chars().next()?;
                s = &s[n + 1 + next.len_utf8()..];
            }
            None => return None,
        }
    }
}

fn parse_backslash(input: &str) -> Option<(usize, LexErrorMsg)> {
    let hex = CharIs(|c| c.is_ascii_hexdigit());

    let ident = Many1(CharIs(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '+' | '_')));

    let after_gk: &[&dyn MicroRegex<Context = _>] = &[
        &('<', ident, '>'),
        &('{', ident, '}'),
        &('\'', ident, '\''),
        &(&["-", "+", ""][..], CharIs(|c| c.is_ascii_digit())),
    ];

    let after_p: &[&dyn MicroRegex<Context = _>] =
        &[&CharIs(|c| c.is_ascii_alphanumeric()), &('{', ident, '}'), &("{^", ident, '}')];

    let after_backslash: [&dyn MicroRegex<Context = _>; 6] = [
        &(&["u{", "x{"][..], Many1(hex), '}').ctx(LexErrorMsg::BackslashUnicode),
        &('u', hex, hex, hex, hex).ctx(LexErrorMsg::BackslashU4),
        &('x', hex, hex).ctx(LexErrorMsg::BackslashX2),
        &(&['k', 'g'][..], after_gk).ctx(LexErrorMsg::BackslashGK),
        &(&['p', 'P'][..], after_p).ctx(LexErrorMsg::BackslashProperty),
        &CharIs(|_| true).ctx(LexErrorMsg::Backslash),
    ];

    Capture(('\\', &after_backslash[..])).is_start(input).map(|(len, (_, err))| (len, err))
}

fn parse_special_group(input: &str) -> Option<(usize, LexErrorMsg)> {
    let ident = Many1(CharIs(|c| c.is_ascii_alphanumeric() || c == '-' || c == '+'));

    let after_open: &[&dyn MicroRegex<Context = _>] = &[
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

    Capture(("(?", after_open)).is_start(input).map(|(len, (_, err))| (len, err))
}
