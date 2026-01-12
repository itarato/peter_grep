use std::collections::HashSet;

use crate::{ast::AstNode, common::Error, reader::Reader};

pub(crate) struct Parser;

impl Parser {
    pub(crate) fn parse_regex_str(s: &str) -> Result<AstNode, Error> {
        Self::parse(&mut Reader::new(&s.chars().collect::<Vec<_>>()[..]))
    }

    fn parse(reader: &mut Reader<'_, char>) -> Result<AstNode, Error> {
        let seq_node = Self::parse_sequence(reader, |reader| reader.peek().is_none())?;
        Ok(AstNode::Root(Box::new(seq_node)))
    }

    fn parse_sequence<FnUntil>(
        reader: &mut Reader<'_, char>,
        until_pred: FnUntil,
    ) -> Result<AstNode, Error>
    where
        FnUntil: Fn(&mut Reader<'_, char>) -> bool,
    {
        let mut items = vec![];

        loop {
            if until_pred(reader) {
                break;
            }

            items.push(Self::parse_unit(reader)?);
        }

        Ok(AstNode::Seq(items))
    }

    fn parse_unit(reader: &mut Reader<'_, char>) -> Result<AstNode, Error> {
        match reader.peek() {
            Some(c) => match c {
                '(' => {
                    reader.assert_pop('(')?;
                    let mut alts = vec![];

                    loop {
                        let alt = Self::parse_sequence(reader, |r| match r.peek() {
                            Some(')') | None | Some('|') => true,
                            _ => false,
                        })?;
                        alts.push(alt);

                        match reader.peek() {
                            Some(')') => break,
                            Some('|') => {
                                reader.assert_pop('|')?;
                                continue;
                            }
                            other => {
                                return Err(
                                    format!("Invalid ending of paren group: {:?}", other).into()
                                );
                            }
                        }
                    }

                    reader.assert_pop(')')?;

                    Ok(Self::check_modifier(reader, AstNode::Alt(alts))?)
                }
                '[' => {
                    reader.assert_pop('[')?;
                    let is_negated = if let Some('^') = reader.peek() {
                        reader.assert_pop('^')?;
                        true
                    } else {
                        false
                    };

                    let mut chars = HashSet::new();
                    loop {
                        match reader.peek() {
                            Some(']') => break,
                            None => return Err("Expected closing brace. Got empty.".into()),
                            _ => {
                                let group_char = reader.pop();
                                if let Some('-') = reader.peek() {
                                    reader.assert_pop('-')?;
                                    let until_char = reader.pop();

                                    for ch_u8 in *group_char as u8..=*until_char as u8 {
                                        chars.insert(ch_u8 as char);
                                    }
                                } else {
                                    chars.insert(*group_char);
                                }
                            }
                        }
                    }

                    reader.assert_pop(']')?;

                    Ok(Self::check_modifier(
                        reader,
                        AstNode::CharGroup { is_negated, chars },
                    )?)
                }
                '^' => {
                    reader.pop();
                    Ok(Self::check_modifier(reader, AstNode::Start)?)
                }
                '$' => {
                    reader.pop();
                    Ok(Self::check_modifier(reader, AstNode::End)?)
                }
                '.' => {
                    reader.pop();
                    Ok(Self::check_modifier(reader, AstNode::AnyChar)?)
                }
                '\\' => {
                    reader.pop();
                    match reader.pop() {
                        'd' => Ok(Self::check_modifier(
                            reader,
                            AstNode::CharGroup {
                                is_negated: false,
                                chars: HashSet::from([
                                    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
                                ]),
                            },
                        )?),
                        other => Err(format!("Unexpected char after slash: {}", other).into()),
                    }
                }
                other => {
                    reader.pop(); // char
                    Ok(Self::check_modifier(reader, AstNode::Char(*other))?)
                }
            },
            None => Err("No more input to read".into()),
        }
    }

    fn check_modifier(reader: &mut Reader<'_, char>, node: AstNode) -> Result<AstNode, Error> {
        match reader.peek() {
            Some('*') => {
                reader.pop();
                Ok(AstNode::Repeat {
                    min: None,
                    max: None,
                    node: Box::new(node),
                })
            }
            Some('?') => {
                reader.pop();
                Ok(AstNode::Repeat {
                    min: None,
                    max: Some(1),
                    node: Box::new(node),
                })
            }
            Some('+') => {
                reader.pop();
                Ok(AstNode::Repeat {
                    min: Some(1),
                    max: None,
                    node: Box::new(node),
                })
            }
            Some('{') => {
                reader.pop();
                let min = Some(Self::parse_number(reader)? as usize);
                let max = if let Some(',') = reader.peek() {
                    reader.pop(); // comma

                    if let Some('}') = reader.peek() {
                        None
                    } else {
                        Some(Self::parse_number(reader)? as usize)
                    }
                } else {
                    min
                };
                reader.assert_pop('}')?;

                Ok(AstNode::Repeat {
                    min,
                    max,
                    node: Box::new(node),
                })
            }
            _ => Ok(node),
        }
    }

    fn parse_number(reader: &mut Reader<'_, char>) -> Result<u64, Error> {
        let raw = reader.parse_while(|c| c.is_ascii_digit());
        let raw_str: String = raw.iter().collect();
        u64::from_str_radix(&raw_str, 10).map_err(|err| err.into())
    }
}

#[cfg(test)]
mod test {
    use crate::parser::Parser;

    #[test]
    fn test_parsing() {
        dbg!(Parser::parse_regex_str("^a?.*[^a-f]{1,}$").unwrap());
        dbg!(Parser::parse_regex_str("x(a|bc|([0-3]|.*))").unwrap());
        dbg!(Parser::parse_regex_str("\\d+").unwrap());
    }
}
