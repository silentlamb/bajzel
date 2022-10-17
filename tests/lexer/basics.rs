use bajzel_lib::lexer::{Lexer, Token};
use itertools::Itertools;
use pretty_assertions::assert_eq;

#[test]
fn keywords() {
    let input = "DEFINE AS WITH WHERE";
    let output = Lexer::lex_tokens(input);
    let expected = vec![
        Token::Define,
        Token::As,
        Token::With,
        Token::Where,
        Token::Eof,
    ];
    assert_eq!(output, Ok(expected));
}

#[test]
fn operators() {
    let input = "= ->";
    let output = Lexer::lex_tokens(input);
    let expected = vec![Token::Assign, Token::RightArrow, Token::Eof];
    assert_eq!(output, Ok(expected));
}

#[test]
fn punctuations() {
    let input = "( )";
    let output = Lexer::lex_tokens(input);
    let expected = vec![Token::LeftParen, Token::RightParen, Token::Eof];
    assert_eq!(output, Ok(expected));
}

#[test]
fn types() {
    let input = "i8 i32 i64 u8 u32 u64";
    let output = Lexer::lex_tokens(input);
    let expected = vec![
        Token::Type("i8"),
        Token::Type("i32"),
        Token::Type("i64"),
        Token::Type("u8"),
        Token::Type("u32"),
        Token::Type("u64"),
        Token::Eof,
    ];
    assert_eq!(output, Ok(expected));
}

#[test]
fn reserved_idents() {
    let input = "null";
    let output = Lexer::lex_tokens(input);
    let expected = vec![Token::ReservedIdent("null"), Token::Eof];
    assert_eq!(output, Ok(expected));
}

#[test]
fn const_bytes() {
    let input = "`de ad c0 0f fe`";
    let output = Lexer::lex_tokens(input);
    let expected = vec![Token::Bytes(vec![222, 173, 192, 15, 254]), Token::Eof];
    assert_eq!(output, Ok(expected));
}

#[test]
fn const_strings() {
    let input = r#""Hello, world!""#;
    let output = Lexer::lex_tokens(input);
    let expected = vec![Token::StringLiteral("Hello, world!"), Token::Eof];
    assert_eq!(output, Ok(expected));
}

#[test]
fn const_integers() {
    let input = "0 -9223372036854775808 9223372036854775807";
    let output = Lexer::lex_tokens(input);
    let expected = vec![
        Token::IntegerLiteral(0),
        Token::IntegerLiteral(-9223372036854775808),
        Token::IntegerLiteral(9223372036854775807),
        Token::Eof,
    ];
    assert_eq!(output, Ok(expected));
}

#[test]
fn const_integers_overflowing() {
    let input = r#"
        -170141183460469231731687303715884105728
        170141183460469231731687303715884105727
    "#;
    let output = Lexer::lex_tokens(input);
    let invalids = {
        if let Ok(ref tokens) = output {
            tokens
                .iter()
                .filter(|token| match token {
                    Token::Illegal(_) => true,
                    _ => false,
                })
                .count()
        } else {
            0
        }
    };
    // Count invalids so we don't need to add them into expected vector
    assert_eq!(invalids, 41);

    // Filter out Invalids as we already figured it out
    let output = output.map(|tokens| {
        tokens
            .into_iter()
            .filter(|token| match token {
                Token::Illegal(_) => false,
                _ => true,
            })
            .collect_vec()
    });

    // When there's too many integer numbers, only the last part (of size i64) is parsed
    let expected = vec![
        Token::IntegerLiteral(1687303715884105728),
        Token::IntegerLiteral(1687303715884105727),
        Token::Eof,
    ];
    assert_eq!(output, Ok(expected));
}

#[test]
fn const_integers_double_minus() {
    let input = "--1";
    let output = Lexer::lex_tokens(input);
    let expected =
        vec![Token::Illegal("-"), Token::IntegerLiteral(-1), Token::Eof];
    assert_eq!(output, Ok(expected));
}

#[test]
fn const_strings_emoji() {
    let input = "\"🙈🙉🙊\"";
    let output = Lexer::lex_tokens(input);
    let expected = vec![Token::StringLiteral("🙈🙉🙊"), Token::Eof];
    assert_eq!(output, Ok(expected));
}

#[test]
fn emoji_idents_not_allowed() {
    let input = "🙈🙉🙊";
    let output = Lexer::lex_tokens(input);
    let expected = vec![
        Token::Illegal("🙈"),
        Token::Illegal("🙉"),
        Token::Illegal("🙊"),
        Token::Eof,
    ];
    assert_eq!(output, Ok(expected));
}

#[test]
fn ref_token() {
    let input = "ref";
    let output = Lexer::lex_tokens(input);
    let expected = vec![Token::Type("ref"), Token::Eof];
    assert_eq!(output, Ok(expected));
}

#[test]
fn comments_are_discarded() {
    let input = r#"
        42 # 43
        44 # 45 46 47
    "#;
    let output = Lexer::lex_tokens(input);
    let expected = vec![
        Token::IntegerLiteral(42),
        Token::IntegerLiteral(44),
        Token::Eof,
    ];
    assert_eq!(output, Ok(expected));
}
