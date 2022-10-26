use crate::{
    error::BajzelError,
    parser::{Expr, Literal},
};

use super::{eval_expr_to_i64, syntax_err};

#[derive(Debug, Default)]
pub struct GroupDefinition {
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub def: FieldDefinition,
    pub alias: Option<String>,
}

#[derive(Debug)]
pub enum FieldDefinition {
    /// Non-random string, such as "BM"
    ///
    ConstString(String),

    /// Non-random bytes, such as `0x00 0x42 0x48`
    ///
    // ConstBytes(Vec<u8>),

    /// Random number represented as a text.
    ///
    /// For example, number 42069 will be represented as a string "42069"
    TextNumber(TextNumberDef),

    /// Random number represented in byte form
    ///
    /// For example, number 305_419_896  can be represented as bytes:
    /// - big endian:     `0x12 0x34 0x56 0x78`
    /// - little endian:  `0x78 0x56 0x34 0x12`
    // ByteNumber(ByteNumberDef),

    /// Random string that can be displayed (only displayable characters)
    ///
    AsciiString(AsciiStringDef),

    ByteNumber(ByteNumberDef),
}

#[derive(Debug)]
pub struct TextNumberDef {
    format: NumberFormat,
    min_value: i128,
    max_value: i128,
    display: Option<NumberDisplayFormat>,
}

#[derive(Debug)]
pub struct ByteNumberDef {
    format: NumberFormat,
    endianess: ByteOrder,
    min_value: i128,
    max_value: i128,
}

#[derive(Debug)]
pub struct AsciiStringDef {
    length_min: usize,
    length_max: usize,
}

#[derive(Debug)]
pub enum NumberFormat {
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
}

#[derive(Debug)]
pub enum NumberDisplayFormat {
    Binary,
    Octal,
    Decimal,
    Hex,
}

#[derive(Debug)]
pub enum ByteOrder {
    /// Big-end is first (the first byte is biggest)
    BigEndian,

    /// Little-end is first (the first byte is smallest)
    LittleEndian,
}

impl ByteNumberDef {
    pub fn new(format: NumberFormat, endianess: ByteOrder) -> Self {
        let min = format.min_as_i128();
        let max = format.max_as_i128();
        Self {
            format,
            endianess,
            min_value: min,
            max_value: max,
        }
    }

    pub fn from_str(format: &str) -> Result<Self, BajzelError> {
        match format {
            "le_u16" => {
                Ok(Self::new(NumberFormat::Uint16, ByteOrder::LittleEndian))
            }
            "le_u32" => {
                Ok(Self::new(NumberFormat::Uint32, ByteOrder::LittleEndian))
            }
            "le_u64" => {
                Ok(Self::new(NumberFormat::Uint64, ByteOrder::LittleEndian))
            }
            "le_i16" => {
                Ok(Self::new(NumberFormat::Int16, ByteOrder::LittleEndian))
            }
            "le_i32" => {
                Ok(Self::new(NumberFormat::Int32, ByteOrder::LittleEndian))
            }
            "le_i64" => {
                Ok(Self::new(NumberFormat::Int64, ByteOrder::LittleEndian))
            }
            "be_u16" => {
                Ok(Self::new(NumberFormat::Uint16, ByteOrder::BigEndian))
            }
            "be_u32" => {
                Ok(Self::new(NumberFormat::Uint32, ByteOrder::BigEndian))
            }
            "be_u64" => {
                Ok(Self::new(NumberFormat::Uint64, ByteOrder::BigEndian))
            }
            "be_i16" => {
                Ok(Self::new(NumberFormat::Int16, ByteOrder::BigEndian))
            }
            "be_i32" => {
                Ok(Self::new(NumberFormat::Int32, ByteOrder::BigEndian))
            }
            "be_i64" => {
                Ok(Self::new(NumberFormat::Int64, ByteOrder::BigEndian))
            }
            _ => Err(BajzelError::Conversion(format.to_owned())),
        }
    }

    pub fn update(
        &mut self,
        attr_name: &str,
        expr: Expr,
    ) -> Result<(), BajzelError> {
        unimplemented!("ByteNumberDef: update {:?} -> {:?}", attr_name, expr);
    }
}

impl TextNumberDef {
    pub fn new(format: NumberFormat) -> Self {
        let min = format.min_as_i128();
        let max = format.max_as_i128();
        Self {
            format,
            min_value: min,
            max_value: max,
            display: None,
        }
    }

