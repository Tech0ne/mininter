use std::{env, fs, process};

use mininter::prelude::miniscript::{Lexer, TokenKind};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {}, <miniscript code>", args[0]);
        process::exit(1);
    }

    let fname = &args[1];

    let code = match fs::read_to_string(fname) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("{}: {}: {}", args[0], fname, e);
            process::exit(1);
        }
    };

    println!("=================");
    println!("Lexing the following code:");
    println!("-----------------");
    println!("{}", code);
    println!("=================");

    let lexer = Lexer::new(&code);

    for token in lexer {
        if let TokenKind::NewLine = token.kind {
            println!();
        } else {
            println!("{}", token);
        }
    }
}
