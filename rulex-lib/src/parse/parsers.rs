use std::{
    borrow::{Borrow, Cow},
    collections::HashSet,
    str::FromStr,
};

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
    group::{Capture, Group},
    literal::Literal,
    lookaround::{Lookaround, LookaroundKind},
    range::Range,
    reference::{Reference, ReferenceTarget},
    repetition::{Quantifier, Repetition, RepetitionError, RepetitionKind},
    rule::Rule,
    span::Span,
    stmt::{BooleanSetting, Let, Stmt, StmtExpr},
    var::Variable,
};

use super::{Input, Token};

pub(super) type PResult<'i, 'b, T> = IResult<Input<'i, 'b>, T, ParseError>;

pub(crate) fn parse(source: &str) -> Result<Rule<'_>, ParseError> {
    let tokens = super::tokenize::tokenize(source);
    let input = Input::from(source, &tokens)?;

    let (rest, rules) = parse_modified(input)?;
    if rest.is_empty() {
        Ok(rules)
    } else {
        Err(ParseErrorKind::LeftoverTokens.at(rest.span()))
    }
}

pub(super) fn parse_modified<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
    enum ModifierKind {
        Enable,
        Disable,
    }

    try_map2(
        pair(
            many0(alt((
                map(
                    tuple((
                        alt((
                            map("enable", |(_, span)| (ModifierKind::Enable, span)),
                            map("disable", |(_, span)| (ModifierKind::Disable, span)),
                        )),
                        value(BooleanSetting::Lazy, "lazy"),
                        Token::Semicolon,
                    )),
                    |((kind, span_start), value, (_, span_end))| {
                        let stmt = match kind {
                            ModifierKind::Enable => Stmt::Enable(value),
                            ModifierKind::Disable => Stmt::Disable(value),
                        };
                        (stmt, span_start.join(span_end))
                    },
                ),
                map(
                    tuple((
                        "let",
                        cut(Token::Identifier),
                        cut(Token::Equals),
                        cut(parse_or),
                        cut(Token::Semicolon),
                    )),
                    |((_, span_start), (name, name_span), _, rule, (_, span_end))| {
                        (Stmt::Let(Let::new(name, rule, name_span)), span_start.join(span_end))
                    },
                ),
            ))),
            parse_or,
        ),
        |(stmts, mut rule): (Vec<(Stmt, Span)>, _)| {
            if stmts.len() > 1 {
                let mut set = HashSet::new();
                for (stmt, _) in &stmts {
                    if let Stmt::Let(l) = stmt {
                        if set.contains(l.name()) {
                            return Err(ParseErrorKind::LetBindingExists.at(l.name_span));
                        }
                        set.insert(l.name());
                    }
                }
            }

            let span_end = rule.span();
            for (stmt, span) in stmts.into_iter().rev() {
                rule = Rule::StmtExpr(Box::new(StmtExpr::new(stmt, rule, span.join(span_end))));
            }
            Ok(rule)
        },
        nom::Err::Failure,
    )(input)
}

pub(super) fn parse_or<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
    map(separated_list0(Token::Pipe, parse_sequence), |mut rules| {
        if rules.len() == 1 {
            rules.pop().unwrap()
        } else {
            Alternation::new_rulex(rules)
        }
    })(input)
}

pub(super) fn parse_sequence<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
    map(many1(parse_fixes), |mut rules| {
        if rules.len() == 1 {
            rules.pop().unwrap()
        } else {
            let start = rules.first().map(|f| f.span()).unwrap_or_default();
            let end = rules.last().map(|f| f.span()).unwrap_or_default();

            Rule::Group(Group::new(rules, None, start.join(end)))
        }
    })(input)
}

