use std::collections::{HashMap, HashSet};

use crate::{
    capturer::Capturer,
    common::{END_STATE, Incrementer},
    cond::MatchResult,
    token::Token,
    transition::{CaptureGroupInstruction, Transition},
};

pub(crate) enum EvalMatchResult {
    NoMatch,
    Match { matches: Vec<(usize, usize)> },
}

impl EvalMatchResult {
    #[allow(dead_code)]
    pub(crate) fn is_match(&self) -> bool {
        match self {
            Self::Match { .. } => true,
            _ => false,
        }
    }
}

pub(crate) struct Evaluator {
    transitions: Vec<Transition>,
}

impl Evaluator {
    pub(crate) fn new(transitions: Vec<Transition>) -> Self {
        Self { transitions }
    }

    /**
     * Max-transition tracking idea:
     * - centrally managed by the evaluator -> use a hash map or something
     * - each journey gets an increasing number
     * - when max-trans end node is reached from a non-max-trans trans - the number is increased
     * - each max-counting is tied to this number
     */
    pub(crate) fn is_match(&self, chars: &[Token]) -> EvalMatchResult {
        let loop_start_transitions = self.get_loop_start_transitions();
        let mut matches = vec![];

        let mut offset = 0;

        'main_loop: while offset < chars.len() {
            let mut visit_counter: HashMap<u64, u64> = HashMap::new();
            let mut id_provider = Incrementer::new();
            let mut stack = vec![(&chars[offset..], id_provider.get(), 0u64, Capturer::new())];

            while let Some((stream, loop_id, current_state, capturer)) = stack.pop() {
                if current_state == END_STATE {
                    matches.push((offset, chars.len() - stream.len()));
                    // `max(offset + 1)` ensures the scanner is not stuck with valid empty matches.
                    offset = (chars.len() - stream.len()).max(offset + 1);
                    continue 'main_loop;
                }

                let available_transitions = self.get_available_transitions(current_state);

                for tr in available_transitions.iter().rev() {
                    // Increase loop_id when starts a loop.
                    let loop_id = if loop_start_transitions.contains(&(tr.from_state, tr.to_state))
                    {
                        id_provider.get()
                    } else {
                        loop_id
                    };

                    // Block if already reached max use.
                    if let Some(max_use) = tr.max_use {
                        let current_use = visit_counter.get(&current_state).unwrap_or(&0);
                        if current_use >= &max_use {
                            continue;
                        }
                    }

                    match tr.cond.is_match(stream, &capturer.captures) {
                        MatchResult::Match(step) => {
                            if tr.max_use.is_some() {
                                *visit_counter.entry(current_state).or_default() += 1;
                            }

                            let mut new_capturer = capturer.clone();
                            new_capturer.push(&stream[..step]);

                            match tr.capture_group_ins {
                                CaptureGroupInstruction::Start(id) => {
                                    new_capturer.start_capture(id)
                                }
                                CaptureGroupInstruction::End(id) => new_capturer.end_capture(id),
                                CaptureGroupInstruction::None => {}
                            }

                            stack.push((&stream[step..], loop_id, tr.to_state, new_capturer));
                        }
                        MatchResult::NoMatch => {}
                    }
                }
            }

            offset += 1;
        }

        if matches.is_empty() {
            EvalMatchResult::NoMatch
        } else {
            EvalMatchResult::Match { matches }
        }
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

    fn get_loop_start_transitions(&self) -> HashSet<(u64, u64)> {
        let mut loop_start_states = HashSet::new();
        for tr in &self.transitions {
            if tr.max_use.is_some() {
                loop_start_states.insert(tr.from_state);
            }
        }

        let mut loop_start_transitions = HashSet::new();
        for tr in &self.transitions {
            if tr.max_use.is_none() && loop_start_states.contains(&tr.to_state) {
                loop_start_transitions.insert((tr.from_state, tr.to_state));
            }
        }

        loop_start_transitions
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

        assert!(eval_match("^x{2}$", "xx"));
        assert!(!eval_match("^x{2}$", "x"));
        assert!(!eval_match("^x{2}$", "xxx"));
        assert!(eval_match("^x{2,4}$", "xxx"));
    }

    fn eval_match(pattern: &str, subject: &str) -> bool {
        let ast = Parser::parse_regex_str(pattern).unwrap();
        let e = Evaluator::new(ast.generate());
        e.is_match(&str_to_tokens(subject)[..]).is_match()
    }
}
