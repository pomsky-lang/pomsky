use nom::{
    branch::alt,
    combinator::{cut, map, opt},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair},
    IResult, Parser,
};

use crate::{
    alternation::Alternation,
    boundary::Boundary,
    char_class::CharClass,
    error::{CharStringError, CodePointError, NumberError, ParseError, ParseErrorKind},
    group::{Capture, Group},
    lookaround::{Lookaround, LookaroundKind},
    repetition::{Greedy, Repetition, RepetitionKind},
    Rulex,
};

use super::{Token, Tokens};

pub(super) type PResult<'i, 'b, T> = IResult<Tokens<'i, 'b>, T, ParseError>;

pub(crate) fn parse(source: &str) -> Result<Rulex<'_>, ParseError> {
    let mut buf = Vec::new();
    let tokens = Tokens::tokenize(source, &mut buf)?;
    let (rest, rules) = parse_or(tokens)?;
    if rest.is_empty() {
        Ok(rules)
    } else {
        Err(ParseErrorKind::LeftoverTokens.at(rest.index()))
    }
}

pub(super) fn parse_or<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(separated_list1(Token::Pipe, parse_sequence), |mut rules| {
        if rules.len() == 1 {
            rules.pop().unwrap()
        } else {
            Alternation::new_rulex(rules)
        }
    })(tokens)
}

pub(super) fn parse_sequence<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(many1(parse_fixes), |mut rules| {
        if rules.len() == 1 {
            rules.pop().unwrap()
        } else {
            Rulex::Group(Group::new(rules, None))
        }
    })(tokens)
}

pub(super) fn parse_fixes<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    enum Suffix {
        Not,
        Repetition((RepetitionKind, Greedy)),
    }

    try_map(
        pair(
            parse_atom,
            many0(alt((
                map(Token::ExclamationMark, |_| Suffix::Not),
                map(parse_repetition, Suffix::Repetition),
            ))),
        ),
        |(mut rule, suffixes)| {
            for suffix in suffixes {
                rule = match suffix {
                    Suffix::Not => Rulex::negate(rule).ok_or(ParseErrorKind::InvalidNot)?,
                    Suffix::Repetition((kind, greedy)) => {
                        Rulex::Repetition(Box::new(Repetition::new(rule, kind, greedy)))
                    }
                }
            }
            Ok(rule)
        },
        nom::Err::Failure,
    )(tokens)
}

pub(super) fn parse_repetition<'i, 'b>(
    tokens: Tokens<'i, 'b>,
) -> PResult<'i, 'b, (RepetitionKind, Greedy)> {
    pair(
        alt((
            map(Token::QuestionMark, |_| RepetitionKind::zero_one()),
            map(Token::Star, |_| RepetitionKind::zero_inf()),
            map(Token::Plus, |_| RepetitionKind::one_inf()),
            parse_braced_repetition,
        )),
        map(opt("greedy"), |a| match a {
            Some(_) => Greedy::Yes,
            None => Greedy::No,
        }),
    )(tokens)
}

pub(super) fn parse_braced_repetition<'i, 'b>(
    tokens: Tokens<'i, 'b>,
) -> PResult<'i, 'b, RepetitionKind> {
    delimited(
        Token::OpenBrace,
        cut(alt((
            try_map(
                separated_pair(
                    opt(parse_repetition_count),
                    Token::Comma,
                    opt(parse_repetition_count),
                ),
                |(lower, upper)| Ok(RepetitionKind::try_from((lower.unwrap_or(0), upper))?),
                nom::Err::Failure,
            ),
            map(parse_repetition_count, RepetitionKind::fixed),
        ))),
        cut(Token::CloseBrace),
    )(tokens)
}

fn parse_repetition_count<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, u32> {
    try_map(
        Token::RepetitionCount,
        |n| str::parse(n).map_err(|_| ParseErrorKind::Number(NumberError::TooLarge)),
        nom::Err::Failure,
    )(tokens)
}

pub(super) fn parse_atom<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    alt((
        parse_group,
        parse_chars,
        parse_string,
        parse_char_range_class,
        parse_char_word_class,
        parse_boundary,
        parse_lookaround,
        err(|| ParseErrorKind::Expected("expression")),
    ))(tokens)
}

pub(super) fn parse_group<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(
        pair(
            opt(parse_capture),
            delimited(Token::OpenParen, opt(parse_or), cut(Token::CloseParen)),
        ),
        |(capture, rule)| match (capture, rule) {
            (None, Some(rule)) => rule,
            (capture, Some(Rulex::Group(mut g))) => {
                g.set_capture(capture);
                Rulex::Group(g)
            }
            (capture, Some(rule)) => Rulex::Group(Group::new(vec![rule], capture)),
            (capture, None) => Rulex::Group(Group::new(vec![], capture)),
        },
    )(tokens)
}

