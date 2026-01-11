use std::collections::HashSet;

use crate::token::Token;

pub(crate) enum MatchResult {
    MatchAndConsume,
    MatchNoConsume,
    NoMatch,
}

#[derive(Debug)]
pub(crate) enum Cond {
    Char(char),
    AnyChar,
    CharGroup {
        chars: HashSet<char>,
        is_negated: bool,
    },
    Start,
    End,
    None,
}

impl Cond {
    pub(crate) fn to_label(&self) -> String {
        match self {
            Self::Char(c) => format!("C({})", c),
            Self::CharGroup { chars, is_negated } => {
                format!(
                    "[{}{}]",
                    if *is_negated { "^" } else { "" },
                    chars
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join("")
                )
            }
            Self::None => "-".to_string(),
            Self::Start => "^".to_string(),
            Self::End => "$".to_string(),
            Self::AnyChar => ".".to_string(),
        }
    }

    pub(crate) fn is_match(&self, c: Option<&Token>) -> MatchResult {
        match self {
            Self::Char(expected) => match c {
                Some(Token::Char(c)) => {
                    if c == expected {
                        MatchResult::MatchAndConsume
                    } else {
                        MatchResult::NoMatch
                    }
                }
                _ => MatchResult::NoMatch,
            },
            Self::None => MatchResult::MatchNoConsume,
            Self::CharGroup { chars, is_negated } => match c {
                Some(Token::Char(c)) => {
                    if chars.contains(c) ^ is_negated {
                        MatchResult::MatchAndConsume
                    } else {
                        MatchResult::NoMatch
                    }
                }
                _ => MatchResult::NoMatch,
            },
            Self::Start => match c {
                Some(Token::Start) => MatchResult::MatchAndConsume,
                _ => MatchResult::NoMatch,
            },
            Self::End => match c {
                Some(Token::End) => MatchResult::MatchAndConsume,
                _ => MatchResult::NoMatch,
            },
            Self::AnyChar => match c {
                Some(Token::Char(_)) => MatchResult::MatchAndConsume,
                _ => MatchResult::NoMatch,
            },
        }
    }
}
