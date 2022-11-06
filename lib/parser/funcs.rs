use super::{Expr, Ident, Literal, Program, Statement, Tokens};
use crate::lexer::Token;
use itertools::Itertools;
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
        map(parse_define_group_where, single_to_vec),
        parse_define_fields,
        parse_update_attrs,
        map(parse_define_generator_statement, single_to_vec),
        map(parse_set_param_statement, single_to_vec),
    ))(input)
}

/// Parse group definition statement
///
/// Input: `DEFINE group_name`
/// Output: StartGroupDefinition(group_name)
///
fn parse_define_group_statement(input: Tokens) -> IResult<Tokens, Statement> {
    map(
        preceded(define_tag, parse_ident),
        Statement::StartGroupDefinition,
    )(input)
}

/// Parse field definition statement
///
/// Input: field_def [AS alias] [-> ATTR(attr_params)]
/// Output: Vec<Statements>
fn parse_define_fields(input: Tokens) -> IResult<Tokens, Vec<Statement>> {
    map(
        pair(parse_field_definition_with_alias, parse_opt_attrs),
        |((decl, alias), attrs)| {
            // First, decl statements were created separately from aliases
            // therefore alias needs to be added if possible
            let decl = if let Some(ident) = &alias {
                match decl {
                    Statement::DefineVariableField(x, None) => {
                        Statement::DefineVariableField(x, Some(ident.clone()))
                    }
                    Statement::DefineConstField(x, None) => {
                        Statement::DefineConstField(x, Some(ident.clone()))
                    }
                    _ => panic!("unexpected statement"),
                }
            } else {
                decl
            };

            // Then partial statements needs to be combined together
            let mut out = vec![decl];
            if let Some(statements) = attrs {
                let alias = alias.expect("alias must be set");
                out.push(Statement::MakeCurrentField(alias));
                out.extend(statements);
            }
            out
        },
    )(input)
}

fn parse_field_definition_with_alias(
    input: Tokens,
) -> IResult<Tokens, (Statement, Option<Ident>)> {
    pair(parse_field_definition, opt(preceded(as_tag, parse_ident)))(input)
}

fn parse_field_definition(input: Tokens) -> IResult<Tokens, Statement> {
    alt((
        map(parse_type, |x| Statement::DefineVariableField(x, None)),
        map(parse_literal, |x| Statement::DefineConstField(x, None)),
    ))(input)
}

fn parse_opt_attrs(input: Tokens) -> IResult<Tokens, Option<Vec<Statement>>> {
    opt(parse_req_attrs)(input)
}

fn parse_req_attrs(input: Tokens) -> IResult<Tokens, Vec<Statement>> {
    delimited(right_arrow_tag, parse_attrs_list, comma_tag)(input)
}

fn parse_attrs_list(input: Tokens) -> IResult<Tokens, Vec<Statement>> {
    map(many1(parse_single_attr), |x| {
        x.into_iter()
            .map(|(ident, expr)| Statement::UpdateField(ident, expr))
            .collect_vec()
    })(input)
}

fn parse_single_attr(input: Tokens) -> IResult<Tokens, (Ident, Expr)> {
    pair(
        parse_ident,
        delimited(open_paren_tag, new_parse_expr_list, close_paren_tag),
    )(input)
}

fn new_parse_expr_list(input: Tokens) -> IResult<Tokens, Expr> {
    map(many1(parse_expr), |list| {
        if list.len() == 1 {
            list.into_iter().next().expect("size just checked")
        } else {
            Expr::Group(list)
        }
    })(input)
}

fn parse_define_generator_statement(
    input: Tokens,
) -> IResult<Tokens, Statement> {
    map(
        delimited(generate_tag, parse_ident, opt(with_tag)),
        Statement::StartGeneratorDefinition,
    )(input)
}

fn parse_update_attrs(input: Tokens) -> IResult<Tokens, Vec<Statement>> {
    map(pair(parse_ident, parse_req_attrs), |(ident, attrs)| {
        let mut out = vec![Statement::MakeCurrentField(ident)];
        out.extend(attrs.into_iter());
        out
    })(input)
}

fn parse_define_group_where(input: Tokens) -> IResult<Tokens, Statement> {
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
            Token::ReservedIdent(x) => {
                Ok((rest, Literal::Reserved(Ident(x.to_string()))))
            }
            _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
        }
    }
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
tag_token!(comma_tag, Token::Comma);
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
