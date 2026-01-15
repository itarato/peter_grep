use std::collections::HashSet;

use crate::{
    common::{END_STATE, START_STATE},
    cond::{Cond, Literal},
    transition::Transition,
};

#[derive(Debug)]
pub(crate) enum AstNode {
    Root(Box<AstNode>),
    Char(Literal),
    Seq(Vec<AstNode>),
    Alt(Vec<AstNode>),
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
}

impl AstNode {
    pub(crate) fn generate(&self) -> Vec<Transition> {
        self.__generate(&mut (END_STATE + 1), START_STATE, END_STATE)
    }

    fn __generate(
        &self,
        id_provider: &mut u64,
        start_state: u64,
        end_state: u64,
    ) -> Vec<Transition> {
        match self {
            Self::Root(inner) => inner.__generate(id_provider, start_state, end_state),
            Self::Char(c) => vec![Transition {
                from_state: start_state,
                to_state: end_state,
                cond: Cond::Char(c.clone()),
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

                        let mut seq_transitions = seq[i].__generate(id_provider, from_id, to_id);
                        transitions.append(&mut seq_transitions);

                        from_id = to_id;
                    }

                    transitions
                }
            }
            Self::Alt(alts) => {
                let mut transitions = vec![];

                for alt in alts {
                    let mut alt_transitions = alt.__generate(id_provider, start_state, end_state);
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
                    let mut inner_t = node.__generate(id_provider, inner_start, inner_end);
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

                let mut inner_t = node.__generate(id_provider, inner_start, inner_end);
                transitions.append(&mut inner_t);

                transitions.push(Transition {
                    from_state: inner_end,
                    to_state: inner_start,
                    cond: Cond::None,
                    max_use: optional_len,
                });

                transitions
            }
            Self::Start => vec![Transition {
                from_state: start_state,
                to_state: end_state,
                cond: Cond::Start,
                max_use: None,
            }],
            Self::End => vec![Transition {
                from_state: start_state,
                to_state: end_state,
                cond: Cond::End,
                max_use: None,
            }],
            Self::AnyChar => vec![Transition {
                from_state: start_state,
                to_state: end_state,
                cond: Cond::AnyChar,
                max_use: None,
            }],
            Self::CharGroup { is_negated, chars } => vec![Transition {
                from_state: start_state,
                to_state: end_state,
                cond: Cond::CharGroup {
                    chars: chars.clone(),
                    is_negated: *is_negated,
                },
                max_use: None,
            }],
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
            AstNode::Alt(vec![
                AstNode::Char(Literal::Char('a')),
                AstNode::Char(Literal::Char('b')),
                AstNode::Seq(vec![
                    AstNode::Char(Literal::Char('x')),
                    AstNode::Char(Literal::Char('y')),
                ]),
            ]),
            AstNode::Alt(vec![
                AstNode::Seq(vec![
                    AstNode::Char(Literal::Char('1')),
                    AstNode::Char(Literal::Char('1')),
                ]),
                AstNode::Seq(vec![
                    AstNode::Char(Literal::Char('2')),
                    AstNode::Char(Literal::Char('2')),
                ]),
            ]),
            AstNode::Char(Literal::Char('c')),
        ])));

        let transitions = root.generate();
        dbg!(&transitions);
        // create_dot_file_from_transitions(&transitions);
    }

    #[test]
    fn test_complex() {
        let root = AstNode::Root(Box::new(AstNode::Seq(vec![
            AstNode::Alt(vec![
                AstNode::Seq(vec![
                    AstNode::Char(Literal::Char('x')),
                    AstNode::Char(Literal::Char('x')),
                    AstNode::Char(Literal::Char('1')),
                ]),
                AstNode::Seq(vec![
                    AstNode::Char(Literal::Char('x')),
                    AstNode::Char(Literal::Char('x')),
                ]),
            ]),
            AstNode::Alt(vec![
                AstNode::Seq(vec![
                    AstNode::Char(Literal::Char('1')),
                    AstNode::Char(Literal::Char('1')),
                ]),
                AstNode::Seq(vec![
                    AstNode::Char(Literal::Char('2')),
                    AstNode::Char(Literal::Char('2')),
                ]),
            ]),
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
            AstNode::Alt(vec![
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
            ]),
            AstNode::Alt(vec![
                AstNode::Seq(vec![
                    AstNode::Char(Literal::Char('1')),
                    AstNode::Char(Literal::Char('1')),
                ]),
                AstNode::Seq(vec![
                    AstNode::Char(Literal::Char('2')),
                    AstNode::Char(Literal::Char('2')),
                ]),
            ]),
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
    fn test_nested_repeat() {
        create_dot_file_from_transitions(
            &Parser::parse_regex_str("(x{3,6}|y){2,4}")
                .unwrap()
                .generate(),
        );
    }
}
