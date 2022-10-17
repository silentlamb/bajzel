use std::num::ParseIntError;

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

// static ARRAY_TYPES: &[&str; 1] = &["bytes"];
static TYPES: &[&str; 20] = &[
    "i8", "i32", "i64", "u8", "u32", "u64", "bytes", "ref", "le_u16", "le_u32",
    "le_u64", "le_i16", "le_i32", "le_i64", "be_u16", "be_u32", "be_u64",
    "be_i16", "be_i32", "be_i64",
];
static RESERVED: &[&str; 1] = &["null"];

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
                if TYPES.contains(&x) {
                    return Token::Type(x);
                }
                if RESERVED.contains(&x) {
                    return Token::ReservedIdent(x);
                }
                match x.to_lowercase().as_str() {
                    // Keywords
                    "as" => Token::As,
                    "define" => Token::Define,
                    "from" => Token::From,
                    "generate" => Token::Generate,
                    "where" => Token::Where,
                    "with" => Token::With,
                    _ => Token::UserIdent(x),
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
    UserIdent(&'a str),
    Where,
    With,
}
