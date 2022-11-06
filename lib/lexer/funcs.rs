use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take};
use nom::character::complete::{
    alpha1, alphanumeric1, char, digit1, multispace0,
};
use nom::combinator::{map, map_res, opt, recognize, value};
use nom::multi::many0;
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;
use std::borrow::Cow;
use std::num::ParseIntError;

use super::Token;

static TYPES: &[&str; 21] = &[
    "i8", "i32", "i64", "u8", "u32", "u64", "le_u16", "le_u32", "le_u64",
    "le_i16", "le_i32", "le_i64", "be_u16", "be_u32", "be_u64", "be_i16",
    "be_i32", "be_i64", "bytes", "ref", "string",
];
static RESERVED: &[&str; 2] = &["null", "lf"];

/// Matches any number of whitespaces followed by # character followed by anything that's not \n
///
pub(crate) fn comment_to_eol(input: &str) -> IResult<&str, Token<'_>> {
    value(
        Token::Comment,
        delimited(multispace0, char('#'), is_not("\n")),
    )(input)
}

/// Matches any known token preceded with any number of whitespaces followed
/// by any number of whitespaces.
///
pub(crate) fn space_separated_token(input: &str) -> IResult<&str, Token<'_>> {
    delimited(multispace0, lex_token, multispace0)(input)
}

/// Lex input into a single token
///
fn lex_token(input: &str) -> IResult<&str, Token<'_>> {
    alt((
        lex_string,
        lex_bytes,
        lex_ident_array,
        lex_ident,
        lex_integer,
        lex_operator,
        lex_punctuations,
        lex_comment, // No idea why "comment_to_eol" doesn't solve all comment issues, but this one helps
        lex_illegal,
    ))(input)
}

/// Lex input into a string literal
///
fn lex_string(input: &str) -> IResult<&str, Token<'_>> {
    map(
        delimited(char('"'), is_not("\""), char('"')),
        Token::StringLiteral,
    )(input)
}

/// Lex input into a byte sequence
///
/// Byte sequence syntax: `41 42 43 44 4f`
///
fn lex_bytes(input: &str) -> IResult<&str, Token<'_>> {
    map(
        map_res(delimited(char('`'), is_not("`"), char('`')), |x: &str| {
            x.split(' ')
                .map(|s| u8::from_str_radix(s, 16))
                .collect::<Vec<_>>()
                .into_iter()
                .collect::<Result<Vec<_>, ParseIntError>>()
        }),
        Token::Bytes,
    )(input)
}

fn lex_ident_array(input: &str) -> IResult<&str, Token<'_>> {
    map(
        tuple((
            recognize(pair(alpha1, many0(alt((alphanumeric1, tag("_")))))),
            delimited(
                char('['),
                map_res(recognize(pair(opt(char('-')), digit1)), |x: &str| {
                    x.parse::<u32>()
                }),
                char(']'),
            ),
        )),
        |(name, size)| Token::TypeArray(name, size),
    )(input)
}

/// Lex input into an indentifier
///
fn lex_ident(input: &str) -> IResult<&str, Token<'_>> {
    map(
        recognize(pair(alpha1, many0(alt((alphanumeric1, tag("_")))))),
        |x: &str| {
            let icase_x = x.to_lowercase();
            let icase_x = icase_x.as_str();
            if TYPES.contains(&icase_x) {
                return Token::Type(x);
            }
            if RESERVED.contains(&icase_x) {
                return Token::ReservedIdent(Cow::Owned(x.to_uppercase()));
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
fn lex_integer(input: &str) -> IResult<&str, Token<'_>> {
    map(
        map_res(recognize(pair(opt(char('-')), digit1)), |x: &str| {
            // i64::from_str_radix(x, 10)
            x.parse::<i64>()
        }),
        Token::IntegerLiteral,
    )(input)
}

/// Lex input into illegal token
///
fn lex_illegal(input: &str) -> IResult<&str, Token<'_>> {
    map(take(1usize), Token::Illegal)(input)
}

/// Lex input into an operator
///
fn lex_operator(input: &str) -> IResult<&str, Token<'_>> {
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
fn lex_punctuations(input: &str) -> IResult<&str, Token<'_>> {
    alt((
        map(tag("("), |_| Token::LeftParen),
        map(tag(")"), |_| Token::RightParen),
        map(tag(","), |_| Token::Comma),
        map(tag("$"), |_| Token::Reference),
        map(tag(":"), |_| Token::Colon),
    ))(input)
}

fn lex_comment(input: &str) -> IResult<&str, Token<'_>> {
    map(tag("#"), |_| Token::Comment)(input)
}
