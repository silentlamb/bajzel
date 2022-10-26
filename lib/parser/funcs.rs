use super::{Expr, Ident, Literal, Program, Statement, Tokens};
use crate::lexer::Token;
use nom::{
    branch::alt,
    bytes::complete::take,
    combinator::{map, opt, verify},
    error::{Error, ErrorKind},
    multi::{many0, many1},
    sequence::{delimited, pair, preceded, separated_pair, terminated},
    Err, IResult,
};

pub(crate) fn parse_program(input: Tokens) -> IResult<Tokens, Program> {
    let parse_stmt = many0(parse_statement);
    let parse_prog = terminated(parse_stmt, eof_tag);
    // map(parse_prog, flatten_program)(input)
    map(parse_prog, |x| {
        let mut x = flatten_vec(x);
        x.push(Statement::Run);
        x.into()
    })(input)
}

fn parse_statement(input: Tokens) -> IResult<Tokens, Vec<Statement>> {
    alt((
        map(parse_define_group_statement, single_to_vec),
        parse_define_field_statement,
        map(parse_update_field_section_statement, single_to_vec),
        parse_update_field_statement,
        map(parse_define_generator_statement, single_to_vec),
        map(parse_set_param_statement, single_to_vec),
    ))(input)
}

fn parse_define_group_statement(input: Tokens) -> IResult<Tokens, Statement> {
    map(
        preceded(define_tag, parse_ident),
        Statement::StartGroupDefinition,
    )(input)
}

fn parse_define_generator_statement(
    input: Tokens,
) -> IResult<Tokens, Statement> {
    map(
        delimited(generate_tag, parse_ident, opt(with_tag)),
        Statement::StartGeneratorDefinition,
    )(input)
}

fn parse_define_field_statement(
    input: Tokens,
) -> IResult<Tokens, Vec<Statement>> {
    alt((
        map(
            pair(
                parse_type,
                opt(preceded(
                    as_tag,
                    pair(
                        parse_ident,
                        opt(preceded(
                            right_arrow_tag,
                            many1(parse_update_fields),
                        )),
                    ),
                )),
            ),
            |(name, extra)| {
                let (alias, updates) = match extra {
                    Some((alias, updates)) => (Some(alias), updates),
                    None => (None, None),
                };
                let mut res =
                    vec![Statement::DefineVariableField(name, alias.clone())];
                if let Some(statements) = updates {
                    let alias = alias.expect("Guaranteed to be Some");
                    res.push(Statement::MakeCurrentField(alias));
                    res.extend(statements.into_iter());
                }
                res
            },
        ),
        map(
            pair(parse_literal, opt(preceded(as_tag, parse_ident))),
            |(value, alias)| vec![Statement::DefineConstField(value, alias)],
        ),
    ))(input)
}

fn parse_update_field_statement(
    input: Tokens,
) -> IResult<Tokens, Vec<Statement>> {
    map(
        separated_pair(
            parse_ident,
            right_arrow_tag,
            many1(parse_update_fields),
        ),
        |(ident, exp_vec)| {
            let mut ret = vec![Statement::MakeCurrentField(ident)];
            ret.extend(exp_vec.into_iter());
            ret
        },
    )(input)
}

fn parse_update_field_section_statement(
    input: Tokens,
) -> IResult<Tokens, Statement> {
    map(where_tag, |_| Statement::StartFieldsSection)(input)
}

fn parse_set_param_statement(input: Tokens) -> IResult<Tokens, Statement> {
    let p = separated_pair(parse_ident, assign_tag, parse_expr);
    map(p, |(ident, expr)| Statement::UpdateParam(ident, expr))(input)
}

fn parse_ident(input: Tokens) -> IResult<Tokens, Ident> {
    let (rest, t) = take(1usize)(input)?;
    if t.tokens.is_empty() {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    } else {
        match t.tokens[0] {
            Token::Ident(x) => Ok((rest, Ident(x.to_owned()))),
            _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
        }
    }
}

fn parse_type(input: Tokens) -> IResult<Tokens, String> {
    let (rest, t) = take(1usize)(input)?;
    if t.tokens.is_empty() {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    } else {
        match t.tokens[0] {
            Token::Type(name) => Ok((rest, name.to_owned())),
            _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
        }
    }
}

fn parse_literal(input: Tokens) -> IResult<Tokens, Literal> {
    let (rest, t) = take(1usize)(input)?;
    if t.tokens.is_empty() {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    } else {
        match t.tokens[0].clone() {
            Token::IntegerLiteral(x) => Ok((rest, Literal::IntegerLiteral(x))),
            Token::StringLiteral(x) => {
                Ok((rest, Literal::StringLiteral(x.to_owned())))
            }
            Token::Bytes(x) => Ok((rest, Literal::BytesLiteral(x))),
            Token::ReservedIdent(x) => Ok((rest, Literal::Reserved(x.into()))),
            _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
        }
    }
}

fn parse_update_fields(input: Tokens) -> IResult<Tokens, Statement> {
    let p = pair(
        parse_ident,
        delimited(open_paren_tag, many1(parse_expr), close_paren_tag),
    );

    map(p, |(ident, inner)| {
        let expr = if inner.len() == 1 {
            inner.into_iter().next().expect("Just checked the size")
        } else {
            Expr::Group(inner)
        };
        Statement::UpdateField(ident, expr)
    })(input)
}

fn parse_expr(input: Tokens) -> IResult<Tokens, Expr> {
    alt((map(parse_literal, Expr::LiteralExpr),))(input)
}

// https://github.com/Rydgel/monkey-rust/blob/master/lib/parser/mod.rs#L15
macro_rules! tag_token (
    ($func_name:ident, $tag:expr) => (
        fn $func_name(tokens: Tokens) -> IResult<Tokens, Tokens> {
            verify(take(1usize), |t: &Tokens| t.tokens[0] == $tag)(tokens)
        }
    )
);

tag_token!(as_tag, Token::As);
tag_token!(assign_tag, Token::Assign);
tag_token!(close_paren_tag, Token::RightParen);
tag_token!(define_tag, Token::Define);
tag_token!(generate_tag, Token::Generate);
tag_token!(eof_tag, Token::Eof);
tag_token!(open_paren_tag, Token::LeftParen);
tag_token!(right_arrow_tag, Token::RightArrow);
tag_token!(where_tag, Token::Where);
tag_token!(with_tag, Token::With);

fn single_to_vec<T>(single: T) -> Vec<T> {
    vec![single]
}

fn flatten_vec<T>(nested: Vec<Vec<T>>) -> Vec<T> {
    nested.into_iter().flatten().collect()
}
