mod lex;
mod err;
mod parse;
mod eval;

use std::{env, error, fs};
use std::collections::HashMap;
use crate::eval::{Stmt, PscObject};

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut args = env::args();
    let prog = fs::read_to_string(args.nth(1).unwrap())?;
    let tokens = lex::lex(prog.as_str())?;
    let stmts = parse::parse(tokens)?;

    let mut vars: HashMap<String, PscObject> = HashMap::new();

    for stmt in stmts {
        Stmt::eval(&stmt, &mut vars)?;
    }

    Ok(())
}
