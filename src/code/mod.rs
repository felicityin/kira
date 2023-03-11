mod builder;
mod info;
mod func;
mod gen;
mod values;

use std::fs::File;
use std::io::Result;

use koopa::ir::{Program, Type};

use info::ProgramInfo;
use gen::GenerateToAsm;

/// Generates the given Koopa IR program to RISC-V assembly.
pub fn generate_asm(program: &Program, path: &str) -> Result<()> {
    Type::set_ptr_size(4);
    program.generate(&mut File::create(path)?, &mut ProgramInfo::new(program))
}