    pub fn from_str(format: &str) -> Result<Self, BajzelError> {
        match format {
            "i8" => Ok(TextNumberDef::new(NumberFormat::Int8)),
            "i16" => Ok(TextNumberDef::new(NumberFormat::Int16)),
            "i32" => Ok(TextNumberDef::new(NumberFormat::Int32)),
            "i64" => Ok(TextNumberDef::new(NumberFormat::Int64)),
            "u8" => Ok(TextNumberDef::new(NumberFormat::Uint8)),
            "u16" => Ok(TextNumberDef::new(NumberFormat::Uint16)),
            "u32" => Ok(TextNumberDef::new(NumberFormat::Uint32)),
            "u64" => Ok(TextNumberDef::new(NumberFormat::Uint64)),
            _ => Err(BajzelError::Conversion(format.to_owned())),
        }
    }

    pub fn update(
        &mut self,
        attr_name: &str,
        expr: Expr,
    ) -> Result<(), BajzelError> {
        match attr_name {
            "RANGE" => self.set_range(expr),
            _ => syntax_err("unsupported text number attribute"),
        }
    }

    fn set_range(&mut self, expr: Expr) -> Result<(), BajzelError> {
        match expr {
            Expr::LiteralExpr(_) => {
                syntax_err("RANGE(min max): expects exactly 2 values")
            }
            Expr::Group(v) => {
                if v.len() == 2 {
                    let min = eval_expr_to_i64(&v[0])?;
                    let max = eval_expr_to_i64(&v[1])?;
                    self.min_value = min as i128;
                    self.max_value = max as i128;
                    Ok(())
                } else {
                    syntax_err("RANGE(min max) expects exactly 2 values")
                }
            }
        }
    }
}

impl AsciiStringDef {
    pub fn new() -> Self {
        Self {
            length_min: 0,
            length_max: 4096,
        }
    }

    pub fn update(
        &mut self,
        attr_name: &str,
        expr: Expr,
    ) -> Result<(), BajzelError> {
        match attr_name {
            "LEN" => self.set_len(expr),
            _ => syntax_err("unsupported string attribute name"),
        }
    }

    /// Sets min and max number of characters in the output
    ///
    /// Syntax:
    ///     LEN(value)      - sets exactly `value` number of characters
    ///     LEN(min max)    - sets length to be in range from `min` to `max`
    ///
    fn set_len(&mut self, expr: Expr) -> Result<(), BajzelError> {
        match expr {
            Expr::LiteralExpr(literal) => match literal {
                Literal::IntegerLiteral(value) => {
                    if value < 0 {
                        return syntax_err(
                            "LEN(value): value < 0 is not allowed",
                        );
                    }
                    // Single valuye means length is constant
                    self.length_min = value as usize;
                    self.length_max = value as usize;
                    Ok(())
                }
                x => syntax_err(
                    "string literals: unsupported literal expression",
                ),
            },
            Expr::Group(v) => {
                if v.len() == 2 {
                    let min = eval_expr_to_i64(&v[0])?;
                    let max = eval_expr_to_i64(&v[1])?;
                    if min > max {
                        return syntax_err(
                            "LEN(min max): min > max is not allowed",
                        );
                    }
                    if min < 0 {
                        return syntax_err(
                            "LEN(min max): min < 0 is not allowed",
                        );
                    }
                    self.length_min = min as usize;
                    self.length_max = max as usize;
                    Ok(())
                } else {
                    syntax_err("string literals: ")
                }
            }
        }
    }
}

impl Default for AsciiStringDef {
    fn default() -> Self {
        Self::new()
    }
}

impl NumberFormat {
    pub fn min_as_i128(&self) -> i128 {
        match self {
            NumberFormat::Int8 => i8::MIN as i128,
            NumberFormat::Int16 => i16::MIN as i128,
            NumberFormat::Int32 => i32::MIN as i128,
            NumberFormat::Int64 => i64::MIN as i128,
            NumberFormat::Uint8 => u8::MIN as i128,
            NumberFormat::Uint16 => u16::MIN as i128,
            NumberFormat::Uint32 => u32::MIN as i128,
            NumberFormat::Uint64 => u64::MIN as i128,
        }
    }

    pub fn max_as_i128(&self) -> i128 {
        match self {
            NumberFormat::Int8 => i8::MAX as i128,
            NumberFormat::Int16 => i16::MAX as i128,
            NumberFormat::Int32 => i32::MAX as i128,
            NumberFormat::Int64 => i64::MAX as i128,
            NumberFormat::Uint8 => u8::MAX as i128,
            NumberFormat::Uint16 => u16::MAX as i128,
            NumberFormat::Uint32 => u32::MAX as i128,
            NumberFormat::Uint64 => u64::MAX as i128,
        }
    }
}
