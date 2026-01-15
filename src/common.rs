use crate::token::Token;

pub(crate) const EXIT_CODE_SUCCESS: i32 = 0;
pub(crate) const EXIT_CODE_NO_MATCH: i32 = 1;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;

pub(crate) fn str_to_tokens(s: &str) -> Vec<Token> {
    let mut out = s.chars().map(|c| Token::Char(c)).collect::<Vec<_>>();

    out.insert(0, Token::Start);
    out.push(Token::End);

    out
}

pub(crate) struct Incrementer {
    v: u64,
}

impl Incrementer {
    pub(crate) fn new() -> Self {
        Self { v: 0 }
    }

    pub(crate) fn get(&mut self) -> u64 {
        self.v += 1;
        self.v - 1
    }
}
