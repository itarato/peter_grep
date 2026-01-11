use crate::token::Token;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;

pub(crate) fn str_to_tokens(s: &str) -> Vec<Token> {
    let mut out = s.chars().map(|c| Token::Char(c)).collect::<Vec<_>>();

    out.insert(0, Token::Start);
    out.push(Token::End);

    out
}
