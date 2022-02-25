use nom::{
    branch::alt,
    combinator::{cut, map, opt, value},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair},
    IResult, Parser,
};

use crate::{
    alternation::Alternation,
    boundary::Boundary,
    char_class::{CharClass, CharGroup},
    error::{
        CharClassError, CharStringError, CodePointError, NumberError, ParseError, ParseErrorKind,
    },
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
    map(separated_list0(Token::Pipe, parse_sequence), |mut rules| {
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
    alt((
        map(pair(parse_lookaround, parse_or), |(kind, rule)| {
            Rulex::Lookaround(Box::new(Lookaround::new(rule, kind)))
        }),
        map(
            pair(parse_atom, many0(parse_repetition)),
            |(mut rule, repetitions)| {
                for (kind, greedy) in repetitions {
                    rule = Rulex::Repetition(Box::new(Repetition::new(rule, kind, greedy)));
                }
                rule
            },
        ),
    ))(tokens)
}

pub(super) fn parse_lookaround<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, LookaroundKind> {
    alt((
        value(LookaroundKind::Ahead, Token::LookAhead),
        value(LookaroundKind::Behind, Token::LookBehind),
        value(LookaroundKind::AheadNegative, pair("not", Token::LookAhead)),
        value(
            LookaroundKind::BehindNegative,
            pair("not", Token::LookBehind),
        ),
    ))(tokens)
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
    fn str_to_u32(s: &str) -> Result<u32, ParseErrorKind> {
        str::parse(s).map_err(|_| ParseErrorKind::Number(NumberError::TooLarge))
    }

    fn parse_u32<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, u32> {
        try_map(Token::Number, str_to_u32, nom::Err::Failure)(tokens)
    }

    delimited(
        Token::OpenBrace,
        cut(alt((
            try_map(
                separated_pair(opt(parse_u32), Token::Comma, opt(parse_u32)),
                |(lower, upper)| Ok(RepetitionKind::try_from((lower.unwrap_or(0), upper))?),
                nom::Err::Failure,
            ),
            map(parse_u32, RepetitionKind::fixed),
        ))),
        cut(Token::CloseBrace),
    )(tokens)
}

pub(super) fn parse_atom<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    alt((
        parse_group,
        parse_string,
        parse_char_class,
        parse_boundary,
        map(parse_code_point, |c| {
            Rulex::CharClass(CharGroup::from_char(c).into())
        }),
        err(|| ParseErrorKind::Expected("expression")),
    ))(tokens)
}

pub(super) fn parse_group<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    fn parse_capture<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Capture<'i>> {
        map(pair(Token::Colon, opt(Token::Identifier)), |(_, name)| {
            Capture::new(name)
        })(tokens)
    }

    map(
        pair(
            opt(parse_capture),
            delimited(Token::OpenParen, parse_or, cut(Token::CloseParen)),
        ),
        |(capture, rule)| match (capture, rule) {
            (None, rule) => rule,
            (capture, Rulex::Group(mut g)) => {
                g.set_capture(capture);
                Rulex::Group(g)
            }
            (capture, rule) => Rulex::Group(Group::new(vec![rule], capture)),
        },
    )(tokens)
}

pub(super) fn parse_string<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(Token::String, |s| Rulex::Literal(strip_first_last(s)))(tokens)
}

pub(super) fn parse_char_class<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    pub(super) fn parse_single_char_string<'i, 'b>(
        tokens: Tokens<'i, 'b>,
    ) -> PResult<'i, 'b, char> {
        try_map(
            Token::String,
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

    fn parse_single_char<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, char> {
        alt((
            parse_single_char_string,
            parse_code_point,
            err(|| ParseErrorKind::ExpectedCodePointOrChar),
        ))(tokens)
    }

    fn parse_char_or_range<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, CharGroup<'i>> {
        try_map(
            pair(
                parse_single_char,
                opt(preceded(Token::Dash, cut(parse_single_char))),
            ),
            |(first, last)| match (first, last) {
                (first, None) => Ok(CharGroup::from_char(first)),
                (first, Some(last)) => CharGroup::try_from_range(first, last)
                    .ok_or(ParseErrorKind::CodePoint(CodePointError::InvalidRange)),
            },
            nom::Err::Failure,
        )(tokens)
    }

    fn parse_chars<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, CharGroup<'i>> {
        map(Token::String, |s| {
            CharGroup::from_chars(strip_first_last(s))
        })(tokens)
    }

    fn parse_char_group<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, CharGroup<'i>> {
        try_map(
            many0(alt((
                parse_char_or_range,
                parse_chars,
                value(CharGroup::Dot, Token::Dot),
                map(Token::Identifier, CharGroup::from_group_name),
                err(|| ParseErrorKind::CharClass(CharClassError::Invalid)),
            ))),
            |ranges| {
                let mut iter = ranges.into_iter();
                let mut class = iter
                    .next()
                    .ok_or(ParseErrorKind::CharClass(CharClassError::Empty))?;

                for range in iter {
                    class.add(range).map_err(ParseErrorKind::CharClass)?;
                }
                Ok(class)
            },
            nom::Err::Failure,
        )(tokens)
    }

    delimited(
        Token::OpenBracket,
        map(
            pair(opt("not"), parse_char_group),
            |(not, class): (_, CharGroup<'_>)| {
                let mut class: CharClass<'_> = class.into();
                if not.is_some() {
                    class.negate();
                }
                Rulex::CharClass(class)
            },
        ),
        Token::CloseBracket,
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

pub(super) fn parse_boundary<'i, 'b>(tokens: Tokens<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(
        alt((
            value(Boundary::Start, Token::BStart),
            value(Boundary::End, Token::BEnd),
            value(Boundary::Word, Token::BWord),
            value(Boundary::NotWord, pair("not", Token::BWord)),
        )),
        Rulex::Boundary,
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
