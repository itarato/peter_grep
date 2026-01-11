use std::{collections::HashSet, fs::File, io::Write};

enum Token {
    Char(char),
    Start,
    End,
}

enum MatchResult {
    MatchAndConsume,
    MatchNoConsume,
    NoMatch,
}

#[derive(Debug)]
enum Cond {
    Char(char),
    AnyChar,
    CharGroup(HashSet<char>),
    Start,
    End,
    None,
}

impl Cond {
    fn to_label(&self) -> String {
        match self {
            Self::Char(c) => format!("C({})", c),
            Self::CharGroup(chars) => {
                format!(
                    "{}",
                    chars
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                )
            }
            Self::None => "-".to_string(),
            Self::Start => "^".to_string(),
            Self::End => "$".to_string(),
            Self::AnyChar => ".".to_string(),
        }
    }

    fn is_match(&self, c: Option<&Token>) -> MatchResult {
        match self {
            Self::Char(expected) => match c {
                Some(Token::Char(c)) => {
                    if c == expected {
                        MatchResult::MatchAndConsume
                    } else {
                        MatchResult::NoMatch
                    }
                }
                _ => MatchResult::NoMatch,
            },
            Self::None => MatchResult::MatchNoConsume,
            Self::CharGroup(chars) => match c {
                Some(Token::Char(c)) => {
                    if chars.contains(c) {
                        MatchResult::MatchAndConsume
                    } else {
                        MatchResult::NoMatch
                    }
                }
                _ => MatchResult::NoMatch,
            },
            Self::Start => match c {
                Some(Token::Start) => MatchResult::MatchAndConsume,
                _ => MatchResult::NoMatch,
            },
            Self::End => match c {
                Some(Token::End) => MatchResult::MatchAndConsume,
                _ => MatchResult::NoMatch,
            },
            Self::AnyChar => match c {
                Some(Token::Char(_)) => MatchResult::MatchAndConsume,
                _ => MatchResult::NoMatch,
            },
        }
    }
}

#[derive(Debug)]
struct Transition {
    from_state: u64,
    to_state: u64,
    cond: Cond,
    max_use: Option<usize>,
}

impl Transition {
    fn to_label(&self) -> String {
        match self.max_use {
            Some(v) => format!("{} (max {})", self.cond.to_label(), v),
            None => self.cond.to_label(),
        }
    }
}

#[derive(Debug)]
enum AstNode {
    Root(Box<AstNode>),
    Char(char),
    Seq(Vec<AstNode>),
    Alt(Vec<AstNode>),
    Repeat {
        min: Option<usize>,
        max: Option<usize>,
        node: Box<AstNode>,
    },
}

impl AstNode {
    fn generate(&self, id_provider: &mut u64, start_state: u64, end_state: u64) -> Vec<Transition> {
        match self {
            Self::Root(inner) => inner.generate(id_provider, start_state, end_state),
            Self::Char(c) => vec![Transition {
                from_state: start_state,
                to_state: end_state,
                cond: Cond::Char(*c),
                max_use: None,
            }],
            Self::Seq(seq) => {
                if seq.is_empty() {
                    vec![Transition {
                        from_state: start_state,
                        to_state: end_state,
                        cond: Cond::None,
                        max_use: None,
                    }]
                } else {
                    let mut transitions = vec![];
                    let mut from_id = start_state;

                    for i in 0..seq.len() {
                        let to_id = if i + 1 == seq.len() {
                            end_state
                        } else {
                            *id_provider += 1;
                            *id_provider - 1
                        };

                        let mut seq_transitions = seq[i].generate(id_provider, from_id, to_id);
                        transitions.append(&mut seq_transitions);

                        from_id = to_id;
                    }

                    transitions
                }
            }
            Self::Alt(alts) => {
                let mut transitions = vec![];

                for alt in alts {
                    let mut alt_transitions = alt.generate(id_provider, start_state, end_state);
                    transitions.append(&mut alt_transitions);
                }

                transitions
            }
            Self::Repeat { min, max, node } => {
                if max.map(|v| v == 0).unwrap_or(false) {
                    return vec![Transition {
                        from_state: start_state,
                        to_state: end_state,
                        cond: Cond::None,
                        max_use: None,
                    }];
                }

                if max
                    .and_then(|max_v| min.map(|min_v| max_v < min_v))
                    .unwrap_or(false)
                {
                    panic!("Invalid repeat range");
                }

                let mut transitions = vec![];
                let min = min.unwrap_or(0);
                let req_len = if max.map(|v| v >= min).unwrap_or(true) && min > 1 {
                    min - 1
                } else {
                    0
                };
                let optional_len = max.map(|v| v - req_len - 1);

                let mut inner_start = *id_provider;
                *id_provider += 1;
                let mut inner_end = *id_provider;
                *id_provider += 1;

                transitions.push(Transition {
                    from_state: start_state,
                    to_state: inner_start,
                    cond: Cond::None,
                    max_use: None,
                });

                if min == 0 {
                    transitions.push(Transition {
                        from_state: start_state,
                        to_state: end_state,
                        cond: Cond::None,
                        max_use: None,
                    });
                }

                for _ in 0..req_len {
                    let mut inner_t = node.generate(id_provider, inner_start, inner_end);
                    transitions.append(&mut inner_t);
                    inner_start = inner_end;
                    inner_end = *id_provider;
                    *id_provider += 1;
                }

                transitions.push(Transition {
                    from_state: inner_end,
                    to_state: end_state,
                    cond: Cond::None,
                    max_use: None,
                });

                let mut inner_t = node.generate(id_provider, inner_start, inner_end);
                transitions.append(&mut inner_t);

                transitions.push(Transition {
                    from_state: inner_end,
                    to_state: inner_start,
                    cond: Cond::None,
                    max_use: optional_len,
                });

                transitions
            }
        }
    }
}

