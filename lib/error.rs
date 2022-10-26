#[derive(Debug)]
pub enum BajzelError {
    Conversion(String),
    ProgramNotFinished,
    Syntax(String),
    Expr(String),
}
