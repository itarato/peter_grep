use std::collections::HashSet;

use crate::{
    common::{END_STATE, Incrementer, START_STATE},
    cond::{Cond, Literal},
    transition::{CaptureGroupInstruction, Transition},
};

#[derive(Debug)]
pub(crate) enum AstNode {
    Root(Box<AstNode>),
    Char(Literal),
    Seq(Vec<AstNode>),
    Alt {
        options: Vec<AstNode>,
        id: u64,
    },
    Repeat {
        min: Option<u64>,
        max: Option<u64>,
        node: Box<AstNode>,
    },
    Start,
    End,
    AnyChar,
    CharGroup {
        is_negated: bool,
        chars: HashSet<Literal>,
    },
    CaptureRef(u64),
}

impl AstNode {
    pub(crate) fn generate(&self) -> Vec<Transition> {
        let mut id_provider = Incrementer::new_from(END_STATE + 1);
        self.__generate(&mut id_provider, START_STATE, END_STATE)
    }

    fn __generate(
        &self,
        id_provider: &mut Incrementer,
        start_state: u64,
        end_state: u64,
    ) -> Vec<Transition> {
        match self {
            Self::Root(inner) => inner.__generate(id_provider, start_state, end_state),
            Self::Char(c) => vec![Transition::new_cond(
                start_state,
                end_state,
                Cond::Char(c.clone()),
            )],
            Self::Seq(seq) => {
                if seq.is_empty() {
                    vec![Transition::new(start_state, end_state)]
                } else {
                    let mut transitions = vec![];
                    let mut from_id = start_state;

                    for i in 0..seq.len() {
                        let to_id = if i + 1 == seq.len() {
                            end_state
                        } else {
                            id_provider.get()
                        };

                        let mut seq_transitions = seq[i].__generate(id_provider, from_id, to_id);
                        transitions.append(&mut seq_transitions);

                        from_id = to_id;
                    }

                    transitions
                }
            }
            Self::Alt { options, id } => {
                let mut transitions = vec![];

                let inner_start = id_provider.get();
                let inner_end = id_provider.get();

                transitions.push(Transition::new_full(
                    start_state,
                    inner_start,
                    Cond::None,
                    None,
                    CaptureGroupInstruction::Start(*id),
                ));

                for alt in options {
                    let mut alt_transitions = alt.__generate(id_provider, inner_start, inner_end);
                    transitions.append(&mut alt_transitions);
                }

                transitions.push(Transition::new_full(
                    inner_end,
                    end_state,
                    Cond::None,
                    None,
                    CaptureGroupInstruction::End(*id),
                ));

                transitions
            }
            Self::Repeat { min, max, node } => {
                if max.map(|v| v == 0).unwrap_or(false) {
                    return vec![Transition::new(start_state, end_state)];
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

                let mut inner_start = id_provider.get();
                let mut inner_end = id_provider.get();

                // Get to the inner start.
                transitions.push(Transition::new(start_state, inner_start));

                if min == 0 {
                    // Skip - when 0 iter is allowed.
                    transitions.push(Transition::new(start_state, end_state));
                }

                for _ in 0..req_len {
                    let mut inner_t = node.__generate(id_provider, inner_start, inner_end);
                    // Minimum cycle.
                    transitions.append(&mut inner_t);
                    inner_start = inner_end;
                    inner_end = id_provider.get();
                }

                // Repeat transition.
                transitions.push(Transition::new_full(
                    inner_end,
                    inner_start,
                    Cond::None,
                    optional_len,
                    CaptureGroupInstruction::None,
                ));

                let mut inner_t = node.__generate(id_provider, inner_start, inner_end);
                // The actual inside graph.
                transitions.append(&mut inner_t);

                // Get to inner end to end.
                transitions.push(Transition::new(inner_end, end_state));

                transitions
            }
            Self::Start => vec![Transition::new_cond(start_state, end_state, Cond::Start)],
            Self::End => vec![Transition::new_cond(start_state, end_state, Cond::End)],
            Self::AnyChar => vec![Transition::new_cond(start_state, end_state, Cond::AnyChar)],
            Self::CharGroup { is_negated, chars } => vec![Transition::new_cond(
                start_state,
                end_state,
                Cond::CharGroup {
                    chars: chars.clone(),
                    is_negated: *is_negated,
                },
            )],
            Self::CaptureRef(id) => vec![Transition::new_cond(
                start_state,
                end_state,
                Cond::CaptureRef(*id),
            )],
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::AstNode, common::str_to_tokens, cond::Literal, evaluator::Evaluator, parser::Parser,
        transition::create_dot_file_from_transitions,
    };

    #[test]
    fn test_generation() {
        let root = AstNode::Root(Box::new(AstNode::Seq(vec![
            AstNode::Alt {
                options: vec![
                    AstNode::Char(Literal::Char('a')),
                    AstNode::Char(Literal::Char('b')),
                    AstNode::Seq(vec![
                        AstNode::Char(Literal::Char('x')),
                        AstNode::Char(Literal::Char('y')),
                    ]),
                ],
                id: 1,
            },
            AstNode::Alt {
                options: vec![
                    AstNode::Seq(vec![
                        AstNode::Char(Literal::Char('1')),
                        AstNode::Char(Literal::Char('1')),
                    ]),
                    AstNode::Seq(vec![
                        AstNode::Char(Literal::Char('2')),
                        AstNode::Char(Literal::Char('2')),
                    ]),
                ],
                id: 2,
            },
            AstNode::Char(Literal::Char('c')),
        ])));

        let transitions = root.generate();
        dbg!(&transitions);
        // create_dot_file_from_transitions(&transitions);
    }

    #[test]
    fn test_complex() {
        let root = AstNode::Root(Box::new(AstNode::Seq(vec![
            AstNode::Alt {
                options: vec![
                    AstNode::Seq(vec![
                        AstNode::Char(Literal::Char('x')),
                        AstNode::Char(Literal::Char('x')),
                        AstNode::Char(Literal::Char('1')),
                    ]),
                    AstNode::Seq(vec![
                        AstNode::Char(Literal::Char('x')),
                        AstNode::Char(Literal::Char('x')),
                    ]),
                ],
                id: 1,
            },
            AstNode::Alt {
                options: vec![
                    AstNode::Seq(vec![
                        AstNode::Char(Literal::Char('1')),
                        AstNode::Char(Literal::Char('1')),
                    ]),
                    AstNode::Seq(vec![
                        AstNode::Char(Literal::Char('2')),
                        AstNode::Char(Literal::Char('2')),
                    ]),
                ],
                id: 2,
            },
            AstNode::Char(Literal::Char('c')),
        ])));

        let transitions = root.generate();
        dbg!(&transitions);
        // create_dot_file_from_transitions(&transitions);

        let evaluator = Evaluator::new(transitions);
        assert!(evaluator.is_match(&str_to_tokens("xx11c")[..]).is_match());
        assert!(evaluator.is_match(&str_to_tokens("__xx11c")[..]).is_match());
        assert!(!evaluator.is_match(&str_to_tokens("__xx11")[..]).is_match());
    }

    #[test]
    fn test_repeat() {
        let root = AstNode::Root(Box::new(AstNode::Seq(vec![
            AstNode::Alt {
                options: vec![
                    AstNode::Repeat {
                        min: Some(0),
                        max: Some(0),
                        node: Box::new(AstNode::Char(Literal::Char('a'))),
                    },
                    AstNode::Char(Literal::Char('b')),
                    AstNode::Seq(vec![
                        AstNode::Char(Literal::Char('x')),
                        AstNode::Char(Literal::Char('y')),
                    ]),
                ],
                id: 1,
            },
            AstNode::Alt {
                options: vec![
                    AstNode::Seq(vec![
                        AstNode::Char(Literal::Char('1')),
                        AstNode::Char(Literal::Char('1')),
                    ]),
                    AstNode::Seq(vec![
                        AstNode::Char(Literal::Char('2')),
                        AstNode::Char(Literal::Char('2')),
                    ]),
                ],
                id: 2,
            },
            AstNode::Char(Literal::Char('c')),
        ])));

        let transitions = root.generate();
        dbg!(&transitions);
        create_dot_file_from_transitions(&transitions);
    }

    #[test]
    fn test_transition_for_loop() {
        let ast = Parser::parse_regex_str("x{2}").unwrap();
        create_dot_file_from_transitions(&ast.generate());
    }

    #[test]
    fn test_transition_for_optional() {
        // let ast = Parser::parse_regex_str("x?").unwrap();
        // let ast = Parser::parse_regex_str("ab{2}a").unwrap();
        // let ast = Parser::parse_regex_str("a*").unwrap();
        // let ast = Parser::parse_regex_str("a(x|(y|z))b").unwrap();
        // let ast = Parser::parse_regex_str("(\\d+)").unwrap();
        let ast = Parser::parse_regex_str("(cat|dog) and \\1").unwrap();
        create_dot_file_from_transitions(&ast.generate());
    }

    #[test]
    fn test_nested_repeat() {
        create_dot_file_from_transitions(
            &Parser::parse_regex_str("(x{3,6}|y){2,4}")
                .unwrap()
                .generate(),
        );
    }
}