fn create_dot_file_from_transitions(transitions: &Vec<Transition>) {
    let mut f = File::create("./state_machine.dot").unwrap();

    f.write_all(b"digraph {{\n").unwrap();

    for tr in transitions {
        f.write_all(
            format!(
                "\t{} -> {} [label=\"{}\"]\n",
                state_id_to_label(tr.from_state),
                state_id_to_label(tr.to_state),
                tr.to_label()
            )
            .as_bytes(),
        )
        .unwrap();
    }

    f.write_all(b"}}\n").unwrap();
}

fn state_id_to_label(id: u64) -> String {
    match id {
        0 => "Start".to_string(),
        1 => "End".to_string(),
        other => other.to_string(),
    }
}

struct Evaluator {
    transitions: Vec<Transition>,
}

impl Evaluator {
    fn new(transitions: Vec<Transition>) -> Self {
        Self { transitions }
    }

    fn is_match(&self, chars: &[Token]) -> bool {
        for offset in 0..chars.len() {
            let chars = &chars[offset..];
            let mut stack = vec![(chars, 0u64)];

            while !stack.is_empty() {
                let (stream, current_state) = stack.pop().unwrap();
                if current_state == 1 {
                    return true;
                }

                let available_transitions = self.get_available_transitions(current_state);

                for tr in available_transitions {
                    match tr.cond.is_match(stream.get(0)) {
                        MatchResult::MatchAndConsume => stack.push((&stream[1..], tr.to_state)),
                        MatchResult::MatchNoConsume => stack.push((stream, tr.to_state)),
                        MatchResult::NoMatch => {}
                    }
                }
            }
        }

        false
    }

    fn get_available_transitions(&self, start_state: u64) -> Vec<&Transition> {
        let mut transitions = vec![];

        for t in &self.transitions {
            if t.from_state == start_state {
                transitions.push(t);
            }
        }

        transitions
    }
}

#[cfg(test)]
mod test {
    use crate::v0::{AstNode, Evaluator, Token, create_dot_file_from_transitions};

    #[test]
    fn test_generation() {
        let root = AstNode::Root(Box::new(AstNode::Seq(vec![
            AstNode::Alt(vec![
                AstNode::Char('a'),
                AstNode::Char('b'),
                AstNode::Seq(vec![AstNode::Char('x'), AstNode::Char('y')]),
            ]),
            AstNode::Alt(vec![
                AstNode::Seq(vec![AstNode::Char('1'), AstNode::Char('1')]),
                AstNode::Seq(vec![AstNode::Char('2'), AstNode::Char('2')]),
            ]),
            AstNode::Char('c'),
        ])));

        let transitions = root.generate(&mut 2, 0, 1);
        dbg!(&transitions);
        // create_dot_file_from_transitions(&transitions);
    }

    #[test]
    fn test_complex() {
        let root = AstNode::Root(Box::new(AstNode::Seq(vec![
            AstNode::Alt(vec![
                AstNode::Seq(vec![
                    AstNode::Char('x'),
                    AstNode::Char('x'),
                    AstNode::Char('1'),
                ]),
                AstNode::Seq(vec![AstNode::Char('x'), AstNode::Char('x')]),
            ]),
            AstNode::Alt(vec![
                AstNode::Seq(vec![AstNode::Char('1'), AstNode::Char('1')]),
                AstNode::Seq(vec![AstNode::Char('2'), AstNode::Char('2')]),
            ]),
            AstNode::Char('c'),
        ])));

        let transitions = root.generate(&mut 2, 0, 1);
        dbg!(&transitions);
        // create_dot_file_from_transitions(&transitions);

        let evaluator = Evaluator::new(transitions);
        assert!(evaluator.is_match(&str_to_tokens("xx11c")[..]));
        assert!(evaluator.is_match(&str_to_tokens("__xx11c")[..]));
        assert!(!evaluator.is_match(&str_to_tokens("__xx11")[..]));
    }

    #[test]
    fn test_repeat() {
        let root = AstNode::Root(Box::new(AstNode::Seq(vec![
            AstNode::Alt(vec![
                AstNode::Repeat {
                    min: Some(0),
                    max: Some(0),
                    node: Box::new(AstNode::Char('a')),
                },
                AstNode::Char('b'),
                AstNode::Seq(vec![AstNode::Char('x'), AstNode::Char('y')]),
            ]),
            AstNode::Alt(vec![
                AstNode::Seq(vec![AstNode::Char('1'), AstNode::Char('1')]),
                AstNode::Seq(vec![AstNode::Char('2'), AstNode::Char('2')]),
            ]),
            AstNode::Char('c'),
        ])));

        let transitions = root.generate(&mut 2, 0, 1);
        dbg!(&transitions);
        create_dot_file_from_transitions(&transitions);
    }

    fn str_to_tokens(s: &str) -> Vec<Token> {
        let mut out = s.chars().map(|c| Token::Char(c)).collect::<Vec<_>>();

        out.insert(0, Token::Start);
        out.push(Token::End);

        out
    }
}
