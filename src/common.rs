use crate::token::Token;

pub(crate) const EXIT_CODE_SUCCESS: i32 = 0;
pub(crate) const EXIT_CODE_NO_MATCH: i32 = 1;

pub(crate) const START_STATE: u64 = 0;
pub(crate) const END_STATE: u64 = 1;

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

    pub(crate) fn new_from(v: u64) -> Self {
        Self { v }
    }

    pub(crate) fn get(&mut self) -> u64 {
        self.v += 1;
        self.v - 1
    }
}

pub(crate) fn merge_overlapping_match_ranges(ranges: &Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut ranges = ranges.clone();
    ranges.sort();

    let mut out: Vec<(usize, usize)> = vec![];

    for (start, end) in ranges {
        if out.is_empty() || out.last().unwrap().1 < start {
            out.push((start, end));
        } else {
            out.last_mut().unwrap().1 = end;
        }
    }

    out
}

// Compensate for <start> token.
pub(crate) fn range_start_adjust(start: usize) -> usize {
    if start == 0 { 0 } else { start - 1 }
}

// Compensate for <start> and <end> tokens.
pub(crate) fn range_end_adjust(end: usize, len: usize) -> usize {
    if end > len {
        len
    } else if end >= 1 {
        end - 1
    } else {
        end
    }
}
