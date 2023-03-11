mod func;
mod gen;
mod scopes;

use std::fmt;

use crate::ast::CompUnit;

use gen::GenerateProgram;
use koopa::ir::Program;
use scopes::Scopes;

pub type Result<T> = std::result::Result<T, Error>;

/// Generates Koopa IR program for the given compile unit (ASTs).
pub fn generate_program(comp_unit: &CompUnit) -> Result<Program> {
    let mut program = Program::new();
    comp_unit.generate(&mut program, &mut Scopes::new())?;
    Ok(program)
}

/// Error returned by IR generator.
pub enum Error {
}

impl fmt::Display for Error {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}
