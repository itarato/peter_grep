#[derive(Debug)]
pub(crate) enum Token {
    Char(char),
    Start,
    End,
}
