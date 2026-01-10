use std::env;
use std::io;
use std::process;

enum Token {
    Char(char),
    Start,
    End,
}

impl Token {
    fn is_char_predicate<T>(&self, pred: T) -> bool
    where
        T: Fn(char) -> bool,
    {
        match self {
            Self::Char(c) => pred(*c),
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
enum EvalResult {
    Success { progress: usize },
    FailNonterminal,
    FailTerminal,
}

impl EvalResult {
    fn is_success(&self) -> bool {
        match self {
            Self::Success { .. } => true,
            _ => false,
        }
    }
}

enum PatternBit {
    Anywhere(Box<State>),
    Char(char),
    PatternSeq(Vec<State>),
}

struct State {
    pattern: PatternBit,
}

impl State {
    fn eval(&self, s: &[Token]) -> EvalResult {
        match &self.pattern {
            PatternBit::Anywhere(pat) => {
                let state = &**pat;
                let mut s = s;

                loop {
                    match state.eval(s) {
                        EvalResult::FailNonterminal => {
                            if s.is_empty() {
                                return EvalResult::FailTerminal;
                            } else {
                                s = &s[1..];
                            }
                        }
                        terminal_result => return terminal_result,
                    }
                }
            }
            PatternBit::PatternSeq(seq) => {
                let mut s = s;
                let mut total_progress = 0;

                for pat in seq {
                    match pat.eval(s) {
                        EvalResult::Success { progress } => {
                            total_progress += progress;
                            s = &s[progress..];
                        }
                        EvalResult::FailTerminal => return EvalResult::FailNonterminal,
                        EvalResult::FailNonterminal => return EvalResult::FailNonterminal,
                    }
                }

                EvalResult::Success {
                    progress: total_progress,
                }
            }
            PatternBit::Char(c) => {
                if s.is_empty() {
                    EvalResult::FailTerminal
                } else if s[0].is_char_predicate(|ch| ch == *c) {
                    EvalResult::Success { progress: 1 }
                } else {
                    EvalResult::FailNonterminal
                }
            }
        }
    }
}

struct Regex {}

impl Regex {
    fn is_match(s: &str, state: &State) -> bool {
        let mut tokens = s.chars().map(|c| Token::Char(c)).collect::<Vec<_>>();
        tokens.insert(0, Token::Start);
        tokens.push(Token::End);

        state.eval(&tokens[..]).is_success()
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern.chars().count() == 1 {
        input_line.contains(pattern)
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    // TODO: Uncomment the code below to pass the first stage
    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}

#[cfg(test)]
mod test {
    use crate::{PatternBit, Regex, State};

    #[test]
    fn test_char() {
        let state = State {
            pattern: PatternBit::Anywhere(Box::new(State {
                pattern: PatternBit::Char('x'),
            })),
        };

        assert!(Regex::is_match(&"x", &state));
        assert!(Regex::is_match(&"aax", &state));
        assert!(Regex::is_match(&"xaa", &state));

        assert!(!Regex::is_match(&"aa", &state));
        assert!(!Regex::is_match(&"", &state));
    }

    #[test]
    fn test_seq() {
        let state = State {
            pattern: PatternBit::Anywhere(Box::new(State {
                pattern: PatternBit::PatternSeq(vec![
                    State {
                        pattern: PatternBit::Char('x'),
                    },
                    State {
                        pattern: PatternBit::Char('y'),
                    },
                    State {
                        pattern: PatternBit::Char('z'),
                    },
                ]),
            })),
        };

        assert!(Regex::is_match(&"xyz", &state));
        assert!(Regex::is_match(&"aaxyz", &state));
        assert!(Regex::is_match(&"xyzaa", &state));
    }
}
