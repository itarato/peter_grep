use std::collections::HashSet;

use crate::token::Token;

pub(crate) enum MatchResult {
    Match(usize),
    NoMatch,
}

impl MatchResult {
    fn is_success(&self) -> bool {
        match self {
            Self::Match(_) => true,
            Self::NoMatch => false,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) enum Literal {
    Char(char),
    Range { start: char, end: char },
    Numeric,
    Alphanumeric,
}

impl Literal {
    fn to_label(&self) -> String {
        match self {
            Self::Alphanumeric => "\\w".to_string(),
            Self::Char(c) => c.to_string(),
            Self::Range { start, end } => format!("{}-{}", start, end),
            Self::Numeric => "\\d".to_string(),
        }
    }

    pub(crate) fn is_match(&self, token: Option<&Token>) -> MatchResult {
        match self {
            Self::Alphanumeric => match token {
                Some(Token::Char(c)) => {
                    if c.is_ascii_alphanumeric() || c == &'_' {
                        MatchResult::Match(1)
                    } else {
                        MatchResult::NoMatch
                    }
                }
                _ => MatchResult::NoMatch,
            },
            Self::Char(c) => match token {
                Some(Token::Char(tc)) => {
                    if tc == c {
                        MatchResult::Match(1)
                    } else {
                        MatchResult::NoMatch
                    }
                }
                _ => MatchResult::NoMatch,
            },
            Self::Numeric => match token {
                Some(Token::Char(c)) => {
                    if c.is_ascii_digit() {
                        MatchResult::Match(1)
                    } else {
                        MatchResult::NoMatch
                    }
                }
                _ => MatchResult::NoMatch,
            },
            Self::Range { start, end } => match token {
                Some(Token::Char(c)) => {
                    if c >= start && c <= end {
                        MatchResult::Match(1)
                    } else {
                        MatchResult::NoMatch
                    }
                }
                _ => MatchResult::NoMatch,
            },
        }
    }
}

#[derive(Debug)]
pub(crate) enum Cond {
    Char(Literal),
    AnyChar,
    CharGroup {
        chars: HashSet<Literal>,
        is_negated: bool,
    },
    Start,
    End,
    None,
    CaptureRef(u64),
}

impl Cond {
    pub(crate) fn to_label(&self) -> String {
        match self {
            Self::Char(t) => t.to_label(),
            Self::CharGroup { chars, is_negated } => {
                format!(
                    "[{}{}]",
                    if *is_negated { "^" } else { "" },
                    chars
                        .iter()
                        .map(|c| c.to_label())
                        .collect::<Vec<_>>()
                        .join("")
                )
            }
            Self::None => "-".to_string(),
            Self::Start => "^".to_string(),
            Self::End => "$".to_string(),
            Self::AnyChar => ".".to_string(),
            Self::CaptureRef(id) => format!("ref{}", id),
        }
    }

    pub(crate) fn is_match(&self, c: Option<&Token>) -> MatchResult {
        match self {
            Self::Char(t) => t.is_match(c),
            Self::None => MatchResult::Match(0),
            Self::CharGroup { chars, is_negated } => match c {
                Some(Token::Char(c)) => {
                    if chars
                        .iter()
                        .any(|group_c| group_c.is_match(Some(&Token::Char(*c))).is_success())
                        ^ is_negated
                    {
                        MatchResult::Match(1)
                    } else {
                        MatchResult::NoMatch
                    }
                }
                _ => MatchResult::NoMatch,
            },
            Self::Start => match c {
                Some(Token::Start) => MatchResult::Match(1),
                _ => MatchResult::NoMatch,
            },
            Self::End => match c {
                Some(Token::End) => MatchResult::Match(1),
                _ => MatchResult::NoMatch,
            },
            Self::AnyChar => match c {
                Some(Token::Char(_)) => MatchResult::Match(1),
                _ => MatchResult::NoMatch,
            },
            Self::CaptureRef(id) => MatchResult::NoMatch,
        }
    }
}
