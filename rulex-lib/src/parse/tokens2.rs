use logos::{Lexer, Span, SpannedIter};
use nom::{InputIter, InputLength, InputTake};

use super::token::Token;

#[derive(Clone)]
pub(crate) struct Tokens<'i> {
    pub(super) logos: Lexer<'i, Token>,
}

impl<'i> Tokens<'i> {
    pub(crate) fn tokenize(input: &'i str) -> Self {
        Tokens {
            logos: Lexer::new(input),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.logos.source().len() - self.logos.span().end
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn peek(&self) -> Option<(Token, &'i str)> {
        let mut iter = self.logos.clone().spanned();
        iter.next().map(|(t, span)| (t, &self.logos.source()[span]))
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        Tokens {
            logos: Lexer::new(""),
        }
    }
}

impl<'i> PartialEq for Tokens<'i> {
    fn eq(&self, other: &Self) -> bool {
        Iterator::eq(self.logos.clone(), other.logos.clone())
    }
}

#[cfg(feature = "dbg")]
impl core::fmt::Debug for Tokens<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source = self.logos.source();
        let start = self.logos.span().start;
        f.debug_tuple("Tokens").field(&&source[start..]).finish()
    }
}

pub(crate) struct PosIter<'i> {
    logos: Lexer<'i, Token>,
}

impl<'i> Iterator for PosIter<'i> {
    type Item = (usize, (Token, Span));

    fn next(&mut self) -> Option<Self::Item> {
        self.logos.next().map(|t| {
            let span = self.logos.span();
            (span.start, (t, span))
        })
    }
}

impl<'i> InputIter for Tokens<'i> {
    type Item = (Token, Span);

    type Iter = PosIter<'i>;

    type IterElem = SpannedIter<'i, Token>;

    fn iter_indices(&self) -> Self::Iter {
        PosIter {
            logos: self.logos.clone(),
        }
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.logos.clone().spanned()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        let mut iter = self.logos.clone().spanned();
        iter.find(|t| predicate(t.clone()))
            .map(|(_, span)| span.start)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        let mut cnt = 0;
        for (_, span) in self.logos.clone().spanned() {
            if cnt == count {
                return Ok(span.start);
            }
            cnt += 1;
        }
        if cnt == count {
            return Ok(self.len());
        }
        Err(nom::Needed::Unknown)
    }
}

impl InputLength for Tokens<'_> {
    fn input_len(&self) -> usize {
        self.len()
    }
}

impl InputTake for Tokens<'_> {
    fn take(&self, count: usize) -> Self {
        Tokens {
            logos: Lexer::new(&self.logos.source()[..count]),
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (a, b) = self.logos.source().split_at(count);
        (
            Tokens {
                logos: Lexer::new(a),
            },
            Tokens {
                logos: Lexer::new(b),
            },
        )
    }
}
