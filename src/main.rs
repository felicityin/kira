mod ast;
mod code;
mod ir;

use std::env::args;
use std::fmt;
use std::fs::read_to_string;
use std::io;
use std::process::exit;

use koopa::back::KoopaGenerator;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(sysy);

// cargo run -- -koopa input/hello.c -o output/hello.koopa
// cargo run -- -riscv input/hello.c -o output/hello.asm
fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        exit(-1);
    }
}

fn try_main() -> Result<(), Error> {
    // parse command line arguments
    let CommandLineArgs {
        mode,
        input,
        output,
    } = CommandLineArgs::parse()?;

    // parse input file
    let input = read_to_string(input).map_err(Error::File)?;
    let program_ast = sysy::CompUnitParser::new()
        .parse(&input)
        .map_err(|_| Error::Parse)?;
    println!("AST:\n{:#?}", program_ast);

    // generate IR
    let program_ir = ir::generate_program(&program_ast).map_err(Error::Generate)?;
    if matches!(mode, Mode::Koopa) {
        return KoopaGenerator::from_path(output)
          .map_err(Error::File)?
          .generate_on(&program_ir)
          .map_err(Error::Io);
    }

    // generate RISC-V assembly
    code::generate_asm(&program_ir, &output).map_err(Error::Io)
}

/// Error returned by `main` procedure.
enum Error {
    InvalidArgs,
    Parse,
    Generate(ir::Error),
    File(io::Error),
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidArgs => write!(
                f,
                r#"Usage: kira MODE INPUT -o OUTPUT

    Options:
        MODE:   can be `-koopa`, `-riscv` or `-perf`
        INPUT:  the input SysY source file
        OUTPUT: the output file"#
            ),
            Self::Parse => write!(f, "error occurred while parsing"),
            Self::Generate(err) => write!(f, "{}", err),
            Self::File(err) => write!(f, "invalid input SysY file: {}", err),
            Self::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

/// Command line arguments.
struct CommandLineArgs {
    mode: Mode,
    input: String,
    output: String,
}

impl CommandLineArgs {
    /// Parses the command line arguments, returns `Error` if error occurred.
    fn parse() -> Result<Self, Error> {
        let mut args = args();
        args.next();
        match (args.next(), args.next(), args.next(), args.next()) {
            (Some(m), Some(input), Some(o), Some(output)) if o == "-o" => {
            let mode = match m.as_str() {
                "-koopa" => Mode::Koopa,
                "-riscv" => Mode::Riscv,
                _ => return Err(Error::InvalidArgs),
            };
            Ok(Self {
                mode,
                input,
                output,
            })
            }
            _ => Err(Error::InvalidArgs),
        }
    }
}

/// Compile mode.
enum Mode {
    /// Compile SysY to Koopa IR.
    Koopa,
    /// Compile SysY to RISC-V assembly.
    Riscv,
}
