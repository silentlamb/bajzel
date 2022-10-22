use bajzel_lib::{
    lexer::{Token, Tokens},
    parser::{parse_tokens, Expr, Literal, Program, Statement},
};
use pretty_assertions::assert_eq;

#[test]
fn define_group() {
    let input = vec![
        Token::Define,
        Token::Ident("long_string_to_mess_with_formatter"),
        Token::Eof,
    ];
    let input = Tokens::new(&input);
    let output = parse_tokens(input);

    let expected: Program = vec![
        Statement::StartGroupDefinition(
            "long_string_to_mess_with_formatter".into(),
        ),
        Statement::Run,
    ]
    .into();

    assert_eq!(output, Ok(expected));
}

#[test]
fn define_group_with_fields() {
    let input = vec![
        Token::Define,
        Token::Ident("long_string_to_mess_with_formatter"),
        Token::Type("u32"),
        Token::As,
        Token::Ident("size"),
        Token::IntegerLiteral(42),
        Token::Eof,
    ];
    let input = Tokens::new(&input);
    let output = parse_tokens(input);

    let expected: Program = vec![
        Statement::StartGroupDefinition(
            "long_string_to_mess_with_formatter".into(),
        ),
        Statement::DefineVariableField("u32".into(), Some("size".into())),
        Statement::DefineConstField(Literal::IntegerLiteral(42), None),
        Statement::Run,
    ]
    .into();
    assert_eq!(output, Ok(expected));
}

#[test]
fn define_group_where_fields_updated() {
    let input = vec![
        Token::Define,
        Token::Ident("cmd"),
        Token::Type("u32"),
        Token::As,
        Token::Ident("size"),
        Token::StringLiteral("hello world"),
        Token::Where,
        Token::Ident("size"),
        Token::RightArrow,
        Token::Ident("RANGE"),
        Token::LeftParen,
        Token::IntegerLiteral(0),
        Token::IntegerLiteral(7),
        Token::RightParen,
        Token::Ident("FORMAT"),
        Token::LeftParen,
        Token::StringLiteral("hex"),
        Token::RightParen,
        Token::Eof,
    ];
    let input = Tokens::new(&input);
    let output = parse_tokens(input);

    let expected: Program = vec![
        Statement::StartGroupDefinition("cmd".into()),
        Statement::DefineVariableField("u32".into(), Some("size".into())),
        Statement::DefineConstField(
            Literal::StringLiteral("hello world".into()),
            None,
        ),
        Statement::StartFieldsSection,
        Statement::MakeCurrentField("size".into()),
        Statement::UpdateField(
            "RANGE".into(),
            Expr::Group(vec![
                Expr::LiteralExpr(Literal::IntegerLiteral(0)),
                Expr::LiteralExpr(Literal::IntegerLiteral(7)),
            ]),
        ),
        Statement::UpdateField(
            "FORMAT".into(),
            Expr::LiteralExpr(Literal::StringLiteral("hex".into())),
        ),
        Statement::Run,
    ]
    .into();
    assert_eq!(output, Ok(expected));
}

#[test]
fn define_generator() {
    let input = vec![
        Token::Generate,
        Token::Ident("very_long_imaginary_command"),
        Token::Eof,
    ];
    let input = Tokens::new(&input);
    let output = parse_tokens(input);

    let expected: Program = vec![
        Statement::StartGeneratorDefinition(
            "very_long_imaginary_command".into(),
        ),
        Statement::Run,
    ]
    .into();

    assert_eq!(output, Ok(expected));
}

#[test]
fn generator_params() {
    let input = vec![
        Token::Generate,
        Token::Ident("cmd"),
        Token::With,
        Token::Ident("OUT_MIN"),
        Token::Assign,
        Token::IntegerLiteral(5),
        Token::Ident("OUT_MAX"),
        Token::Assign,
        Token::IntegerLiteral(42),
        Token::Ident("TERM"),
        Token::Assign,
        Token::ReservedIdent("LF"),
        Token::Eof,
    ];
    let input = Tokens::new(&input);
    let output = parse_tokens(input);
    let expected: Program = vec![
        Statement::StartGeneratorDefinition("cmd".into()),
        Statement::UpdateParam(
            "OUT_MIN".into(),
            Expr::LiteralExpr(Literal::IntegerLiteral(5)),
        ),
        Statement::UpdateParam(
            "OUT_MAX".into(),
            Expr::LiteralExpr(Literal::IntegerLiteral(42)),
        ),
        Statement::UpdateParam(
            "TERM".into(),
            Expr::LiteralExpr(Literal::Reserved("LF".into())),
        ),
        Statement::Run,
    ]
    .into();

    assert_eq!(output, Ok(expected));
}
