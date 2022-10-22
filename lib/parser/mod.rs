use crate::lexer::Tokens;
use std::ops::Deref;

mod funcs;

///
///
pub fn parse_tokens(tokens: Tokens) -> Result<Program, String> {
    funcs::parse_program(tokens)
        .map(|(_tokens, program)| program)
        .map_err(|e| match e {
            nom::Err::Incomplete(e) => {
                format!("[-] Incomplete: {:?}", e)
            }
            nom::Err::Error(e) => {
                format!(
                    "[-] Error ({:?}) at token: {:?}, next: {:?}",
                    e.code,
                    e.input.tokens[0],
                    &e.input.tokens[1..]
                )
            }
            nom::Err::Failure(e) => {
                format!("[!] Failure: {:?}", e)
            }
        })
}

#[derive(Debug, PartialEq)]
pub struct Program(Vec<Statement>);

impl Deref for Program {
    type Target = Vec<Statement>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Statement>> for Program {
    fn from(src: Vec<Statement>) -> Self {
        Program(src)
    }
}

#[derive(Debug, PartialEq)]
pub struct Ident(String);

impl Deref for Ident {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for Ident {
    fn from(ident: &str) -> Self {
        Ident(ident.to_owned())
    }
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    /// Create a new definition group and set it as active
    ///
    /// Example:
    ///
    /// ```fuzl
    /// DEFINE cmd
    /// ```
    ///
    StartGroupDefinition(Ident),

    /// Starts a generator definition
    ///
    /// Example:
    ///
    /// ```fuzl
    /// GENERATE structure WITH
    ///     TERM = null
    /// ```
    ///
    StartGeneratorDefinition(Ident),

    /// Define new variable field in an active group and make it active
    ///
    /// Example:
    ///
    /// ```fuzl
    /// DEFINE command  # To create a group and make it active
    ///     u32 AS cmd
    /// ```
    ///
    DefineVariableField(String, Option<Ident>),

    /// Define new constant field in an active group and make it active
    ///
    /// Example
    ///
    /// ```fuzl
    /// DEFINE command
    ///     "BM" AS magic
    /// ```
    ///
    DefineConstField(Literal, Option<Ident>),

    /// Make a field of a current group as active
    ///
    /// This is called in WHERE section of a `DEFINE` statement
    ///
    /// Example:
    ///
    /// ```fuzl
    /// DEFINE command
    ///     u32 AS cmd
    /// WHERE
    ///     param -> RANGE(0 10)    # "param ->" part
    /// ```
    ///
    MakeCurrentField(Ident),

    /// Run the generation
    ///
    /// Returned at the end of a token stream
    Run,

    /// Start section with field parameter updates
    ///
    /// Example:
    ///
    /// ```fuzl
    /// DEFINE command
    ///     [...]
    /// WHERE
    ///     [...]
    /// ```
    StartFieldsSection,

    /// Call a function in a form of `NAME(expr)` to update attribute
    /// of a current field.
    ///
    /// Each field type has its own set of functions
    ///
    /// Example: See above
    ///
    UpdateField(Ident, Expr),

    /// Assign value of an expression to a parameter
    ///
    /// This is called in `WITH` section of a `GENERATE` block
    ///
    /// Example:
    ///
    /// ```fuzl
    /// DEFINE command
    ///     [...]
    ///
    /// GENERATE command WITH
    ///     gen_param_1 = 42
    ///     gen_param_2 = 69
    /// ```
    ///
    UpdateParam(Ident, Expr),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    /// Single literal (string, integer, etc)
    ///
    /// Examples:
    ///
    /// - `42`
    /// - `"Hello, world!"`
    /// - `null`
    ///
    LiteralExpr(Literal),

    Group(Vec<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    IntegerLiteral(i64),
    StringLiteral(String),
    BytesLiteral(Vec<u8>),
    Reserved(Ident),
}
