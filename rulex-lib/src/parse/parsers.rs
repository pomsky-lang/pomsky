use nom::{
    branch::alt,
    combinator::{cut, map, opt, value},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, separated_pair},
    IResult, Parser,
};

use crate::{
    alternation::Alternation,
    boundary::Boundary,
    char_class::CharClass,
    char_group::CharGroup,
    error::{
        CharClassError, CharStringError, CodePointError, NumberError, ParseError, ParseErrorKind,
    },
    group::{Capture, Group},
    lookaround::{Lookaround, LookaroundKind},
    repetition::{Greedy, Repetition, RepetitionKind},
    Rulex,
};

use super::{Input, Token};

pub(super) type PResult<'i, 'b, T> = IResult<Input<'i, 'b>, T, ParseError>;

pub(crate) fn parse(source: &str) -> Result<Rulex<'_>, ParseError> {
    let mut buf = Vec::new();
    let tokens = Input::tokenize(source, &mut buf)?;
    let (rest, rules) = parse_or(tokens)?;
    if rest.is_empty() {
        Ok(rules)
    } else {
        Err(ParseErrorKind::LeftoverTokens.at(rest.index()))
    }
}

pub(super) fn parse_or<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(separated_list0(Token::Pipe, parse_sequence), |mut rules| {
        if rules.len() == 1 {
            rules.pop().unwrap()
        } else {
            Alternation::new_rulex(rules)
        }
    })(input)
}

pub(super) fn parse_sequence<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(many1(parse_fixes), |mut rules| {
        if rules.len() == 1 {
            rules.pop().unwrap()
        } else {
            Rulex::Group(Group::new(rules, None))
        }
    })(input)
}

pub(super) fn parse_fixes<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
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
    ))(input)
}

pub(super) fn parse_lookaround<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, LookaroundKind> {
    alt((
        value(LookaroundKind::Ahead, Token::LookAhead),
        value(LookaroundKind::Behind, Token::LookBehind),
        value(
            LookaroundKind::AheadNegative,
            pair(Token::Not, Token::LookAhead),
        ),
        value(
            LookaroundKind::BehindNegative,
            pair(Token::Not, Token::LookBehind),
        ),
    ))(input)
}

pub(super) fn parse_repetition<'i, 'b>(
    input: Input<'i, 'b>,
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
    )(input)
}

pub(super) fn parse_braced_repetition<'i, 'b>(
    input: Input<'i, 'b>,
) -> PResult<'i, 'b, RepetitionKind> {
    fn str_to_u32(s: &str) -> Result<u32, ParseErrorKind> {
        str::parse(s).map_err(|_| ParseErrorKind::Number(NumberError::TooLarge))
    }

    fn parse_u32<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, u32> {
        try_map(Token::Number, str_to_u32, nom::Err::Failure)(input)
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
    )(input)
}

pub(super) fn parse_atom<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    alt((
        parse_group,
        parse_string,
        parse_char_class,
        parse_boundary,
        map(parse_code_point, |c| {
            Rulex::CharClass(CharGroup::from_char(c).into())
        }),
        err(|| ParseErrorKind::Expected("expression")),
    ))(input)
}

pub(super) fn parse_group<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    fn parse_capture<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Capture<'i>> {
        map(pair(Token::Colon, opt(Token::Identifier)), |(_, name)| {
            Capture::new(name)
        })(input)
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
    )(input)
}

pub(super) fn parse_string<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(Token::String, |s| Rulex::Literal(strip_first_last(s)))(input)
}

