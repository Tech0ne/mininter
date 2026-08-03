use mininter::prelude::*;

use clap::{CommandFactory, Parser, ValueEnum};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Language {
    Ms,
    Mcs,
}

#[derive(Parser, Debug)]
#[command(
    name = "mininter",
    author,
    version,
    about = "MiniScript/MicroScript language interpreter",
    long_about = None,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Source file to execute.
    #[arg(required_unless_present = "interactive")]
    pub file: Option<PathBuf>,

    /// Start the interactive REPL.
    #[arg(short, long, conflicts_with = "compile")]
    pub interactive: bool,

    /// Language / dialect. Autodetected from the file extension (.ms(c) / .mcs(c))
    #[arg(short = 'l', long, value_enum, default_value_t = Language::Ms)]
    pub lang: Language,

    /// Pre-compile your input .ms / .mcs file into a .msc / .mcsc (respectively)
    /// This is still interpreted, but speedup the execution, plus report any syntax errors
    #[arg(short, long, requires = "file")]
    pub compile: bool,
}

impl Cli {
    pub fn language(&self) -> Language {
        if let Some(path) = &self.file {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                return match ext {
                    "ms" | "msc" => Language::Ms,
                    "mcs" | "mcsc" => Language::Mcs,
                    _ => self.lang,
                };
            }
        }

        self.lang
    }
}

fn compile(path: impl AsRef<Path>) {
    // TODO: implement "semi-compilation"
    println!("Compiling file {}", path.as_ref().display());
}

// TODO: actually use the required context
#[derive(Debug, Default)]
struct Context(u8);

fn run(context: &mut Option<Context>, path: impl AsRef<Path>) {
    // TODO: implement running
    let ctx = context.get_or_insert(Context::default());
    ctx.0 += 1;

    println!(
        "Running from file {} with context {:?}",
        path.as_ref().display(),
        ctx
    );
}

fn repl(context: &mut Option<Context>) {
    // TODO: implement REPL
    let ctx = context.get_or_insert(Context::default());
    ctx.0 += 1;

    println!("Starting REPL from context {:?}", ctx);
}

fn main() {
    let start = Instant::now();
    let mut context = None;

    let cli = Cli::parse();

    match cli.file {
        None => {
            if !cli.interactive {
                eprintln!("Error: require at least one of --interactive / [file]\n");

                Cli::command().render_long_help();

                std::process::exit(1);
            }
            if cli.compile {
                eprintln!("Error: compile requires a file argument\n");

                Cli::command().render_long_help();

                std::process::exit(1);
            }
        }
        Some(file) => {
            if cli.compile && cli.interactive {
                eprintln!(
                    "Warning: interactive REPL will no have the context of the given file (compiled)\n"
                );
            }

            if cli.compile {
                compile(file);
            } else {
                run(&mut context, file);
            }
        }
    }

    if cli.interactive {
        repl(&mut context);
    }
    println!("Running took {:?}", start.elapsed());
}
