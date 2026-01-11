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
                '(' => unimplemented!(),
                '[' => unimplemented!(),
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
                assert_eq!(&'}', reader.pop());

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
        dbg!(Parser::parse_regex_str("^a?.*c{1,}$").unwrap());
    }
}
