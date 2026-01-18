use std::collections::HashSet;

use crate::{
    ast::AstNode,
    capturer,
    common::{Error, Incrementer},
    cond::Literal,
    reader::Reader,
};

pub(crate) struct Parser;

impl Parser {
    pub(crate) fn parse_regex_str(s: &str) -> Result<AstNode, Error> {
        Self::parse(&mut Reader::new(&s.chars().collect::<Vec<_>>()[..]))
    }

    fn parse(reader: &mut Reader<'_, char>) -> Result<AstNode, Error> {
        let mut capture_group_id = Incrementer::new_from(1);
        let seq_node = Self::parse_sequence(reader, &mut capture_group_id, |reader| {
            reader.peek().is_none()
        })?;
        Ok(AstNode::Root(Box::new(seq_node)))
    }

    fn parse_sequence<FnUntil>(
        reader: &mut Reader<'_, char>,
        capture_group_id: &mut Incrementer,
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

            items.push(Self::parse_unit(reader, capture_group_id)?);
        }

        Ok(AstNode::Seq(items))
    }

    fn parse_unit(
        reader: &mut Reader<'_, char>,
        capture_id_provider: &mut Incrementer,
    ) -> Result<AstNode, Error> {
        match reader.peek() {
            Some(c) => match c {
                '(' => {
                    let capture_id = capture_id_provider.get();
                    reader.assert_pop('(')?;
                    let mut options = vec![];

                    loop {
                        let alt = Self::parse_sequence(reader, capture_id_provider, |r| {
                            match r.peek() {
                                Some(')') | None | Some('|') => true,
                                _ => false,
                            }
                        })?;
                        options.push(alt);

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

                    Ok(Self::check_modifier(
                        reader,
                        AstNode::Alt {
                            options,
                            id: capture_id,
                        },
                    )?)
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

                                    chars.insert(Literal::Range {
                                        start: *group_char,
                                        end: *until_char,
                                    });
                                } else {
                                    chars.insert(Literal::Char(*group_char));
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
                    match reader.peek() {
                        Some(peeked_c) => match peeked_c {
                            'd' => {
                                reader.pop();
                                Ok(Self::check_modifier(
                                    reader,
                                    AstNode::Char(crate::cond::Literal::Numeric),
                                )?)
                            }
                            'w' => {
                                reader.pop();
                                Ok(Self::check_modifier(
                                    reader,
                                    AstNode::Char(crate::cond::Literal::Alphanumeric),
                                )?)
                            }
                            '1'..'9' => {
                                let id_raw = reader.parse_while(|c| c.is_ascii_digit());
                                let id =
                                    u64::from_str_radix(&id_raw.iter().collect::<String>(), 10)
                                        .unwrap();
                                Ok(Self::check_modifier(reader, AstNode::CaptureRef(id))?)
                            }
                            other => {
                                reader.pop();
                                Ok(Self::check_modifier(
                                    reader,
                                    AstNode::Char(Literal::Char(*other)),
                                )?)
                            }
                        },
                        None => panic!("Missing char after \\"),
                    }
                }
                other => {
                    reader.pop(); // char
                    Ok(Self::check_modifier(
                        reader,
                        AstNode::Char(crate::cond::Literal::Char(*other)),
                    )?)
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
                let min = Some(Self::parse_number(reader)?);
                let max = if let Some(',') = reader.peek() {
                    reader.pop(); // comma

                    if let Some('}') = reader.peek() {
                        None
                    } else {
                        Some(Self::parse_number(reader)?)
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
