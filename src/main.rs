#![allow(clippy::wildcard_imports)]
#![allow(clippy::too_many_lines)]
mod definitions;
mod util;
mod compile;
mod parser;
mod lexer;
mod preprocessor;
mod typechecker;
mod precompiler;
mod config;
mod errors;
pub mod test;
use config::*;
use std::{fs, collections::HashMap};

use clap::Parser;
use anyhow::{Result, bail};

#[derive(Parser, Debug, Clone)]
#[command(author=env!("CARGO_PKG_AUTHORS"), version=env!("CARGO_PKG_VERSION"), about=env!("CARGO_PKG_DESCRIPTION"), long_about=env!("CARGO_PKG_DESCRIPTION"))]
pub struct Args {
    /// Input source file
    in_file: String,

    /// Output compiled file
    #[arg(long, short, default_value_t=String::from(DEFAULT_OUT_FILE))]
    out_file: String,

    /// Interpert
    #[arg(long, short='s')]
    interpret: bool,

    /// Run the compiled executable
    #[arg(long, short)]
    run: bool,

    /// Dont print any output exept the actual running codes output
    #[arg(long, short)]
    quiet: bool,
    
    /// Add an include directory [default: ["./include", "~/.mclang/include"]]
    #[arg(long, short='I')]
    include: Vec<String>,

    /// Unsafe mode, disables typechecking
    #[arg(long="unsafe", default_value_t = false)]
    unsaf: bool,
    
    /// Optimisation level, available levels: 'D': debug, '0': No optimisations
    #[arg(long, short='O', default_value_t=String::from("0"))]
    optimisation: String,

    // disables the main function
    #[arg(long="lib")]
    lib_mode: bool
    //#[arg(long, short='F')]
    //features: Vec<String>,

}

impl Args {
    /// Get optimisation level
    /// 0 => no optimisations
    /// 1 => slight optimisations, mostly size ones
    /// # Errors
    /// 
    /// Throws when the opt level is not known
    pub fn get_opt_level(&self) -> Result<usize>{
        match self.optimisation.as_str() {
            "D" | "d" => Ok(0),
            "0" | "" => Ok(1),
            o => {
                error!("Unknown optimisation level {o}");
                bail!("")
            }
        }
    }
}

fn main() -> Result<()>{


    let args = Args::parse();

    let Ok(code) = fs::read_to_string(&args.in_file) else {
        error!("Failed to read file {}, exiting!", &args.in_file);
        return Ok(());
    };
    
    let tokens = lexer::lex(&code, args.in_file.as_str(), &args);

    
    let mut parser = parser::Parser::new(tokens, &args, None);
    let program = match parser.parse(){
        Ok(t) => t,
        Err(e) => {
            error!("Parsing failed, exiting!");
            if crate::DEV_MODE {
                return Err(e)
            }
            return Ok(());
        }
    };

    match typechecker::typecheck(program.ops.clone(), &args, None, HashMap::new(), HashMap::new()) {
        Ok(_) => (),
        Err(e) => {
            error!("Typechecking failed, exiting!");
            if crate::DEV_MODE {
                return Err(e);
            }
            return Ok(());
        }
    };

    let c =match compile::linux_x86_64::compile(&program, &args) {
        Ok(c) => c,
        Err(e) => {
            error!("Compilation failed, exiting!");
            println!("{e}");
            1
        }
    };

    std::process::exit(c);
}
