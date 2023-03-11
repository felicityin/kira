use std::env::args;
use std::fs::read_to_string;
use std::io::Result;

use lalrpop_util::lalrpop_mod;

mod ast;

lalrpop_mod!(sysy);

// cargo run -- input/hello.c
fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let input = args.next().unwrap();

    let input = read_to_string(input)?;

    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
    println!("{:#?}", ast);

    Ok(())
}
