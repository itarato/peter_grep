use std::collections::HashMap;

use crate::token::Token;

#[derive(Debug, Clone)]
pub(crate) struct Capturer {
    captures: HashMap<u64, String>,
    currents: Vec<u64>,
}

impl Capturer {
    pub(crate) fn new() -> Self {
        Self {
            captures: HashMap::new(),
            currents: vec![],
        }
    }

    pub(crate) fn start_capture(&mut self, id: u64) {
        assert!(!self.currents.contains(&id));
        self.currents.push(id);
        self.captures.insert(id, String::new());
    }

    pub(crate) fn end_capture(&mut self, id: u64) {
        assert!(id == self.currents.pop().unwrap());
    }

    pub(crate) fn push(&mut self, tokens: &[Token]) {
        for token in tokens {
            match token {
                Token::Char(c) => {
                    for id in &self.currents {
                        self.captures.get_mut(id).unwrap().push(*c);
                    }
                }
                _ => {}
            }
        }
    }
}
