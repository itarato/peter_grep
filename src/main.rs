extern crate isatty;

use std::collections::VecDeque;
use std::fs::read_to_string;
use std::io;
use std::process;

use clap::{Parser, ValueEnum};
use log::error;
use log::info;

use crate::common::EXIT_CODE_NO_MATCH;
use crate::common::EXIT_CODE_SUCCESS;
use crate::common::merge_overlapping_match_ranges;
use crate::common::range_end_adjust;
use crate::common::range_start_adjust;
use crate::common::str_to_tokens;
use crate::evaluator::EvalMatchResult;
use crate::evaluator::Evaluator;

use isatty::stdout_isatty;

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
    filepath: Option<Vec<String>>,

    #[arg(short = 'E')]
    pattern: String,

    #[arg(short = 'o', default_value = "false")]
    only_match: bool,

    #[arg(long, default_value = "never")]
    color: ColorArg,

    #[arg(short, default_value = "false")]
    recursive: bool,
}

impl ProgramArgs {
    fn is_color(&self) -> bool {
        match self.color {
            ColorArg::Always => true,
            ColorArg::Never => false,
            ColorArg::Auto => stdout_isatty(),
        }
    }

    fn input_iterator(&self) -> InputIterator {
        if self.recursive {
            InputIterator::new_from_directories(
                &self
                    .filepath
                    .as_ref()
                    .expect("missing files in recursive mode"),
            )
        } else if let Some(files) = self.filepath.as_ref() {
            InputIterator::new_from_files(files.clone())
        } else {
            InputIterator::new_from_stdin()
        }
    }
}

enum InputIterator {
    Stdin,
    Files {
        file_names: VecDeque<String>,
        active_file_lines: VecDeque<String>,
        current_file_path: Option<String>,
        should_return_current_file_path: bool,
    },
}

impl InputIterator {
    fn new_from_stdin() -> Self {
        Self::Stdin
    }

    fn new_from_files(file_names: Vec<String>) -> Self {
        Self::Files {
            file_names: file_names.clone().into(),
            active_file_lines: VecDeque::new(),
            current_file_path: None,
            should_return_current_file_path: file_names.len() > 1,
        }
    }

    fn new_from_directories(dir_names: &Vec<String>) -> Self {
        let mut file_names = VecDeque::new();
        let mut dir_stack = dir_names.clone();

        while let Some(dir) = dir_stack.pop() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            file_names.push_back(entry.path().to_string_lossy().to_string());
                        } else if metadata.is_dir() {
                            dir_stack.push(entry.path().to_string_lossy().to_string());
                        }
                    } else {
                        error!("Error: cannot read metadata for dir entry: {:?}", entry);
                    }
                }
            } else {
                error!("Error: cannot read dir: {}", dir);
            }
        }

        let should_return_current_file_path = file_names.len() > 1;

        Self::Files {
            file_names,
            active_file_lines: VecDeque::new(),
            current_file_path: None,
            should_return_current_file_path,
        }
    }
}

impl Iterator for InputIterator {
    type Item = (String, Option<String>);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Stdin => {
                let mut input_line = String::new();
                match io::stdin().read_line(&mut input_line) {
                    Ok(0) => None,
                    Ok(_) => Some((input_line.trim_end().to_string(), None)),
                    _ => {
                        error!("Failed reading STDIN");
                        None
                    }
                }
            }
            Self::Files {
                file_names,
                active_file_lines,
                current_file_path,
                should_return_current_file_path,
            } => {
                if active_file_lines.is_empty() {
                    loop {
                        if file_names.is_empty() {
                            return None;
                        }

                        let file_name = file_names.pop_front().unwrap();
                        *current_file_path = Some(file_name.clone());
                        let content = read_to_string(file_name).unwrap();
                        *active_file_lines = content
                            .lines()
                            .map(|l| l.to_string())
                            .collect::<VecDeque<_>>();

                        if !active_file_lines.is_empty() {
                            break;
                        }
                    }
                }

                match active_file_lines.pop_front() {
                    Some(line) => Some((
                        line,
                        if *should_return_current_file_path {
                            current_file_path.clone()
                        } else {
                            None
                        },
                    )),
                    None => None,
                }
            }
        }
    }
}

fn main() {
    // unsafe { std::env::set_var("RUST_LOG", "debug") };
    pretty_env_logger::init();

    info!("Peter Grep Starts");

    let args = ProgramArgs::parse();
    let mut has_match = false;
    let input_it = args.input_iterator();

    for (line, source) in input_it {
        let ast_root = crate::parser::Parser::parse_regex_str(&args.pattern).unwrap();
        let evaluator = Evaluator::new(ast_root.generate());

        match evaluator.is_match(&str_to_tokens(&line)[..]) {
            EvalMatchResult::Match { matches } => {
                if let Some(source) = source {
                    print!("{}:", source);
                }

                if args.only_match {
                    for (start, end) in matches {
                        let start = range_start_adjust(start);
                        let end = range_end_adjust(end, line.len());
                        println!("{}", &line[start..end]);
                    }
                } else {
                    if args.is_color() {
                        let merged_ranges = merge_overlapping_match_ranges(&matches);

                        let mut merge_iter = merged_ranges.iter();
                        let mut previous_range = merge_iter.next().unwrap();

                        print!("{}", &line[..range_start_adjust(previous_range.0)]);
                        print!(
                            "\x1B[01;31m{}\x1B[m",
                            &line[range_start_adjust(previous_range.0)
                                ..range_end_adjust(previous_range.1, line.len())]
                        );

                        for range in merge_iter {
                            print!(
                                "{}",
                                &line[range_end_adjust(previous_range.1, line.len())
                                    ..range_start_adjust(range.0)]
                            );
                            print!(
                                "\x1B[01;31m{}\x1B[m",
                                &line[range_start_adjust(range.0)
                                    ..range_end_adjust(range.1, line.len())]
                            );

                            previous_range = range;
                        }

                        print!(
                            "{}\n",
                            &line[range_end_adjust(previous_range.1, line.len())..]
                        );
                    } else {
                        println!("{}", line);
                    }
                }
                has_match = true;
            }
            EvalMatchResult::NoMatch => {}
        }
    }

    if has_match {
        process::exit(EXIT_CODE_SUCCESS)
    } else {
        process::exit(EXIT_CODE_NO_MATCH)
    }
}
