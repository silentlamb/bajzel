mod funcs;

use itertools::Itertools;
use nom::branch::*;
use nom::multi::many0;
use std::iter::Enumerate;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

use self::funcs::{comment_to_eol, space_separated_token};

/// Entrypoint - lex input into tokens
///
pub fn lex_tokens<'a>(
    input: &'a str,
) -> Result<Vec<Token<'a>>, nom::Err<nom::error::Error<&str>>> {
    many0(alt((comment_to_eol(), space_separated_token())))(input).map(
        |(_, tokens)| {
            itertools::chain(
                tokens.into_iter().filter(|token| match token {
                    Token::Comment => false,
                    _ => true,
                }),
                Some(Token::Eof).into_iter(),
            )
            .collect_vec()
        },
    )
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'a> {
    Add,
    As,
    Assign,
    Bytes(Vec<u8>),
    Colon,
    Comma,
    Comment,
    Define,
    Eof,
    From,
    Generate,
    Ident(&'a str),
    Illegal(&'a str),
    IntegerLiteral(i64),
    LeftParen,
    Multiply,
    Reference,
    ReservedIdent(&'a str),
    RightArrow,
    RightParen,
    StringLiteral(&'a str),
    Subtract,
    Type(&'a str),
    TypeArray(&'a str, u32),
    Where,
    With,
}

impl<'a> nom::InputLength for Token<'a> {
    fn input_len(&self) -> usize {
        1
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens<'a> {
    pub tokens: &'a [Token<'a>],
    pub start: usize,
    pub end: usize,
}

impl<'a> Tokens<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Tokens {
            tokens,
            start: 0,
            end: 0,
        }
    }
}

impl<'a> nom::InputLength for Tokens<'a> {
    fn input_len(&self) -> usize {
        self.tokens.len()
    }
}

impl<'a> nom::InputTake for Tokens<'a> {
    fn take(&self, count: usize) -> Self {
        Tokens {
            tokens: &self.tokens[0..count],
            start: 0,
            end: count,
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (x, y) = self.tokens.split_at(count);
        let x = Tokens {
            tokens: x,
            start: 0,
            end: x.len(),
        };
        let y = Tokens {
            tokens: y,
            start: 0,
            end: y.len(),
        };
        (y, x)
    }
}

impl<'a> nom::Slice<Range<usize>> for Tokens<'a> {
    fn slice(&self, range: Range<usize>) -> Self {
        Tokens {
            tokens: self.tokens.slice(range.clone()),
            start: self.start + range.start,
            end: self.end + range.end,
        }
    }
}

impl<'a> nom::Slice<RangeFrom<usize>> for Tokens<'a> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        self.slice(range.start..self.end - self.start)
    }
}

impl<'a> nom::Slice<RangeTo<usize>> for Tokens<'a> {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        self.slice(0..range.end)
    }
}

impl<'a> nom::Slice<RangeFull> for Tokens<'a> {
    fn slice(&self, _range: RangeFull) -> Self {
        self.clone()
    }
}

impl<'a> nom::InputIter for Tokens<'a> {
    type Item = &'a Token<'a>;
    type Iter = Enumerate<::std::slice::Iter<'a, Token<'a>>>;
    type IterElem = ::std::slice::Iter<'a, Token<'a>>;

    fn iter_indices(&self) -> Self::Iter {
        self.tokens.iter().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.tokens.iter()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.tokens.iter().position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if self.tokens.len() >= count {
            Ok(count)
        } else {
            Err(nom::Needed::Unknown)
        }
    }
}
