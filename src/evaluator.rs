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

#[cfg(test)]
mod test {
    use crate::{common::str_to_tokens, evaluator::Evaluator, parser::Parser};

    #[test]
    fn test_match() {
        assert!(eval_match("a", "abba"));
        assert!(eval_match("a", "vva"));

        assert!(eval_match("ab", "abba"));
        assert!(eval_match("ab", "basab"));

        assert!(eval_match("ab{3}a", "abbba"));
        assert!(!eval_match("ab{4}a", "abbba"));

        assert!(eval_match("(aab|aa)[cb]{2,}", "aabb"));
        assert!(eval_match("(aab|aa)[cb]{2,}", "aabc"));
        assert!(!eval_match("(aab|aa)[cb]{2,}", "aab"));

        assert!(eval_match("(aa|bb)(cc|dd|ee)", "aaee"));
        assert!(!eval_match("(aa|bb)(cc|dd|ee)", "aaed"));

        assert!(eval_match("[0-9]+", "a5342ee"));
        assert!(eval_match("[0-9]+", "a9ee"));
        assert!(!eval_match("[0-9]+", "ewe"));

        assert!(eval_match("^[0-9]+", "5342ee"));
        assert!(!eval_match("^[0-9]+", "d5342ee"));
        assert!(eval_match("^[0-9]+$", "5"));
        assert!(!eval_match("^[0-9]+$", "5f"));

        assert!(eval_match("^\\d+$", "5"));
        assert!(eval_match("^\\w+$", "f"));
        assert!(eval_match("^\\w+$", "5cved"));
    }

    fn eval_match(pattern: &str, subject: &str) -> bool {
        let ast = Parser::parse_regex_str(pattern).unwrap();
        let e = Evaluator::new(ast.generate(&mut 2, 0, 1));
        e.is_match(&str_to_tokens(subject)[..])
    }
}