pub(super) fn parse_fixes<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
    alt((
        try_map(
            pair(Token::Not, opt(parse_fixes)),
            |(_, rule)| {
                if let Some(mut rule) = rule {
                    rule.negate()?;
                    Ok(rule)
                } else {
                    Err(ParseErrorKind::Expected("expression"))
                }
            },
            nom::Err::Failure,
        ),
        map(pair(parse_lookaround, parse_modified), |((kind, span), rule)| {
            let span = span.join(rule.span());
            Rule::Lookaround(Box::new(Lookaround::new(rule, kind, span)))
        }),
        try_map2(
            pair(parse_atom, many0(parse_repetition)),
            |(mut rule, repetitions)| {
                let mut prev_syntax = RepSyntax::ExplicitQuantifier;
                for (kind, quantifier, span, syntax) in repetitions {
                    if matches!(
                        (&prev_syntax, &syntax),
                        (RepSyntax::Other | RepSyntax::QuestionMark, RepSyntax::QuestionMark)
                    ) {
                        return Err(ParseErrorKind::Repetition(
                            RepetitionError::QuestionMarkAfterRepetition,
                        )
                        .at(span));
                    }
                    prev_syntax = syntax;

                    let span = rule.span().join(span);
                    rule =
                        Rule::Repetition(Box::new(Repetition::new(rule, kind, quantifier, span)));
                }
                Ok(rule)
            },
            nom::Err::Failure,
        ),
    ))(input)
}

pub(super) fn parse_lookaround<'i, 'b>(
    input: Input<'i, 'b>,
) -> PResult<'i, 'b, (LookaroundKind, Span)> {
    alt((
        map(Token::LookAhead, |(_, span)| (LookaroundKind::Ahead, span)),
        map(Token::LookBehind, |(_, span)| (LookaroundKind::Behind, span)),
    ))(input)
}

pub(super) enum RepSyntax {
    ExplicitQuantifier,
    QuestionMark,
    Other,
}

pub(super) fn parse_repetition<'i, 'b>(
    input: Input<'i, 'b>,
) -> PResult<'i, 'b, (RepetitionKind, Quantifier, Span, RepSyntax)> {
    map(
        pair(
            alt((
                map(Token::QuestionMark, |(_, span)| {
                    (RepetitionKind::zero_one(), span, RepSyntax::QuestionMark)
                }),
                map(Token::Star, |(_, span)| (RepetitionKind::zero_inf(), span, RepSyntax::Other)),
                map(Token::Plus, |(_, span)| (RepetitionKind::one_inf(), span, RepSyntax::Other)),
                parse_braced_repetition,
            )),
            map(
                opt(alt((
                    map("greedy", |(_, span)| (Quantifier::Greedy, span)),
                    map("lazy", |(_, span)| (Quantifier::Lazy, span)),
                ))),
                |a| match a {
                    Some((q, span)) => (q, span, RepSyntax::ExplicitQuantifier),
                    None => (Quantifier::Default, Span::default(), RepSyntax::Other),
                },
            ),
        ),
        |((kind, span1, rs1), (quantifier, span2, rs2))| {
            (
                kind,
                quantifier,
                span1.join(span2),
                match (rs1, rs2) {
                    (_, RepSyntax::ExplicitQuantifier) => RepSyntax::ExplicitQuantifier,
                    (RepSyntax::QuestionMark, _) => RepSyntax::QuestionMark,
                    _ => RepSyntax::Other,
                },
            )
        },
    )(input)
}

pub(super) fn parse_braced_repetition<'i, 'b>(
    input: Input<'i, 'b>,
) -> PResult<'i, 'b, (RepetitionKind, Span, RepSyntax)> {
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
        |((_, start), rep, (_, end))| (rep, start.join(end), RepSyntax::Other),
    )(input)
}

pub(super) fn parse_atom<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
    alt((
        parse_group,
        parse_string,
        parse_char_class,
        parse_boundary,
        parse_reference,
        map(parse_code_point, |(c, span)| {
            Rule::CharClass(CharClass::new(CharGroup::from_char(c), span))
        }),
        parse_range,
        parse_variable,
        try_map(Token::Dot, |_| Err(ParseErrorKind::Dot), nom::Err::Failure),
        err(|| ParseErrorKind::Expected("expression")),
    ))(input)
}

pub(super) fn parse_group<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
    fn parse_capture<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, (Capture<'i>, Span)> {
        map(pair(Token::Colon, opt(Token::Identifier)), |((_, span1), name)| {
            (Capture::new(name.map(|(s, _)| s)), span1)
        })(input)
    }

    map(
        pair(opt(parse_capture), tuple((Token::OpenParen, parse_modified, cut(Token::CloseParen)))),
        |(capture, (_, rule, (_, close_paren)))| match (capture, rule) {
            (None, rule) => rule,
            (Some((capture, c_span)), Rule::Group(mut g)) if !g.is_capturing() => {
                g.set_capture(capture);
                g.span = c_span.join(g.span);
                Rule::Group(g)
            }
            (Some((capture, c_span)), rule) => {
                Rule::Group(Group::new(vec![rule], Some(capture), c_span.join(close_paren)))
            }
        },
    )(input)
}

