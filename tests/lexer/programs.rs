use std::borrow::Cow;

use bajzel_lib::lexer::{lex_tokens, Token};
use itertools::Itertools;
use pretty_assertions::assert_eq;

#[test]
fn string_command() {
    let input = r#"
            DEFINE const_command
                "HELLO" AS prefix
                ":"     AS delim
                "WORLD" AS suffix

            GENERATE const_command WITH
                OUT_MAX = 4096
                TERM = null
        "#;

    let expected = Ok(vec![
        Token::Define,
        Token::Ident("const_command"),
        Token::StringLiteral("HELLO"),
        Token::As,
        Token::Ident("prefix"),
        Token::StringLiteral(":"),
        Token::As,
        Token::Ident("delim"),
        Token::StringLiteral("WORLD"),
        Token::As,
        Token::Ident("suffix"),
        Token::Generate,
        Token::Ident("const_command"),
        Token::With,
        Token::Ident("OUT_MAX"),
        Token::Assign,
        Token::IntegerLiteral(4096),
        Token::Ident("TERM"),
        Token::Assign,
        Token::ReservedIdent(Cow::Borrowed("NULL")),
        Token::Eof,
    ]);

    let actual = lex_tokens(input);
    assert_eq!(actual, expected);
}

#[test]
fn int_command() {
    let input = r#" 
        DEFINE ints_command
            i32 AS cmd
            "," AS delim1
            i64 AS param1
            " " AS delim2
            i8  AS param2
    "#;
    let output = lex_tokens(input);
    let expected = Ok(vec![
        Token::Define,
        Token::Ident("ints_command"),
        Token::Type("i32"),
        Token::As,
        Token::Ident("cmd"),
        Token::StringLiteral(","),
        Token::As,
        Token::Ident("delim1"),
        Token::Type("i64"),
        Token::As,
        Token::Ident("param1"),
        Token::StringLiteral(" "),
        Token::As,
        Token::Ident("delim2"),
        Token::Type("i8"),
        Token::As,
        Token::Ident("param2"),
        Token::Eof,
    ]);
    assert_eq!(output, expected);
}

#[test]
fn uint_command() {
    let input = r#" 
        DEFINE uints_command
            u8  AS x1
            u32 AS x2
            u64 AS x3
    "#;
    let output = lex_tokens(input);
    let expected = Ok(vec![
        Token::Define,
        Token::Ident("uints_command"),
        Token::Type("u8"),
        Token::As,
        Token::Ident("x1"),
        Token::Type("u32"),
        Token::As,
        Token::Ident("x2"),
        Token::Type("u64"),
        Token::As,
        Token::Ident("x3"),
        Token::Eof,
    ]);
    assert_eq!(output, expected);
}

#[test]
fn anynomous_fields() {
    let input = r#" 
    DEFINE ints_command
        i32 AS cmd
        ","
        i64 AS param1
        " "
        i8  AS param2
    "#;
    let output = lex_tokens(input);
    let expected = Ok(vec![
        Token::Define,
        Token::Ident("ints_command"),
        Token::Type("i32"),
        Token::As,
        Token::Ident("cmd"),
        Token::StringLiteral(","),
        Token::Type("i64"),
        Token::As,
        Token::Ident("param1"),
        Token::StringLiteral(" "),
        Token::Type("i8"),
        Token::As,
        Token::Ident("param2"),
        Token::Eof,
    ]);
    assert_eq!(output, expected);
}

#[test]
fn separate_lines_dont_matter() {
    let input_long = r#" 
        DEFINE ints_command
            i32 AS cmd
        GENERATE ints_command WITH
            TERM = null
    "#;
    let input_short = r#"DEFINE ints_command i32 AS cmd GENERATE ints_command WITH TERM = null"#;
    let output_long =
        lex_tokens(input_long).expect("Long program is lexed properly");
    let output_short =
        lex_tokens(input_short).expect("Short program is lexed properly");
    assert_eq!(output_long, output_short);
}

#[test]
fn generate_where_int_props() {
    let input = r#"
    DEFINE cmd
        i8 AS cmd
    WHERE
        cmd -> RANGE(0 5) FORMAT(hex)
    "#;
    let output = lex_tokens(input);
    let expected = Ok(vec![
        Token::Define,
        Token::Ident("cmd"),
        Token::Type("i8"),
        Token::As,
        Token::Ident("cmd"),
        Token::Where,
        Token::Ident("cmd"),
        Token::RightArrow,
        Token::Ident("RANGE"),
        Token::LeftParen,
        Token::IntegerLiteral(0),
        Token::IntegerLiteral(5),
        Token::RightParen,
        Token::Ident("FORMAT"),
        Token::LeftParen,
        Token::Ident("hex"),
        Token::RightParen,
        Token::Eof,
    ]);

    assert_eq!(output, expected);
}

#[test]
fn const_bytes_input() {
    let expected_bytes1 = ["05", "d6", "3a", "c4", "00", "ff"]; // .map(conv_fn).to_vec();
    let expected_bytes2 = ["de", "ad", "c0", "0f", "fe", "00"]; //.map(conv_fn).to_vec();

    let input = format!(
        r#"
    DEFINE hex_cmd
        `{}` AS magic
        `{}`
    "#,
        itertools::join(expected_bytes1, " "),
        itertools::join(expected_bytes2, " "),
    );
    let output = lex_tokens(input.as_str());

    let conv_fn = |x| u8::from_str_radix(x, 16).unwrap();
    let expected_bytes1 =
        expected_bytes1.into_iter().map(conv_fn).collect_vec();
    let expected_bytes2 =
        expected_bytes2.into_iter().map(conv_fn).collect_vec();
    let expected = Ok(vec![
        Token::Define,
        Token::Ident("hex_cmd"),
        Token::Bytes(expected_bytes1),
        Token::As,
        Token::Ident("magic"),
        Token::Bytes(expected_bytes2),
        Token::Eof,
    ]);

    assert_eq!(output, expected);
}

#[test]
fn const_bytes_input_invalid() {
    let input = r#"
    DEFINE hex_cmd
        `g5 d6 ha z4`
    "#;
    let output = lex_tokens(input);
    let expected = Ok(vec![
        Token::Define,
        Token::Ident("hex_cmd"),
        Token::Illegal("`"),
        Token::Ident("g5"),
        Token::Ident("d6"),
        Token::Ident("ha"),
        Token::Ident("z4"),
        Token::Illegal("`"),
        Token::Eof,
    ]);

    assert_eq!(output, expected);
}
