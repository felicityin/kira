use std::fs::File;
use std::io::Result;

use crate::code::func::Slot;
use crate::code::builder::AsmBuilder;

/// A global/local value.
pub enum AsmValue<'i> {
    Global(&'i str),
    Local(Slot),
    Const(i32),
    Arg(usize),
    Void,
}

/// Returns the assembly value of the given value data.
macro_rules! asm_value {
    ($info:expr, $v:expr) => {
        AsmValue::from(cur_func!($info).slot_offset($v))
    };
}
pub(crate) use asm_value;

impl<'i> AsmValue<'i> {
     /// Returns `true` if the value is a pointer.
    pub fn is_ptr(&self) -> bool {
        matches!(self, Self::Local(slot) if slot.is_ptr)
    }

    /// Writes the assembly value to the given register.
    pub fn write_to(&self, f: &mut File, reg: &'static str) -> Result<()> {
        let mut builder = AsmBuilder::new(f, reg);
        match self {
            Self::Global(symbol) => {
                builder.la(reg, symbol)?;
                builder.lw(reg, reg, 0)
            }
            Self::Local(slot) => builder.lw(reg, "sp", slot.offset as i32),
            Self::Const(num) => builder.li(reg, *num),
            _ => unreachable!(),
        }
    }

    /// Writes the assembly value (argument) to the given register.
    pub fn write_arg_to(&self, f: &mut File, reg: &'static str, sp_offset: usize) -> Result<()> {
        let mut builder = AsmBuilder::new(f, reg);
        match self {
        Self::Arg(index) => {
            if *index < 8 {
                builder.mv(reg, &format!("a{}", *index))
            } else {
                builder.lw(reg, "sp", (sp_offset + (*index - 8) * 4) as i32)
            }
        }
        _ => unreachable!(),
        }
    }

    /// Reads the value of the given register to the assembly value.
    pub fn read_from(&self, f: &mut File, reg: &'static str, temp: &'static str) -> Result<()> {
        let mut builder = AsmBuilder::new(f, temp);
        match self {
            Self::Global(symbol) => {
                builder.la(temp, symbol)?;
                builder.sw(reg, temp, 0)
            }
            Self::Local(slot) => builder.sw(reg, "sp", slot.offset as i32),
            Self::Const(_) => unreachable!(),
            Self::Arg(index) => {
                if *index < 8 {
                    builder.mv(&format!("a{}", *index), reg)
                } else {
                    builder.sw(reg, "sp", ((*index - 8) * 4) as i32)
                }
            }
            Self::Void => Ok(()),
        }
  }
}

impl<'i> From<Option<Slot>> for AsmValue<'i> {
    fn from(v: Option<Slot>) -> Self {
        match v {
            Some(slot) => Self::Local(slot),
            None => Self::Void,
        }
    }
}