pub(super) fn parse_string<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
    try_map(
        Token::String,
        |(s, span)| Ok(Rule::Literal(Literal::new(parse_quoted_text(s)?, span))),
        nom::Err::Failure,
    )(input)
}

pub(super) fn parse_char_class<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
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
                    let s = parse_quoted_text(s)?;
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
        // this is not clean code, but using the combinators results in worse error
        // spans
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
                StringOrChar::String(s) => CharGroup::from_chars(
                    parse_quoted_text(s).map_err(|k| nom::Err::Failure(k.at(span1)))?.borrow(),
                ),
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
        tuple((Token::OpenBracket, cut(parse_char_group), cut(Token::CloseBracket))),
        |((_, start), inner, (_, end))| Rule::CharClass(CharClass::new(inner, start.join(end))),
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

pub(super) fn parse_range<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
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
            Rule::Range(range)
        },
    )(input)
}

pub(super) fn parse_variable<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
    map(Token::Identifier, |(name, span)| Rule::Variable(Variable::new(name, span)))(input)
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

pub(super) fn parse_boundary<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
    map(
        alt((
            map(Token::BStart, |(_, span)| Boundary::new(BoundaryKind::Start, span)),
            map(Token::BEnd, |(_, span)| Boundary::new(BoundaryKind::End, span)),
            map(Token::BWord, |(_, span)| Boundary::new(BoundaryKind::Word, span)),
            map(pair(Token::Not, Token::BWord), |((_, span1), (_, span2))| {
                Boundary::new(BoundaryKind::NotWord, span1.join(span2))
            }),
        )),
        Rule::Boundary,
    )(input)
}

pub(super) fn parse_reference<'i, 'b>(input: Input<'i, 'b>) -> PResult<'i, 'b, Rule<'i>> {
    preceded(
        Token::Backref,
        alt((
            try_map(
                Token::Number,
                |(s, span)| {
                    let target = ReferenceTarget::Number(from_str(s)?);
                    Ok(Rule::Reference(Reference::new(target, span)))
                },
                nom::Err::Failure,
            ),
            map(Token::Identifier, |(s, span)| {
                let target = ReferenceTarget::Named(s);
                Rule::Reference(Reference::new(target, span))
            }),
            try_map(
                pair(alt((Token::Plus, Token::Dash)), Token::Number),
                |((sign, span1), (s, span2))| {
                    let num = if sign == "-" { from_str(&format!("-{s}")) } else { from_str(s) }?;
                    let target = ReferenceTarget::Relative(num);
                    Ok(Rule::Reference(Reference::new(target, span1.join(span2))))
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

fn parse_quoted_text(input: &str) -> Result<Cow<'_, str>, ParseErrorKind> {
    Ok(match input.as_bytes()[0] {
        b'"' => {
            let mut s = strip_first_last(input);
            let mut buf = String::new();

            loop {
                let mut chars = s.chars();
                match chars.next() {
                    Some('\\') => match chars.next() {
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
                    },
                    Some(c) => buf.push(c),
                    None => break,
                }
                s = &s[1..];
            }
            Cow::Owned(buf)
        }
        _ => Cow::Borrowed(strip_first_last(input)),
    })
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

fn try_map2<'i, 'b, O1, O2, P, M, EM>(
    mut parser: P,
    mut map: M,
    err_kind: EM,
) -> impl FnMut(Input<'i, 'b>) -> IResult<Input<'i, 'b>, O2, ParseError>
where
    P: Parser<Input<'i, 'b>, O1, ParseError>,
    M: FnMut(O1) -> Result<O2, ParseError>,
    EM: Copy + FnOnce(ParseError) -> nom::Err<ParseError>,
{
    move |input| {
        let (rest, o1) = parser.parse(input)?;
        let o2 = map(o1).map_err(err_kind)?;
        Ok((rest, o2))
    }
}

fn err<'i, 'b, T>(
    mut error_fn: impl FnMut() -> ParseErrorKind,
) -> impl FnMut(Input<'i, 'b>) -> IResult<Input<'i, 'b>, T, ParseError> {
    move |input| Err(nom::Err::Error(error_fn().at(input.span())))
}
