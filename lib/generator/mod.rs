use crate::evaluator::structure::{
    AsciiStringDef, ByteNumberDef, BytesDef, TextNumberDef,
};
use crate::{
    error::BajzelError,
    evaluator::{
        generator::GenDefinition,
        structure::{FieldDefinition, GroupDefinition},
        ProgramEnv,
    },
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::ops::ControlFlow;

pub trait Pixie {
    /// Determine whether a pixie is happy
    ///
    /// Happy pixie doesn't touch data of a genie.
    ////
    /// The happiness level depends on actual pixie - it represents how
    /// often is it prone to modify data.
    ///
    fn is_happy(&self) -> bool;
}

pub struct Gen {
    _pixies: Vec<Box<dyn Pixie>>,
}

impl Default for Gen {
    fn default() -> Gen {
        Gen::new(vec![])
    }
}

impl Gen {
    /// Create Generator from pixie collection
    ///
    pub fn new(_pixies: Vec<Box<dyn Pixie>>) -> Gen {
        Self { _pixies }
    }

    pub fn generate(&self, env: &ProgramEnv) -> Result<Vec<u8>, BajzelError> {
        let gen = env.get_generator()?;
        let mut bytes: Vec<u8> = Vec::with_capacity(gen.out_max as usize);
        let group = env.get_group(&gen.name)?;
        self.generate_group(env, gen, group, &mut bytes)?;

        Ok(bytes)
    }

    fn generate_group(
        &self,
        _env: &ProgramEnv,
        _gen: &GenDefinition,
        group: &GroupDefinition,
        bytes: &mut Vec<u8>,
    ) -> Result<(), BajzelError> {
        for field in group.fields_iter() {
            let cf = match &field.def {
                FieldDefinition::ConstString(x) => {
                    self.generate_const_string(x, bytes)
                }
                FieldDefinition::TextNumber(x) => {
                    self.generate_text_number(x, bytes)
                }
                FieldDefinition::AsciiString(x) => {
                    self.generate_ascii_string(x, bytes)
                }
                FieldDefinition::ByteNumber(x) => {
                    self.generate_byte_number(x, bytes)
                }
                FieldDefinition::Bytes(x) => self.generate_bytes(x, bytes),
            };
            if let ControlFlow::Break(_) = cf {
                break;
            }
        }
        Ok(())
    }

    fn generate_const_string(
        &self,
        x: &String,
        bytes: &mut Vec<u8>,
    ) -> ControlFlow<()> {
        let required_len = x.len();
        let available_len =
            std::cmp::min(required_len, bytes.capacity() - bytes.len());
        if available_len == 0 {
            return ControlFlow::Break(());
        }
        let to_write = &x.as_bytes()[0..available_len];
        bytes.extend_from_slice(to_write);
        if available_len < required_len {
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    }

    fn generate_ascii_string(
        &self,
        x: &AsciiStringDef,
        bytes: &mut Vec<u8>,
    ) -> ControlFlow<()> {
        let available_len = bytes.capacity() - bytes.len();
        if available_len == 0 {
            return ControlFlow::Break(());
        }
        let mut rng = thread_rng();

        let min_len = std::cmp::min(x.length_min, available_len);
        let max_len = std::cmp::min(x.length_max, available_len);
        let rng_len: usize = {
            if min_len == max_len {
                min_len
            } else {
                rng.gen_range(min_len..=max_len)
            }
        };

        let data: String = rng
            .sample_iter(&Alphanumeric)
            .take(rng_len)
            .map(char::from)
            .collect();

        bytes.extend_from_slice(data.as_bytes());
        ControlFlow::Continue(())
    }

    fn generate_bytes(
        &self,
        x: &BytesDef,
        bytes: &mut Vec<u8>,
    ) -> ControlFlow<()> {
        let available_len = bytes.capacity() - bytes.len();
        if available_len == 0 {
            return ControlFlow::Break(());
        }
        let mut rng = thread_rng();
        let min_len = std::cmp::min(x.length_min, available_len);
        let max_len = std::cmp::min(x.length_max, available_len);
        let rng_len: usize = {
            if min_len == max_len {
                min_len
            } else {
                rng.gen_range(min_len..=max_len)
            }
        };
        let data: Vec<_> = (0..rng_len).map(|_| rng.gen::<u8>()).collect();
        bytes.extend_from_slice(data.as_slice());
        ControlFlow::Continue(())
    }

    fn generate_byte_number(
        &self,
        _x: &ByteNumberDef,
        _bytes: &mut Vec<u8>,
    ) -> ControlFlow<()> {
        todo!()
    }

    fn generate_text_number(
        &self,
        _x: &TextNumberDef,
        bytes: &mut Vec<u8>,
    ) -> ControlFlow<()> {
        let mut rng = thread_rng();
        let value = "42";
        bytes.extend_from_slice(value.as_bytes());
        ControlFlow::Continue(())
    }
}
