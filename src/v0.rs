use std::{fs::File, io::Write};

#[derive(Debug)]
enum Cond {
    Char(char),
    None,
}

#[derive(Debug)]
struct Transition {
    from_state: u64,
    to_state: u64,
    cond: Cond,
}

#[derive(Debug)]
enum AstNode {
    Root(Box<AstNode>),
    Char(char),
    Seq(Vec<AstNode>),
    Alt(Vec<AstNode>),
}

impl AstNode {
    fn generate(&self, id_provider: &mut u64, start_state: u64, end_state: u64) -> Vec<Transition> {
        match self {
            Self::Root(inner) => inner.generate(id_provider, start_state, end_state),
            Self::Char(c) => vec![Transition {
                from_state: start_state,
                to_state: end_state,
                cond: Cond::Char(*c),
            }],
            Self::Seq(seq) => {
                if seq.is_empty() {
                    vec![Transition {
                        from_state: start_state,
                        to_state: end_state,
                        cond: Cond::None,
                    }]
                } else {
                    let mut transitions = vec![];
                    let mut from_id = start_state;

                    for i in 0..seq.len() {
                        *id_provider += 1;
                        let to_id = if i + 1 == seq.len() {
                            end_state
                        } else {
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
        }
    }
}

fn create_dot_file_from_transitions(transitions: &Vec<Transition>) {
    let mut f = File::create("./state_machine.dot").unwrap();

    f.write_all(b"digraph {{").unwrap();

    for tr in transitions {
        f.write_all(format!("\t{} -> {}", tr.from_state, tr.to_state).as_bytes())
            .unwrap();
    }

    f.write_all(b"}}").unwrap();
}

#[cfg(test)]
mod test {
    use crate::v0::AstNode;

    #[test]
    fn test_generation() {
        let root = AstNode::Root(Box::new(AstNode::Seq(vec![
            AstNode::Alt(vec![AstNode::Char('a'), AstNode::Char('b')]),
            AstNode::Char('c'),
        ])));

        dbg!(root.generate(&mut 2, 0, 1));
    }
}