pub(super) fn parse_capture<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Capture<'i>> {
    map(pair(Token::Colon, opt(Token::GroupName)), |(_, name)| {
        Capture::new(name)
    })(tokens)
}

pub(super) fn parse_string<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(alt((Token::DoubleString, Token::SingleString)), |s| {
        Rulex::Literal(strip_first_last(s))
    })(tokens)
}

pub(super) fn parse_chars<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    fn parse_char<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, char> {
        alt((
            parse_code_point,
            parse_char_string,
            err(|| ParseErrorKind::ExpectedCodePointOrChar),
        ))(tokens)
    }

    try_map(
        pair(parse_char, opt(preceded(Token::Dash, cut(parse_char)))),
        |(c1, c2)| {
            let cc = CharClass::try_from_range(c1, c2.unwrap_or(c1))
                .ok_or(ParseErrorKind::CodePoint(CodePointError::InvalidRange))?;
            Ok(Rulex::CharClass(cc))
        },
        nom::Err::Failure,
    )(tokens)
}

pub(super) fn parse_char_string<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, char> {
    try_map(
        alt((Token::SingleString, Token::DoubleString)),
        |s| {
            let s = strip_first_last(s);
            let mut iter = s.chars();
            match iter.next() {
                Some(c) if matches!(iter.next(), None) => Ok(c),
                _ => Err(ParseErrorKind::CharString(CharStringError::Invalid)),
            }
        },
        nom::Err::Error,
    )(tokens)
}

pub(super) fn parse_code_point<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, char> {
    try_map(
        Token::CodePoint,
        |s| {
            let hex = &s[2..];
            if hex.len() > 6 {
                Err(ParseErrorKind::CodePoint(CodePointError::Invalid))
            } else {
                u32::from_str_radix(hex, 16)
                    .ok()
                    .and_then(|n| char::try_from(n).ok())
                    .ok_or(ParseErrorKind::CodePoint(CodePointError::Invalid))
            }
        },
        nom::Err::Failure,
    )(tokens)
}

pub(super) fn parse_char_range_class<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(Token::CharClass, |s| {
        Rulex::CharClass(CharClass::from_chars(strip_first_last(s)))
    })(tokens)
}

pub(super) fn parse_char_word_class<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(Token::CWord, |s| {
        Rulex::CharClass(CharClass::from_named(strip_first_last(s)))
    })(tokens)
}

pub(super) fn parse_boundary<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(
        alt((
            map(Token::BStart, |_| Boundary::Start),
            map(Token::BEnd, |_| Boundary::End),
            map(Token::BWord, |_| Boundary::Word),
            map(Token::BNotWord, |_| Boundary::NotWord),
        )),
        Rulex::Boundary,
    )(tokens)
}

pub(super) fn parse_lookaround<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(
        pair(
            alt((
                map(Token::LookAhead, |_| LookaroundKind::Ahead),
                map(Token::LookBehind, |_| LookaroundKind::Behind),
                map(Token::LookAheadNeg, |_| LookaroundKind::AheadNegative),
                map(Token::LookBehindNeg, |_| LookaroundKind::BehindNegative),
            )),
            cut(parse_atom),
        ),
        |(kind, rule)| Rulex::Lookaround(Box::new(Lookaround::new(rule, kind))),
    )(tokens)
}

fn strip_first_last(s: &str) -> &str {
    &s[1..s.len() - 1]
}

fn try_map<'i, 'b, O1, O2, P, M, EM>(
    mut parser: P,
    mut map: M,
    err_kind: EM,
) -> impl FnMut(Tokens<'i, 'b>) -> IResult<Tokens<'i, 'b>, O2, ParseError>
where
    P: Parser<Tokens<'i, 'b>, O1, ParseError>,
    M: FnMut(O1) -> Result<O2, ParseErrorKind>,
    EM: Copy + FnOnce(ParseError) -> nom::Err<ParseError>,
{
    move |input| {
        let (rest, o) = parser.parse(input.clone())?;
        Ok((
            rest,
            map(o).map_err(|e| e.at(input.index())).map_err(err_kind)?,
        ))
    }
}

fn err<'i, 'b, T>(
    mut error_fn: impl FnMut() -> ParseErrorKind,
) -> impl FnMut(Tokens<'i, 'b>) -> IResult<Tokens<'i, 'b>, T, ParseError> {
    move |input| Err(nom::Err::Error(error_fn().at(input.index())))
}
