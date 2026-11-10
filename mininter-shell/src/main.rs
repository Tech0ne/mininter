use std::fs;
use std::io;

use std::path::PathBuf;
use std::process::Command;

use clap::{Parser as CliParser, Subcommand};
use mininter::MiniError;
use mininter::Value;
use mininter::api;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use mininter::compiler::Compiler;
use mininter::lexer::Lexer;
use mininter::parser::Parser;
use mininter::runtime::Vm;
use mininter::stdlib;

#[derive(CliParser)]
#[command(name = "mininter")]
#[command(version, about = "A miniscript/microscript interpreter/compiler/repl")]
struct Cli {
    /// Disable standard library loading
    #[arg(long, global = true)]
    no_stdlib: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a source file
    Run {
        /// Source file to execute
        file: PathBuf,
    },

    // /// Compile a source file
    // Build {
    //     /// Source file to compile
    //     file: PathBuf,
    //
    //     /// Output file
    //     #[arg(short, long)]
    //     output: Option<PathBuf>,
    // },
    /// Start an interactive REPL
    Repl,
}

fn execute(vm: &mut Vm, code: impl AsRef<str>, mod_path: PathBuf) -> Result<Value, MiniError> {
    let tokens = Lexer::new(code.as_ref()).tokenize()?;
    let stmts = Parser::new(tokens).parse()?;
    let function = Compiler::compile_script_with_mod_path(&stmts, mod_path)?;

    vm.run(function)?;

    Ok(Value::Null)
}

#[api]
fn system(command: String) -> String {
    String::from_utf8_lossy(
        &Command::new("sh")
            .arg("-c")
            .arg(format!("{command} 2>&1"))
            .output()
            .expect("failed to run command")
            .stdout,
    )
    .into_owned()
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let mut vm = Vm::new();
    if !cli.no_stdlib {
        stdlib::register_stdlib(&mut vm);
        vm.register(system_api());
    }

    match cli.command {
        Some(Commands::Run { file }) => {
            let base_path = file
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();

            let source = fs::read_to_string(file)?;

            if let Value::Number(nb) = execute(&mut vm, source, base_path)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("run error: {e:?}")))?
            {
                std::process::exit(nb.floor() as i32);
            }
        }

        // Some(Commands::Build { file, output }) => {
        //     println!("Building {:?} to {:?}", file, output);
        //
        //     let source = fs::read_to_string(file)?;
        //
        //     let tokens = Lexer::new(&source)
        //         .tokenize()
        //         .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("lexing error: {e}")))?;
        //     let stmts = Parser::new(tokens)
        //         .parse()
        //         .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("parsing error: {e}")))?;
        //     let function = Compiler::compile_script(&stmts).map_err(|e| {
        //         io::Error::new(io::ErrorKind::Other, format!("compilation error: {e}"))
        //     })?;
        //
        //     vm.run(function)
        //         .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("runtime error: {e}")))?;
        // }
        Some(Commands::Repl) | None => {
            println!("Starting ms REPL...");

            let mut rl = DefaultEditor::new().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("failed to build repl editor: {e:?}"),
                )
            })?;

            println!("Welcome to the mininter interpreter!");
            println!("Press Ctrl+D to exit.");

            loop {
                match rl.readline("] ") {
                    Ok(line) => {
                        if line.trim().is_empty() {
                            continue;
                        }

                        rl.add_history_entry(&line).map_err(|e| {
                            io::Error::new(
                                io::ErrorKind::Other,
                                format!("failed to add history entry: {e:?}"),
                            )
                        })?;

                        if let Err(e) =
                            execute(&mut vm, line, std::path::Path::new(".").to_path_buf())
                        {
                            println!("cought error executing line: {e}");
                        }
                    }

                    Err(ReadlineError::Interrupted) => {
                        continue;
                    }

                    Err(ReadlineError::Eof) => {
                        println!("Goodbye!");
                        break;
                    }

                    Err(err) => {
                        eprintln!("REPL error: {err}");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
