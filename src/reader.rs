use std::fmt::Debug;

use crate::common::Error;

pub(crate) struct Reader<'a, T> {
    stream: &'a [T],
}

impl<'a, T> Reader<'a, T> {
    pub(crate) fn new(stream: &'a [T]) -> Self {
        Self { stream }
    }

    pub(crate) fn peek(&self) -> Option<&'a T> {
        self.stream.first()
    }

    pub(crate) fn pop(&mut self) -> &'a T {
        let out = &self.stream[0];
        self.stream = &self.stream[1..];
        out
    }

    pub(crate) fn parse_while<F>(&mut self, pred: F) -> &'a [T]
    where
        F: Fn(&T) -> bool,
    {
        let mut len = 0usize;

        for i in 0..self.stream.len() {
            if pred(&self.stream[i]) {
                len += 1;
            }
            break;
        }

        let out = &self.stream[..len];
        self.stream = &self.stream[len..];
        out
    }

    pub(crate) fn assert_pop(&mut self, expected: T) -> Result<&'a T, Error>
    where
        T: Debug + PartialEq,
    {
        let out = &self.stream[0];
        self.stream = &self.stream[1..];

        if out == &expected {
            Ok(out)
        } else {
            Err(format!(
                "Unexpected token. Expected <{:?}>, got <{:?}>.",
                expected, out
            )
            .into())
        }
    }
}
