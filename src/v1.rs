enum Pattern {
    Char(char),
    Seq(Vec<State>),
    Alt(Vec<State>),
    Start,
    End,
}

struct State {
    pattern: Pattern,
    next: Option<Box<State>>,
}

#[cfg(test)]
mod test {
    use crate::v1::{Pattern, State};

    #[test]
    fn test() {
        let state = State {
            pattern: Pattern::Start,
            next: Some(Box::new(State {
                pattern: Pattern::Alt(vec![
                    State {
                        pattern: Pattern::Char('x'),
                        next: None,
                    },
                    State {
                        pattern: Pattern::Char('y'),
                        next: None,
                    },
                ]),
                next: Some(Box::new(State {
                    pattern: Pattern::End,
                    next: None,
                })),
            })),
        };
    }
}
