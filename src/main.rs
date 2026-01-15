use std::io;
use std::process;

use clap::Parser;
use log::info;

use crate::common::EXIT_CODE_NO_MATCH;
use crate::common::EXIT_CODE_SUCCESS;
use crate::common::str_to_tokens;
use crate::evaluator::Evaluator;

mod ast;
mod common;
mod cond;
mod evaluator;
mod parser;
mod reader;
mod token;
mod transition;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct ProgramArgs {
    #[arg(short = 'E')]
    pattern: String,

    #[arg(short = 'o', default_value = "false")]
    only_match: bool,
}

fn main() {
    // unsafe { std::env::set_var("RUST_LOG", "debug") };
    pretty_env_logger::init();

    info!("Peter Grep Starts");

    let args = ProgramArgs::parse();

    let mut input_line = String::new();
    let mut has_match = false;

    while let Ok(1..) = io::stdin().read_line(&mut input_line) {
        let source = input_line.trim_end();

        let ast_root = crate::parser::Parser::parse_regex_str(&args.pattern).unwrap();
        let evaluator = Evaluator::new(ast_root.generate());

        if evaluator.is_match(&str_to_tokens(source)[..]) {
            println!("{}", source);
            has_match = true;
        }

        input_line.clear();
    }

    if has_match {
        process::exit(EXIT_CODE_SUCCESS)
    } else {
        process::exit(EXIT_CODE_NO_MATCH)
    }
}