pub(super) fn parse_char_class<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    #[derive(Clone, Copy)]
    enum StringOrChar<'i> {
        String(&'i str),
        Char(char),
    }

    impl StringOrChar<'_> {
        fn to_char(self) -> Result<char, ParseErrorKind> {
            Err(ParseErrorKind::CharString(match self {
                StringOrChar::Char(c) => return Ok(c),
                StringOrChar::String(s) => {
                    let s = strip_first_last(s);
                    let mut iter = s.chars();
                    match iter.next() {
                        Some(c) if matches!(iter.next(), None) => return Ok(c),
                        Some(_) => CharStringError::TooManyCodePoints,
                        _ => CharStringError::Empty,
                    }
                }
            }))
        }
    }

    fn parse_string_or_char<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, StringOrChar<'i>> {
        alt((
            map(Token::String, StringOrChar::String),
            map(parse_code_point, StringOrChar::Char),
            map(parse_special_char, StringOrChar::Char),
            err(|| ParseErrorKind::ExpectedCodePointOrChar),
        ))(input)
    }

    fn parse_chars_or_range<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, CharGroup<'i>> {
        // this is not clean code, but using the combinators results in worse error spans
        let span1 = input.index();
        let (input, first) = parse_string_or_char(input)?;

        if let Ok((input, _)) = Token::Dash.parse(input.clone()) {
            let span2 = input.index();
            let (input, last) = cut(parse_string_or_char)(input)?;

            let first = first
                .to_char()
                .map_err(|e| nom::Err::Failure(e.at(span1.clone())))?;
            let last = last
                .to_char()
                .map_err(|e| nom::Err::Failure(e.at(span2.clone())))?;

            let group = CharGroup::try_from_range(first, last).ok_or_else(|| {
                nom::Err::Failure(
                    ParseErrorKind::CharClass(CharClassError::DescendingRange(first, last))
                        .at(span1.start..span2.end),
                )
            })?;
            Ok((input, group))
        } else {
            let group = match first {
                StringOrChar::String(s) => CharGroup::from_chars(strip_first_last(s)),
                StringOrChar::Char(c) => CharGroup::from_char(c),
            };
            Ok((input, group))
        }
    }

    fn parse_char_group<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, CharGroup<'i>> {
        let span1 = input.index();

        let (input, ranges) = many1(alt((
            parse_chars_or_range,
            value(CharGroup::Dot, Token::Dot),
            try_map(
                Token::Identifier,
                |s| CharGroup::try_from_group_name(s).map_err(ParseErrorKind::CharClass),
                nom::Err::Failure,
            ),
            err(|| ParseErrorKind::CharClass(CharClassError::Invalid)),
        )))(input)?;

        let mut iter = ranges.into_iter();
        let mut class = iter.next().unwrap();

        for range in iter {
            class.add(range).map_err(|e| {
                nom::Err::Failure(ParseErrorKind::CharClass(e).at(span1.start..input.index().start))
            })?;
        }
        Ok((input, class))
    }

    map(
        pair(
            opt(Token::Not),
            delimited(
                Token::OpenBracket,
                cut(parse_char_group),
                cut(Token::CloseBracket),
            ),
        ),
        |(not, class)| {
            let mut class: CharClass<'_> = class.into();
            if not.is_some() {
                class.negate();
            }
            Rulex::CharClass(class)
        },
    )(input)
}

pub(super) fn parse_code_point<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, char> {
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
    )(input)
}

pub(super) fn parse_special_char<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, char> {
    try_map(
        Token::Identifier,
        |s| {
            Ok(match s {
                "n" => '\n',
                "r" => '\r',
                "t" => '\t',
                "a" => '\u{07}',
                "e" => '\u{1B}',
                "f" => '\u{0C}',
                _ => return Err(ParseErrorKind::Incomplete),
            })
        },
        nom::Err::Error,
    )(input)
}

pub(super) fn parse_boundary<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(
        alt((
            value(Boundary::Start, Token::BStart),
            value(Boundary::End, Token::BEnd),
            value(Boundary::Word, Token::BWord),
            value(Boundary::NotWord, pair(Token::Not, Token::BWord)),
        )),
        Rulex::Boundary,
    )(input)
}

fn strip_first_last(s: &str) -> &str {
    &s[1..s.len() - 1]
}

fn try_map<'i, 'b, O1, O2, P, M, EM>(
    mut parser: P,
    mut map: M,
    err_kind: EM,
) -> impl FnMut(Input<'i, 'b>) -> IResult<Input<'i, 'b>, O2, ParseError>
where
    P: Parser<Input<'i, 'b>, O1, ParseError>,
    M: FnMut(O1) -> Result<O2, ParseErrorKind>,
    EM: Copy + FnOnce(ParseError) -> nom::Err<ParseError>,
{
    move |input| {
        let span = input.index();
        let (rest, o1) = parser.parse(input)?;
        let o2 = map(o1).map_err(|e| err_kind(e.at(span)))?;
        Ok((rest, o2))
    }
}

fn err<'i, 'b, T>(
    mut error_fn: impl FnMut() -> ParseErrorKind,
) -> impl FnMut(Input<'i, 'b>) -> IResult<Input<'i, 'b>, T, ParseError> {
    move |input| Err(nom::Err::Error(error_fn().at(input.index())))
}
