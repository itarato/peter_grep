use crate::{cond::MatchResult, token::Token, transition::Transition};

pub(crate) struct Evaluator {
    transitions: Vec<Transition>,
}

impl Evaluator {
    pub(crate) fn new(transitions: Vec<Transition>) -> Self {
        Self { transitions }
    }

    pub(crate) fn is_match(&self, chars: &[Token]) -> bool {
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
