use std::collections::HashSet;

use crate::{cond::Cond, transition::Transition};

#[derive(Debug)]
pub(crate) enum AstNode {
    Root(Box<AstNode>),
    Char(char),
    Seq(Vec<AstNode>),
    Alt(Vec<AstNode>),
    Repeat {
        min: Option<usize>,
        max: Option<usize>,
        node: Box<AstNode>,
    },
    Start,
    End,
    AnyChar,
    CharGroup {
        is_negated: bool,
        chars: HashSet<char>,
    },
}

impl AstNode {
    pub(crate) fn generate(
        &self,
        id_provider: &mut u64,
        start_state: u64,
        end_state: u64,
    ) -> Vec<Transition> {
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
            Self::CharGroup { is_negated, chars } => {
                unimplemented!()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::AstNode, evaluator::Evaluator, token::Token,
        transition::create_dot_file_from_transitions,
    };

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
