use std::io;
use std::process;

use clap::{Parser, ValueEnum};
use log::info;

use crate::common::EXIT_CODE_NO_MATCH;
use crate::common::EXIT_CODE_SUCCESS;
use crate::common::merge_overlapping_match_ranges;
use crate::common::range_end_adjust;
use crate::common::range_start_adjust;
use crate::common::str_to_tokens;
use crate::evaluator::EvalMatchResult;
use crate::evaluator::Evaluator;

mod ast;
mod common;
mod cond;
mod evaluator;
mod parser;
mod reader;
mod token;
mod transition;

#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum ColorArg {
    Always,
    Auto,
    Never,
}

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct ProgramArgs {
    #[arg(short = 'E')]
    pattern: String,

    #[arg(short = 'o', default_value = "false")]
    only_match: bool,

    #[arg(long, default_value = "never")]
    color: ColorArg,
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

        match evaluator.is_match(&str_to_tokens(source)[..]) {
            EvalMatchResult::Match { matches } => {
                if args.only_match {
                    for (start, end) in matches {
                        let start = range_start_adjust(start); // Compensate for <start> token.
                        let end = range_end_adjust(end, source.len()); // Compensate for <start> and <end> tokens.
                        println!("{}", &source[start..end]);
                    }
                } else {
                    if args.color == ColorArg::Always {
                        let merged_ranges = merge_overlapping_match_ranges(&matches);

                        let mut merge_iter = merged_ranges.iter();
                        let mut previous_range = merge_iter.next().unwrap();

                        print!("{}", &source[..range_start_adjust(previous_range.0)]);
                        print!(
                            "\x1B[01;31m{}\x1B[m",
                            &source[range_start_adjust(previous_range.0)
                                ..range_end_adjust(previous_range.1, source.len())]
                        );

                        for range in merge_iter {
                            print!(
                                "{}",
                                &source[range_end_adjust(previous_range.1, source.len())
                                    ..range_start_adjust(range.0)]
                            );
                            print!(
                                "\x1B[01;31m{}\x1B[m",
                                &source[range_start_adjust(range.0)
                                    ..range_end_adjust(range.1, source.len())]
                            );

                            previous_range = range;
                        }

                        print!(
                            "{}\n",
                            &source[range_end_adjust(previous_range.1, source.len())..]
                        );
                    } else {
                        println!("{}", source);
                    }
                }
                has_match = true;
            }
            EvalMatchResult::NoMatch => {}
        }

        input_line.clear();
    }

    if has_match {
        process::exit(EXIT_CODE_SUCCESS)
    } else {
        process::exit(EXIT_CODE_NO_MATCH)
    }
}
