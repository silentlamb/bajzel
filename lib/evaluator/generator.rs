use nom::AsBytes;

use super::syntax_err;
use crate::{
    error::BajzelError,
    parser::{Expr, Literal},
};

#[derive(Debug)]
pub struct GenDefinition {
    pub name: String,
    pub out_min: u32,
    pub out_max: u32,
    pub term: Vec<u8>,
}

impl GenDefinition {
    pub fn new(name: String) -> Self {
        Self {
            name,
            out_min: 0,
            out_max: 4096,
            term: Vec::new(),
        }
    }

    pub fn set_min_output_len(
        &mut self,
        expr: Expr,
    ) -> Result<(), BajzelError> {
        #[allow(clippy::collapsible_match)]
        match expr {
            Expr::LiteralExpr(literal) => match literal {
                Literal::IntegerLiteral(value) => {
                    // Single valuye means length is constant
                    self.out_min = value as u32;
                    Ok(())
                }
                x => syntax_err("OUT_MIN: expected an integer"),
            },
            _ => syntax_err("OUT_MIN: expected an integer"),
        }
    }

    pub fn set_max_output_len(
        &mut self,
        expr: Expr,
    ) -> Result<(), BajzelError> {
        #[allow(clippy::collapsible_match)]
        match expr {
            Expr::LiteralExpr(literal) => match literal {
                Literal::IntegerLiteral(value) => {
                    // Single valuye means length is constant
                    self.out_max = value as u32;
                    Ok(())
                }
                x => syntax_err("OUT_MAX: expected an integer"),
            },
            _ => syntax_err("OUT_MAX: expected an integer"),
        }
    }

    pub fn set_term(&mut self, expr: Expr) -> Result<(), BajzelError> {
        match expr {
            Expr::LiteralExpr(literal) => match literal {
                Literal::IntegerLiteral(x) => {
                    if (0..=255).contains(&x) {
                        self.term.push(x as u8);
                        Ok(())
                    } else {
                        syntax_err("TERM: Decimal ASCII value expected")
                    }
                }
                Literal::StringLiteral(x) => {
                    self.term.extend(x.as_bytes());
                    Ok(())
                }
                Literal::BytesLiteral(x) => {
                    self.term.extend(x.as_bytes());
                    Ok(())
                }
                Literal::Reserved(x) => match x.as_str() {
                    "LF" => {
                        self.term.push(0x0A);
                        Ok(())
                    }
                    "NULL" => {
                        self.term.push(0x00);
                        Ok(())
                    }
                    "RF" => {
                        self.term.push(0x0D);
                        Ok(())
                    }
                    _ => syntax_err("TERM: unsupported reserved literal"),
                },
            },
            _ => syntax_err("TERM: expected a single literal"),
        }
    }
}
