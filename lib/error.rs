#[derive(Debug, PartialEq, Eq)]
pub enum BajzelError {
    Conversion(String),
    ProgramNotFinished,
    Syntax(String),
    Expr(String),
    NotConstructedProperly,
}
