use std::str::FromStr;

use crate::{
    Span,
    diagnose::{
        LexErrorMsg, NumberError, ParseDiagnostic, ParseError, ParseErrorKind as PEK, ParseWarning,
    },
    exprs::*,
    lexer::{Token, tokenize},
};

/// Parses a source string as a pomsky expression.
///
/// The `recursion` argument determines how much nesting is allowed in the
/// expression. Note that **pomsky will overflow the stack** when parsing an
/// expression with too much nesting, so the `recursion` argument should be low
/// enough to prevent that. The recommended default is 256.
pub fn parse(source: &str, recursion: u32) -> (Option<Rule>, Vec<ParseDiagnostic>) {
    if source.len() > u32::MAX as usize {
        let error = PEK::LexErrorWithMessage(LexErrorMsg::FileTooBig);
        return (None, vec![error.at(Span::empty()).into()]);
    }

    let tokens = tokenize(source);

    let mut errors = Vec::new();
    for &(t, span) in &tokens {
        match t {
            Token::Error => errors.push((span, None)),
            Token::ErrorMsg(m) => errors.push((span, Some(m))),
            _ => {}
        }
    }

    if !errors.is_empty() {
        let errors = errors
            .into_iter()
            .map(|(span, msg)| {
                msg.map_or(PEK::UnknownToken, PEK::LexErrorWithMessage).at(span).into()
            })
            .collect::<Vec<_>>();

        return (None, errors);
    }

    let mut parser = Parser {
        source,
        tokens: tokens.into_boxed_slice(),
        offset: 0,
        warnings: Vec::new(),
        recursion,
        is_lazy: false,
        is_unicode_aware: true,
    };

    let rule = match parser.parse_modified() {
        Ok(rule) => rule,
        Err(err) => {
            let mut diagnostics = vec![err.into()];
            diagnostics.extend(parser.warnings);
            return (None, diagnostics);
        }
    };
    if parser.is_empty() {
        (Some(rule), parser.warnings)
    } else {
        let mut diagnostics = vec![PEK::LeftoverTokens.at(parser.span()).into()];
        diagnostics.extend(parser.warnings);
        (None, diagnostics)
    }
}

type PResult<T> = Result<T, ParseError>;

pub(super) struct Parser<'i> {
    source: &'i str,
    tokens: Box<[(Token, Span)]>,
    offset: usize,
    warnings: Vec<ParseDiagnostic>,
    recursion: u32,
    pub(super) is_lazy: bool,
    pub(super) is_unicode_aware: bool,
}

// Utilities
impl<'i> Parser<'i> {
    pub(super) fn is_empty(&self) -> bool {
        self.tokens.len() == self.offset
    }

    pub(super) fn source_at(&self, span: Span) -> &'i str {
        &self.source[span.range_unchecked()]
    }

    pub(super) fn peek(&self) -> Option<(Token, &'i str)> {
        self.tokens.get(self.offset).map(|&(t, span)| (t, self.source_at(span)))
    }

    pub(super) fn peek_pair(&self) -> Option<(Token, Span)> {
        self.tokens.get(self.offset).copied()
    }

    /// Returns the span of the next token
    pub(super) fn span(&self) -> Span {
        self.tokens
            .get(self.offset)
            .map_or_else(|| Span::new(self.source.len(), self.source.len()), |&(_, s)| s)
    }

    /// Returns the span of the previously consumed token
    pub(super) fn last_span(&self) -> Span {
        self.tokens[self.offset - 1].1
    }

    pub(super) fn advance(&mut self) {
        self.offset += 1;
    }

    pub(super) fn recursion_start(&mut self) -> PResult<()> {
        self.recursion =
            self.recursion.checked_sub(1).ok_or_else(|| PEK::RecursionLimit.at(self.span()))?;
        Ok(())
    }

    pub(super) fn recursion_end(&mut self) {
        self.recursion += 1;
    }

    pub(super) fn add_warning(&mut self, warning: ParseWarning) {
        self.warnings.push(warning.into());
    }

    pub(super) fn is(&mut self, token: Token) -> bool {
        matches!(self.peek_pair(), Some((t, _)) if t == token)
    }

    pub(super) fn consume(&mut self, token: Token) -> bool {
        match self.peek_pair() {
            Some((t, _)) if t == token => {
                self.offset += 1;
                true
            }
            _ => false,
        }
    }

    pub(super) fn consume_as(&mut self, token: Token) -> Option<&'i str> {
        match self.peek_pair() {
            Some((t, span)) if t == token => {
                self.offset += 1;
                Some(self.source_at(span))
            }
            _ => None,
        }
    }

    pub(super) fn consume_reserved(&mut self, reserved: &str) -> bool {
        match self.peek_pair() {
            Some((Token::ReservedName, s)) if self.source_at(s) == reserved => {
                self.offset += 1;
                true
            }
            _ => false,
        }
    }

    pub(super) fn consume_contextual_keyword(&mut self, keyword: &str) -> bool {
        match self.peek_pair() {
            Some((Token::Identifier, s)) if self.source_at(s) == keyword => {
                self.offset += 1;
                true
            }
            _ => false,
        }
    }

    pub(super) fn consume_number<T: FromStr + PartialOrd>(&mut self, max: T) -> PResult<Option<T>> {
        match self.peek_pair() {
            Some((Token::Number, span)) => {
                let n = str::parse(self.source_at(span))
                    .ok()
                    .and_then(|n| if n > max { None } else { Some(n) })
                    .ok_or_else(|| PEK::Number(NumberError::TooLarge).at(span))?;
                self.offset += 1;
                Ok(Some(n))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn expect(&mut self, token: Token) -> PResult<()> {
        match self.peek_pair() {
            Some((t, _)) if t == token => {
                self.offset += 1;
                Ok(())
            }
            _ => Err(PEK::ExpectedToken(token).at(self.span())),
        }
    }

    pub(super) fn expect_as(&mut self, token: Token) -> PResult<&'i str> {
        match self.peek_pair() {
            Some((t, span)) if t == token => {
                self.offset += 1;
                Ok(self.source_at(span))
            }
            _ => Err(PEK::ExpectedToken(token).at(self.span())),
        }
    }

    pub(super) fn expect_number<T: FromStr>(&mut self) -> PResult<T> {
        match self.peek_pair() {
            Some((Token::Number, span)) => {
                let n = str::parse(self.source_at(span))
                    .map_err(|_| PEK::Number(NumberError::TooLarge).at(span))?;
                self.offset += 1;
                Ok(n)
            }
            _ => Err(PEK::ExpectedToken(Token::Number).at(self.span())),
        }
    }
}
