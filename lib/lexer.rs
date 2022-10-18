use itertools::Itertools;
use nom::branch::*;
use nom::bytes::complete::{is_not, tag, take};
use nom::character::complete::{
    alpha1, alphanumeric1, char, digit1, multispace0,
};
use nom::combinator::{map, map_res, opt, recognize, value};
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::IResult;
use std::iter::Enumerate;
use std::num::ParseIntError;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

// static ARRAY_TYPES: &[&str; 1] = &["bytes"];
static TYPES: &[&str; 21] = &[
    "i8", "i32", "i64", "u8", "u32", "u64", "le_u16", "le_u32", "le_u64",
    "le_i16", "le_i32", "le_i64", "be_u16", "be_u32", "be_u64", "be_i16",
    "be_i32", "be_i64", "bytes", "ref", "string",
];
static RESERVED: &[&str; 2] = &["null", "lf"];

pub struct Lexer;

fn comment_to_eol<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Token<'a>> {
    preceded(
        multispace0,
        value(Token::Comment, pair(char('#'), is_not("\r\n"))),
    )
}

fn space_separated_token<'a>(
) -> impl FnMut(&'a str) -> IResult<&'a str, Token<'a>> {
    delimited(multispace0, Lexer::lex_token, multispace0)
}

impl Lexer {
    /// Entrypoint - lex input into tokens
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

    /// Lex input into a single token
    ///
    fn lex_token(input: &'_ str) -> IResult<&'_ str, Token<'_>> {
        alt((
            Lexer::lex_string,
            Lexer::lex_bytes,
            Lexer::lex_ident_array,
            Lexer::lex_ident,
            Lexer::lex_integer,
            Lexer::lex_operator,
            Lexer::lex_punctuations,
            Lexer::lex_illegal,
        ))(input)
    }

    /// Lex input into an operator
    ///
    fn lex_operator(input: &'_ str) -> IResult<&'_ str, Token<'_>> {
        // For a now it's only assignment operator, but let's keep it this way
        alt((
            map(tag("="), |_| Token::Assign),
            map(tag("+"), |_| Token::Add),
            map(tag("->"), |_| Token::RightArrow),
            map(tag("-"), |_| Token::Subtract),
            map(tag("*"), |_| Token::Multiply),
        ))(input)
    }

    /// Lex input into punctuations
    ///
    fn lex_punctuations(input: &'_ str) -> IResult<&'_ str, Token<'_>> {
        alt((
            map(tag("("), |_| Token::LeftParen),
            map(tag(")"), |_| Token::RightParen),
            map(tag(","), |_| Token::Comma),
            map(tag("$"), |_| Token::Reference),
            map(tag(":"), |_| Token::Colon),
        ))(input)
    }

    /// Lex input into a string literal
    ///
    fn lex_string(input: &'_ str) -> IResult<&'_ str, Token<'_>> {
        map(
            delimited(char('"'), is_not("\""), char('"')),
            Token::StringLiteral,
        )(input)
    }

    /// Lex input into a byte sequence
    ///
    fn lex_bytes(input: &'_ str) -> IResult<&'_ str, Token<'_>> {
        map(
            map_res(delimited(char('`'), is_not("`"), char('`')), |x: &str| {
                x.split(" ")
                    .map(|s| u8::from_str_radix(s, 16))
                    .collect::<Vec<_>>()
                    .into_iter()
                    .collect::<Result<Vec<_>, ParseIntError>>()
            }),
            Token::Bytes,
        )(input)
    }

    fn lex_ident_array<'a>(input: &'a str) -> IResult<&'a str, Token<'a>> {
        map(
            tuple((
                recognize(pair(alpha1, many0(alt((alphanumeric1, tag("_")))))),
                delimited(
                    char('['),
                    map_res(recognize(pair(opt(char('-')), digit1)), |x| {
                        u32::from_str_radix(x, 10)
                    }),
                    char(']'),
                ),
            )),
            |(name, size)| Token::TypeArray(name, size),
        )(input)
    }

    /// Lex input into an indentifier
    ///
    fn lex_ident<'a>(input: &'a str) -> IResult<&'a str, Token<'a>> {
        map(
            recognize(pair(alpha1, many0(alt((alphanumeric1, tag("_")))))),
            |x: &str| {
                let icase_x = x.to_lowercase();
                let icase_x = icase_x.as_str();
                if TYPES.contains(&icase_x) {
                    return Token::Type(x);
                }
                if RESERVED.contains(&icase_x) {
                    return Token::ReservedIdent(x);
                }
                match icase_x {
                    // Keywords
                    "as" => Token::As,
                    "define" => Token::Define,
                    "from" => Token::From,
                    "generate" => Token::Generate,
                    "where" => Token::Where,
                    "with" => Token::With,
                    _ => Token::Ident(x),
                }
            },
        )(input)
    }

    /// Lex input into an integer
    ///
    fn lex_integer(input: &'_ str) -> IResult<&'_ str, Token<'_>> {
        map(
            map_res(recognize(pair(opt(char('-')), digit1)), |x| {
                i64::from_str_radix(x, 10)
            }),
            Token::IntegerLiteral,
        )(input)
    }

    /// Lex input into illegal token
    ///
    fn lex_illegal(input: &'_ str) -> IResult<&'_ str, Token<'_>> {
        map(take(1usize), |x| Token::Illegal(x))(input)
    }
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
