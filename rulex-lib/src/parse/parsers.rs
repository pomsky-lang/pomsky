use std::str::FromStr;

use nom::{
    branch::alt,
    combinator::{cut, map, opt, value},
    multi::{many0, many1, separated_list0},
    sequence::{pair, preceded, separated_pair, tuple},
    IResult, Parser,
};

use crate::{
    alternation::Alternation,
    boundary::{Boundary, BoundaryKind},
    char_class::{CharClass, CharGroup},
    error::{
        CharClassError, CharStringError, CodePointError, NumberError, ParseError, ParseErrorKind,
    },
    grapheme::Grapheme,
    group::{Capture, Group},
    literal::Literal,
    lookaround::{Lookaround, LookaroundKind},
    modified::{BooleanSetting, Modified, Modifier},
    range::Range,
    reference::{Reference, ReferenceTarget},
    repetition::{Quantifier, Repetition, RepetitionKind},
    span::Span,
    Rulex,
};

use super::{Input, Token};

pub(super) type PResult<'i, 'b, T> = IResult<Input<'i, 'b>, T, ParseError>;

pub(crate) fn parse(source: &str) -> Result<Rulex<'_>, ParseError> {
    let tokens = super::tokenize::tokenize(source);
    let input = Input::from(source, &tokens)?;

    let (rest, rules) = parse_modified(input)?;
    if rest.is_empty() {
        Ok(rules)
    } else {
        Err(ParseErrorKind::LeftoverTokens.at(rest.span()))
    }
}

pub(super) fn parse_modified<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    enum ModifierKind {
        Enable,
        Disable,
    }

    map(
        pair(
            many0(tuple((
                alt((
                    map("enable", |(_, span)| (ModifierKind::Enable, span)),
                    map("disable", |(_, span)| (ModifierKind::Disable, span)),
                )),
                value(BooleanSetting::Lazy, "lazy"),
                Token::Semicolon,
            ))),
            parse_or,
        ),
        |(modifiers, mut rule)| {
            let span2 = rule.span();
            for ((kind, span1), value, _) in modifiers.into_iter().rev() {
                let modifier = match kind {
                    ModifierKind::Enable => Modifier::Enable(value),
                    ModifierKind::Disable => Modifier::Disable(value),
                };
                rule = Rulex::Modified(Box::new(Modified::new(modifier, rule, span1.join(span2))));
            }
            rule
        },
    )(input)
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
            let start = rules.first().map(|f| f.span()).unwrap_or_default();
            let end = rules.last().map(|f| f.span()).unwrap_or_default();

            Rulex::Group(Group::new(rules, None, start.join(end)))
        }
    })(input)
}

pub(super) fn parse_fixes<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    alt((
        map(pair(parse_lookaround, parse_modified), |((kind, span), rule)| {
            let span = span.join(rule.span());
            Rulex::Lookaround(Box::new(Lookaround::new(rule, kind, span)))
        }),
        map(pair(parse_atom, many0(parse_repetition)), |(mut rule, repetitions)| {
            for (kind, quantifier, span) in repetitions {
                let span = rule.span().join(span);
                rule = Rulex::Repetition(Box::new(Repetition::new(rule, kind, quantifier, span)));
            }
            rule
        }),
    ))(input)
}

pub(super) fn parse_lookaround<'i, 'b>(
    input: Input<'i, 'b>,
) -> PResult<'i, 'b, (LookaroundKind, Span)> {
    alt((
        map(Token::LookAhead, |(_, span)| (LookaroundKind::Ahead, span)),
        map(Token::LookBehind, |(_, span)| (LookaroundKind::Behind, span)),
        map(pair(Token::Not, Token::LookAhead), |((_, span1), (_, span2))| {
            (LookaroundKind::AheadNegative, span1.join(span2))
        }),
        map(pair(Token::Not, Token::LookBehind), |((_, span1), (_, span2))| {
            (LookaroundKind::BehindNegative, span1.join(span2))
        }),
    ))(input)
}

pub(super) fn parse_repetition<'i, 'b>(
    input: Input<'i, 'b>,
) -> PResult<'i, 'b, (RepetitionKind, Quantifier, Span)> {
    map(
        pair(
            alt((
                map(Token::QuestionMark, |(_, span)| (RepetitionKind::zero_one(), span)),
                map(Token::Star, |(_, span)| (RepetitionKind::zero_inf(), span)),
                map(Token::Plus, |(_, span)| (RepetitionKind::one_inf(), span)),
                parse_braced_repetition,
            )),
            map(
                opt(alt((
                    map("greedy", |(_, span)| (Quantifier::Greedy, span)),
                    map("lazy", |(_, span)| (Quantifier::Lazy, span)),
                ))),
                |a| match a {
                    Some((q, span)) => (q, span),
                    None => (Quantifier::Default, Span::default()),
                },
            ),
        ),
        |((kind, span1), (quantifier, span2))| (kind, quantifier, span1.join(span2)),
    )(input)
}

