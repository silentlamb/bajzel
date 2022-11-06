use bajzel_lib::{
    lexer::{Token, Tokens},
    parser::{parse_tokens, Expr, Literal, Program, Statement},
};
use pretty_assertions::assert_eq;
use std::borrow::Cow;

#[test]
fn example0() {
    let tokens = vec![
        Token::Define,
        Token::Ident("string_cmd"),
        Token::Type("string"),
        Token::As,
        Token::Ident("cmd"),
        Token::RightArrow,
        Token::Ident("LEN"),
        Token::LeftParen,
        Token::IntegerLiteral(4),
        Token::RightParen,
        Token::Comma,
        Token::StringLiteral(" "),
        Token::Type("string"),
        Token::As,
        Token::Ident("sub_cmd"),
        Token::RightArrow,
        Token::Ident("LEN"),
        Token::LeftParen,
        Token::IntegerLiteral(1),
        Token::IntegerLiteral(10),
        Token::RightParen,
        Token::Comma,
        Token::StringLiteral(" "),
        Token::Type("i32"),
        Token::As,
        Token::Ident("param"),
        Token::RightArrow,
        Token::Ident("RANGE"),
        Token::LeftParen,
        Token::IntegerLiteral(0),
        Token::IntegerLiteral(5),
        Token::RightParen,
        Token::Comma,
        Token::Generate,
        Token::Ident("string_cmd"),
        Token::With,
        Token::Ident("OUT_MIN"),
        Token::Assign,
        Token::IntegerLiteral(5),
        Token::Ident("OUT_MAX"),
        Token::Assign,
        Token::IntegerLiteral(32),
        Token::Ident("TERM"),
        Token::Assign,
        Token::ReservedIdent(Cow::Borrowed("LF")),
        Token::Eof,
    ];
    let tokens = Tokens::new(&tokens);
    let actual = parse_tokens(tokens);
    let expected: Program = vec![
        Statement::StartGroupDefinition("string_cmd".into()),
        Statement::DefineVariableField("string".to_owned(), Some("cmd".into())),
        Statement::MakeCurrentField("cmd".into()),
        Statement::UpdateField(
            "LEN".into(),
            Expr::LiteralExpr(Literal::IntegerLiteral(4)),
        ),
        Statement::DefineConstField(
            Literal::StringLiteral(" ".to_owned()),
            None,
        ),
        Statement::DefineVariableField(
            "string".to_owned(),
            Some("sub_cmd".into()),
        ),
        Statement::MakeCurrentField("sub_cmd".into()),
        Statement::UpdateField(
            "LEN".into(),
            Expr::Group(vec![
                Expr::LiteralExpr(Literal::IntegerLiteral(1)),
                Expr::LiteralExpr(Literal::IntegerLiteral(10)),
            ]),
        ),
        Statement::DefineConstField(
            Literal::StringLiteral(" ".to_owned()),
            None,
        ),
        Statement::DefineVariableField("i32".to_owned(), Some("param".into())),
        Statement::MakeCurrentField("param".into()),
        Statement::UpdateField(
            "RANGE".into(),
            Expr::Group(vec![
                Expr::LiteralExpr(Literal::IntegerLiteral(0)),
                Expr::LiteralExpr(Literal::IntegerLiteral(5)),
            ]),
        ),
        Statement::StartGeneratorDefinition("string_cmd".into()),
        Statement::UpdateParam(
            "OUT_MIN".into(),
            Expr::LiteralExpr(Literal::IntegerLiteral(5)),
        ),
        Statement::UpdateParam(
            "OUT_MAX".into(),
            Expr::LiteralExpr(Literal::IntegerLiteral(32)),
        ),
        Statement::UpdateParam(
            "TERM".into(),
            Expr::LiteralExpr(Literal::Reserved("LF".into())),
        ),
        Statement::Run,
    ]
    .into();

    assert_eq!(actual, Ok(expected));
}
