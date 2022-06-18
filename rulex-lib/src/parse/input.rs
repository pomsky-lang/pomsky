use std::iter::Enumerate;

use nom::{InputIter, InputLength, InputTake};

use crate::{
    error::{ParseError, ParseErrorKind},
    span::Span,
};

use super::token::Token;

#[derive(Clone)]
pub(crate) struct Input<'i, 'b> {
    source: &'i str,
    tokens: &'b [(Token, Span)],
    recursion: u16,
}

impl<'i, 'b> Input<'i, 'b> {
    pub(super) fn from(
        source: &'i str,
        tokens: &'b [(Token, Span)],
        recursion: u16,
    ) -> Result<Self, ParseError> {
        let error = tokens.iter().find_map(|&(t, span)| match t {
            Token::Error => Some((span, None)),
            Token::ErrorMsg(m) => Some((span, Some(m))),
            _ => None,
        });
        if let Some((span, msg)) = error {
            return match msg {
                Some(msg) => Err(ParseErrorKind::LexErrorWithMessage(msg).at(span)),
                None => Err(ParseErrorKind::UnknownToken.at(span)),
            };
        }

        Ok(Input { source, tokens, recursion })
    }

    pub(super) fn recursion_start(&mut self) -> Result<(), ParseError> {
        self.recursion = self
            .recursion
            .checked_sub(1)
            .ok_or_else(|| ParseErrorKind::RecursionLimit.at(self.span()))?;
        Ok(())
    }

    pub(super) fn recursion_end(&mut self) {
        self.recursion += 1;
    }

    pub(super) fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub(crate) fn span(&self) -> Span {
        self.tokens
            .first()
            .map(|&(_, span)| span)
            .unwrap_or_else(|| (self.source.len()..self.source.len()).into())
    }

    pub(super) fn peek(&self) -> Option<(Token, &'i str)> {
        self.iter_elements().next()
    }
}

impl<'i, 'b> PartialEq for Input<'i, 'b> {
    fn eq(&self, other: &Self) -> bool {
        Iterator::eq(self.iter_elements(), other.iter_elements())
    }
}

#[cfg(feature = "dbg")]
impl<'i, 'b> core::fmt::Debug for Input<'i, 'b> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct FmtHelper<'a>(Token, &'a str);

        impl core::fmt::Debug for FmtHelper<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:?} {:?}", self.0, self.1)
            }
        }

        let v = self
            .tokens
            .iter()
            .map(|&(t, span)| FmtHelper(t, &self.source[span.range()]))
            .collect::<Vec<_>>();

        v.fmt(f)
    }
}

impl<'i, 'b> Iterator for Input<'i, 'b> {
    type Item = (Token, &'i str);

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.split_first() {
            Some((&(token, span), rest)) => {
                self.tokens = rest;
                Some((token, &self.source[span.range()]))
            }
            None => None,
        }
    }
}

impl<'i, 'b> InputIter for Input<'i, 'b> {
    type Item = (Token, &'i str);

    type Iter = Enumerate<Self>;

    type IterElem = Self;

    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        Input { ..*self }
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        let mut iter = self.iter_elements();
        Iterator::position(&mut iter, predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if count <= self.tokens.len() {
            Ok(count)
        } else {
            Err(nom::Needed::Size((count - self.tokens.len()).try_into().unwrap()))
        }
    }
}

impl<'i, 'b> InputLength for Input<'i, 'b> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

impl<'i, 'b> InputTake for Input<'i, 'b> {
    fn take(&self, count: usize) -> Self {
        let tokens = &self.tokens[..count];

        Input { tokens, ..*self }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (left, right) = self.tokens.split_at(count);

        (Input { tokens: left, ..*self }, Input { tokens: right, ..*self })
    }
}