pub(super) fn parse_braced_repetition<'i, 'b>(
    input: Input<'i, 'b>,
) -> PResult<'i, 'b, (RepetitionKind, Span)> {
    fn parse_u32<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, u32> {
        try_map(Token::Number, |(s, _)| from_str(s), nom::Err::Failure)(input)
    }

    map(
        tuple((
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
        )),
        |((_, start), rep, (_, end))| (rep, start.join(end)),
    )(input)
}

pub(super) fn parse_atom<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    alt((
        parse_group,
        parse_string,
        parse_char_class,
        parse_grapheme,
        parse_boundary,
        parse_reference,
        map(parse_code_point, |(c, span)| {
            Rulex::CharClass(CharClass::new(CharGroup::from_char(c), span))
        }),
        parse_range,
        try_map(Token::Dot, |_| Err(ParseErrorKind::Dot), nom::Err::Failure),
        err(|| ParseErrorKind::Expected("expression")),
    ))(input)
}

pub(super) fn parse_group<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    fn parse_capture<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, (Capture<'i>, Span)> {
        map(pair(Token::Colon, opt(Token::Identifier)), |((_, span1), name)| {
            (Capture::new(name.map(|(s, _)| s)), span1)
        })(input)
    }

    map(
        pair(opt(parse_capture), tuple((Token::OpenParen, parse_modified, cut(Token::CloseParen)))),
        |(capture, (_, rule, (_, close_paren)))| match (capture, rule) {
            (None, rule) => rule,
            (Some((capture, c_span)), Rulex::Group(mut g)) if !g.is_capturing() => {
                g.set_capture(capture);
                g.span = c_span.join(g.span);
                Rulex::Group(g)
            }
            (Some((capture, c_span)), rule) => {
                Rulex::Group(Group::new(vec![rule], Some(capture), c_span.join(close_paren)))
            }
        },
    )(input)
}

