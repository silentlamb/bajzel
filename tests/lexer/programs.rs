use bajzel_lib::lexer::{Lexer, Token};
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
        Token::UserIdent("const_command"),
        Token::StringLiteral("HELLO"),
        Token::As,
        Token::UserIdent("prefix"),
        Token::StringLiteral(":"),
        Token::As,
        Token::UserIdent("delim"),
        Token::StringLiteral("WORLD"),
        Token::As,
        Token::UserIdent("suffix"),
        Token::Generate,
        Token::UserIdent("const_command"),
        Token::With,
        Token::UserIdent("OUT_MAX"),
        Token::Assign,
        Token::IntegerLiteral(4096),
        Token::UserIdent("TERM"),
        Token::Assign,
        Token::ReservedIdent("null"),
        Token::Eof,
    ]);

    let actual = Lexer::lex_tokens(input);
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
    let output = Lexer::lex_tokens(input);
    let expected = Ok(vec![
        Token::Define,
        Token::UserIdent("ints_command"),
        Token::Type("i32"),
        Token::As,
        Token::UserIdent("cmd"),
        Token::StringLiteral(","),
        Token::As,
        Token::UserIdent("delim1"),
        Token::Type("i64"),
        Token::As,
        Token::UserIdent("param1"),
        Token::StringLiteral(" "),
        Token::As,
        Token::UserIdent("delim2"),
        Token::Type("i8"),
        Token::As,
        Token::UserIdent("param2"),
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
    let output = Lexer::lex_tokens(input);
    let expected = Ok(vec![
        Token::Define,
        Token::UserIdent("uints_command"),
        Token::Type("u8"),
        Token::As,
        Token::UserIdent("x1"),
        Token::Type("u32"),
        Token::As,
        Token::UserIdent("x2"),
        Token::Type("u64"),
        Token::As,
        Token::UserIdent("x3"),
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
    let output = Lexer::lex_tokens(input);
    let expected = Ok(vec![
        Token::Define,
        Token::UserIdent("ints_command"),
        Token::Type("i32"),
        Token::As,
        Token::UserIdent("cmd"),
        Token::StringLiteral(","),
        Token::Type("i64"),
        Token::As,
        Token::UserIdent("param1"),
        Token::StringLiteral(" "),
        Token::Type("i8"),
        Token::As,
        Token::UserIdent("param2"),
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
        Lexer::lex_tokens(input_long).expect("Long program is lexed properly");
    let output_short = Lexer::lex_tokens(input_short)
        .expect("Short program is lexed properly");
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
    let output = Lexer::lex_tokens(input);
    let expected = Ok(vec![
        Token::Define,
        Token::UserIdent("cmd"),
        Token::Type("i8"),
        Token::As,
        Token::UserIdent("cmd"),
        Token::Where,
        Token::UserIdent("cmd"),
        Token::RightArrow,
        Token::UserIdent("RANGE"),
        Token::LeftParen,
        Token::IntegerLiteral(0),
        Token::IntegerLiteral(5),
        Token::RightParen,
        Token::UserIdent("FORMAT"),
        Token::LeftParen,
        Token::UserIdent("hex"),
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
    let output = Lexer::lex_tokens(input.as_str());

    let conv_fn = |x| u8::from_str_radix(x, 16).unwrap();
    let expected_bytes1 =
        expected_bytes1.into_iter().map(conv_fn).collect_vec();
    let expected_bytes2 =
        expected_bytes2.into_iter().map(conv_fn).collect_vec();
    let expected = Ok(vec![
        Token::Define,
        Token::UserIdent("hex_cmd"),
        Token::Bytes(expected_bytes1),
        Token::As,
        Token::UserIdent("magic"),
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
    let output = Lexer::lex_tokens(input);
    let expected = Ok(vec![
        Token::Define,
        Token::UserIdent("hex_cmd"),
        Token::Illegal("`"),
        Token::UserIdent("g5"),
        Token::UserIdent("d6"),
        Token::UserIdent("ha"),
        Token::UserIdent("z4"),
        Token::Illegal("`"),
        Token::Eof,
    ]);

    assert_eq!(output, expected);
}
