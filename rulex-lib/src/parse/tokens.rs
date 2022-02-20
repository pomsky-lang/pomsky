use std::iter::Enumerate;

use logos::Logos;
use nom::{InputIter, InputLength, InputTake};

use super::token::Token;

#[derive(Clone)]
pub(crate) struct Tokens<'i, 'b> {
    source: &'i str,
    tokens: &'b [(Token, (usize, usize))],
}

impl<'i, 'b> Tokens<'i, 'b> {
    pub(super) fn tokenize(source: &'i str, buf: &'b mut Vec<(Token, (usize, usize))>) -> Self {
        assert!(buf.is_empty());

        let lex = Token::lexer(source);
        buf.extend(lex.spanned().map(|(t, r)| (t, (r.start, r.end))));
        let tokens = &**buf;
        Tokens { source, tokens }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub(super) fn len(&self) -> usize {
        self.tokens.len()
    }

    pub(super) fn peek(&self) -> Option<(Token, &'i str)> {
        self.iter_elements().next()
    }

    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Tokens {
            source: "",
            tokens: &[],
        }
    }
}

impl<'i, 'b> PartialEq for Tokens<'i, 'b> {
    fn eq(&self, other: &Self) -> bool {
        Iterator::eq(self.iter_elements(), other.iter_elements())
    }
}

#[cfg(feature = "dbg")]
impl<'i, 'b> core::fmt::Debug for Tokens<'i, 'b> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct X<'a>(Token, &'a str);

        impl core::fmt::Debug for X<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:?} {:?}", self.0, self.1)
            }
        }

        let v: Vec<_> = self
            .tokens
            .iter()
            .map(|&(t, (start, end))| X(t, &self.source[start..end]))
            .collect();

        v.fmt(f)
    }
}

impl<'i, 'b> Iterator for Tokens<'i, 'b> {
    type Item = (Token, &'i str);

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.split_first() {
            Some((&(token, range), rest)) => {
                self.tokens = rest;
                Some((token, &self.source[range.0..range.1]))
            }
            None => None,
        }
    }
}

impl<'i, 'b> InputIter for Tokens<'i, 'b> {
    type Item = (Token, &'i str);

    type Iter = Enumerate<Self>;

    type IterElem = Self;

    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        Tokens {
            source: self.source,
            tokens: self.tokens,
        }
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
            Err(nom::Needed::Size(
                (count - self.tokens.len()).try_into().unwrap(),
            ))
        }
    }
}

impl<'i, 'b> InputLength for Tokens<'i, 'b> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

impl<'i, 'b> InputTake for Tokens<'i, 'b> {
    fn take(&self, count: usize) -> Self {
        let tokens = &self.tokens[..count];
        let last = tokens.last().map(|&(_, (_, r))| r).unwrap_or_default();

        Tokens {
            source: &self.source[..last],
            tokens,
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (left, right) = self.tokens.split_at(count);
        let limit = left.last().map(|&(_, (_, r))| r).unwrap_or_default();

        (
            Tokens {
                source: &self.source[..limit],
                tokens: left,
            },
            Tokens {
                source: &self.source[limit..],
                tokens: right,
            },
        )
    }
}