pub(super) fn parse_string<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(Token::String, |(s, span)| Rulex::Literal(Literal::new(strip_first_last(s), span)))(input)
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
            map(Token::String, |(s, _)| StringOrChar::String(s)),
            map(parse_code_point, |(c, _)| StringOrChar::Char(c)),
            map(parse_special_char, StringOrChar::Char),
            err(|| ParseErrorKind::ExpectedCodePointOrChar),
        ))(input)
    }

    fn parse_chars_or_range<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, CharGroup> {
        // this is not clean code, but using the combinators results in worse error spans
        let span1 = input.span();
        let (input, first) = parse_string_or_char(input)?;

        if let Ok((input, _)) = Token::Dash.parse(input.clone()) {
            let span2 = input.span();
            let (input, last) = cut(parse_string_or_char)(input)?;

            let first = first.to_char().map_err(|e| nom::Err::Failure(e.at(span1)))?;
            let last = last.to_char().map_err(|e| nom::Err::Failure(e.at(span2)))?;

            let group = CharGroup::try_from_range(first, last).ok_or_else(|| {
                nom::Err::Failure(
                    ParseErrorKind::CharClass(CharClassError::DescendingRange(first, last))
                        .at(span1.join(span2)),
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

    fn parse_char_group<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, CharGroup> {
        let span1 = input.span();

        let (input, ranges) = many1(alt((
            parse_chars_or_range,
            value(CharGroup::Dot, Token::Dot),
            try_map(
                pair(opt(Token::Not), Token::Identifier),
                |(not, (s, _))| {
                    // FIXME: When this fails on a negative item, the span of the exclamation mark
                    // is used instead of the identifier's span
                    CharGroup::try_from_group_name(s, not.is_some())
                        .map_err(ParseErrorKind::CharClass)
                },
                nom::Err::Failure,
            ),
            err(|| ParseErrorKind::CharClass(CharClassError::Invalid)),
        )))(input)?;

        let mut iter = ranges.into_iter();
        let mut class = iter.next().unwrap();

        for range in iter {
            class.add(range).map_err(|e| {
                nom::Err::Failure(ParseErrorKind::CharClass(e).at(span1.join(input.span().start())))
            })?;
        }
        Ok((input, class))
    }

    map(
        pair(
            opt(Token::Not),
            tuple((Token::OpenBracket, cut(parse_char_group), cut(Token::CloseBracket))),
        ),
        |(not, ((_, start), inner, (_, end)))| {
            let mut class: CharClass = CharClass::new(inner, start.join(end));
            if not.is_some() {
                class.negate();
            }
            Rulex::CharClass(class)
        },
    )(input)
}

pub(super) fn parse_code_point<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, (char, Span)> {
    alt((
        try_map(
            Token::CodePoint,
            |(s, span)| {
                let hex = &s[2..];
                if hex.len() > 6 {
                    Err(ParseErrorKind::CodePoint(CodePointError::Invalid))
                } else {
                    u32::from_str_radix(hex, 16)
                        .ok()
                        .and_then(|n| char::try_from(n).ok())
                        .map(|c| (c, span))
                        .ok_or(ParseErrorKind::CodePoint(CodePointError::Invalid))
                }
            },
            nom::Err::Failure,
        ),
        try_map(
            Token::Identifier,
            |(str, span)| {
                if let Some(rest) = str.strip_prefix('U') {
                    if let Ok(n) = u32::from_str_radix(rest, 16) {
                        if let Ok(c) = char::try_from(n) {
                            return Ok((c, span));
                        } else {
                            return Err(ParseErrorKind::CodePoint(CodePointError::Invalid));
                        }
                    }
                }
                Err(ParseErrorKind::ExpectedToken(Token::CodePoint))
            },
            nom::Err::Error,
        ),
    ))(input)
}

pub(super) fn parse_range<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    fn parse_base<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, (u8, Span)> {
        preceded(
            "base",
            try_map(
                cut(Token::Number),
                |(s, span)| {
                    let n = s.parse().map_err(NumberError::from)?;
                    if n > 36 {
                        Err(ParseErrorKind::Number(NumberError::TooLarge))
                    } else if n < 2 {
                        Err(ParseErrorKind::Number(NumberError::TooSmall))
                    } else {
                        Ok((n, span))
                    }
                },
                nom::Err::Failure,
            ),
        )(input)
    }

    fn parse_number(src: &str, radix: u8) -> Result<Vec<u8>, NumberError> {
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

    map(
        pair(
            "range",
            try_map(
                pair(
                    cut(separated_pair(Token::String, Token::Dash, Token::String)),
                    opt(parse_base),
                ),
                |(((start, span1), (end, span2)), base)| {
                    let (radix, span) = match base {
                        Some((base, span3)) => (base, span1.join(span3)),
                        None => (10, span1.join(span2)),
                    };
                    let start = parse_number(strip_first_last(start), radix)?;
                    let end = parse_number(strip_first_last(end), radix)?;

                    if start.len() > end.len() || (start.len() == end.len() && start > end) {
                        return Err(ParseErrorKind::RangeIsNotIncreasing);
                    }

                    Ok(Range::new(start, end, radix, span))
                },
                nom::Err::Failure,
            ),
        ),
        |((_, span), mut range)| {
            range.span = range.span.join(span);
            Rulex::Range(range)
        },
    )(input)
}

pub(super) fn parse_special_char<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, char> {
    try_map(
        Token::Identifier,
        |(s, _)| {
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

pub(super) fn parse_grapheme<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(alt(("Grapheme", "X")), |(_, span)| Rulex::Grapheme(Grapheme { span }))(input)
}

pub(super) fn parse_boundary<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    map(
        alt((
            map(Token::BStart, |(_, span)| Boundary::new(BoundaryKind::Start, span)),
            map(Token::BEnd, |(_, span)| Boundary::new(BoundaryKind::End, span)),
            map(Token::BWord, |(_, span)| Boundary::new(BoundaryKind::Word, span)),
            map(pair(Token::Not, Token::BWord), |((_, span1), (_, span2))| {
                Boundary::new(BoundaryKind::NotWord, span1.join(span2))
            }),
        )),
        Rulex::Boundary,
    )(input)
}

pub(super) fn parse_reference<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rulex<'i>> {
    preceded(
        Token::Backref,
        alt((
            try_map(
                Token::Number,
                |(s, span)| {
                    let target = ReferenceTarget::Number(from_str(s)?);
                    Ok(Rulex::Reference(Reference::new(target, span)))
                },
                nom::Err::Failure,
            ),
            map(Token::Identifier, |(s, span)| {
                let target = ReferenceTarget::Named(s);
                Rulex::Reference(Reference::new(target, span))
            }),
            try_map(
                pair(alt((Token::Plus, Token::Dash)), Token::Number),
                |((sign, span1), (s, span2))| {
                    let num = if sign == "-" { from_str(&format!("-{s}")) } else { from_str(s) }?;
                    let target = ReferenceTarget::Relative(num);
                    Ok(Rulex::Reference(Reference::new(target, span1.join(span2))))
                },
                nom::Err::Failure,
            ),
            err(|| ParseErrorKind::Expected("number or group name")),
        )),
    )(input)
}

fn from_str<T: FromStr>(s: &str) -> Result<T, ParseErrorKind> {
    str::parse(s).map_err(|_| ParseErrorKind::Number(NumberError::TooLarge))
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
        let span = input.span();
        let (rest, o1) = parser.parse(input)?;
        let o2 = map(o1).map_err(|e| err_kind(e.at(span)))?;
        Ok((rest, o2))
    }
}

fn err<'i, 'b, T>(
    mut error_fn: impl FnMut() -> ParseErrorKind,
) -> impl FnMut(Input<'i, 'b>) -> IResult<Input<'i, 'b>, T, ParseError> {
    move |input| Err(nom::Err::Error(error_fn().at(input.span())))
}
