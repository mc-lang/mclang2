
use std::collections::HashMap;

#[macro_use]
mod logger;
mod cli;
mod types;
mod lexer;
pub mod parser;
mod compiler;

fn main() {
    let cli_args = cli::CliArgs::parse_with_passthrough();
    logger::Logger::init(&cli_args).expect("Failed to init logger");

    let mut prog_map = HashMap::new();
    for file in &cli_args.input {
        let mut lexer = lexer::Lexer::new();

        info!("Lexing file {file:?}");
        if let Err(_) = lexer.lex(file.as_std_path()) {
            error!("Lexing failed, exiting");
            return;
        }

        // for t in &lexer.tokens {
        //     info!({loc => t.loc.clone()}, "{:?}", t.typ);
        // }
        // dbg!(&lexer.tokens);

        info!("Parsing file {file:?}");
        let prog = match parser::parse(&cli_args, &mut lexer.tokens) {
            Ok(r) => r,
            Err(_) => {
                error!("Parsing failed, exiting");
                return;
            }
        };

        prog_map.insert(file.as_std_path(), prog);

    }
    if let Err(_) = compiler::compile_program(&cli_args, prog_map) {
        error!("Failed to compile program, exiting");
    }

}
