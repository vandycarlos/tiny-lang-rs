use crate::value::{Ratio, Value};
use std::str::CharIndices;

pub struct Reader<'a> {
    pub name: &'a str,
    content: &'a str,
    chars: CharIndices<'a>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReadError {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub message: String,
}

impl<'a> Reader<'a> {
    pub fn new(name: &'a str, content: &'a str) -> Reader<'a> {
        Reader {
            name,
            content,
            chars: content.char_indices(),
        }
    }

    pub fn read(&mut self) -> Option<Result<Value, ReadError>> {
        self.skip_whitespace();

        self.chars.clone().next().map(|(pos, ch)| match (pos, ch) {
            (start, '0'..='9') => self.read_number(start),
            (start, ch @ '+') | (start, ch @ '-') => self.read_number_or_symbol(start, ch),
            (start, '"') => self.read_string(start),
            (start, ':') => self.read_keyword(start),
            (start, open @ '(') => self.read_list(start, open, ')', Value::ListParen),
            (start, open @ '[') => self.read_list(start, open, ']', Value::ListBracket),
            (start, open @ '{') => self.read_list(start, open, '}', Value::ListBrace),
            (start, ch) if is_symbol_head(ch) => self.read_symbol(start),
            (_, '/') => {
                self.chars.next();
                Ok(Value::Symbol("".into(), "/".into()))
            }
            (start, char) => Err(ReadError {
                name: self.name.into(),
                start,
                end: start,
                message: format!("unexpected char '{char}'"),
            }),
        })
    }

    fn read_number(&mut self, start: usize) -> Result<Value, ReadError> {
        let end = self.advance_while(|ch| ch.is_digit(10));
        self.read_number_rest(start, end)
    }

    fn read_symbol(&mut self, start: usize) -> Result<Value, ReadError> {
        self.chars.next();
        let end = self.advance_while(is_symbol_tail);
        Ok(Value::Symbol("".into(), self.content[start..end].into()))
    }

    fn read_keyword(&mut self, start: usize) -> Result<Value, ReadError> {
        self.chars.next();
        let end = self.advance_while(is_symbol_tail);
        Ok(Value::Keyword(
            "".into(),
            self.content[start + 1..end].into(),
        ))
    }

    fn read_number_or_symbol(&mut self, start: usize, ch: char) -> Result<Value, ReadError> {
        self.chars.next();
        match self.peek() {
            Some('0'..='9') => {
                let start = if ch == '+' { start + 1 } else { start };
                let end = self.advance_while(|ch| ch.is_digit(10));
                self.read_number_rest(start, end)
            }
            Some(ch) if is_symbol_tail(ch) => {
                let end = self.advance_while(is_symbol_tail);
                Ok(Value::Symbol("".into(), self.content[start..end].into()))
            }
            None | Some(' ') | Some('\t') | Some('\n') => {
                Ok(Value::Symbol("".into(), ch.to_string()))
            }
            Some(')') | Some(']') | Some('}') => Ok(Value::Symbol(
                "".into(),
                self.content[start..start + 1].into(),
            )),
            Some(char) => Err(ReadError {
                name: self.name.into(),
                start,
                end: start,
                message: format!("unespected char '{char}'"),
            }),
        }
    }

    fn read_number_rest(&mut self, start: usize, end: usize) -> Result<Value, ReadError> {
        let peek = self.peek();
        if peek == Some('.') {
            self.chars.next();
            let end = self.advance_while(|ch| ch.is_digit(10));
            Ok(Value::Float(self.content[start..end].parse().unwrap()))
        } else if peek == Some('/') {
            self.chars.next();
            let end = self.advance_while(|ch| ch.is_digit(10));
            match parse_ratio(self.content[start..end].into()) {
                Ok(ratio) => Ok(Value::Rational(ratio)),
                _ => Err(ReadError {
                    name: self.name.into(),
                    start,
                    end,
                    message: "invalid rational".into(),
                }),
            }
        } else {
            Ok(Value::Int(self.content[start..end].parse().unwrap()))
        }
    }

    fn read_string(&mut self, start: usize) -> Result<Value, ReadError> {
        self.chars.next();
        let mut string = String::new();
        loop {
            match self.chars.next() {
                Some((_, '"')) => {
                    return Ok(Value::String(string));
                }
                Some((_, '\\')) => {
                    string.push(match self.chars.next() {
                        Some((_, 't')) => '\t',
                        Some((_, 'r')) => '\r',
                        Some((_, 'n')) => '\n',
                        Some((_, '\\')) => '\\',
                        Some((_, '"')) => '\"',
                        Some((pos, ch)) => {
                            return Err(ReadError {
                                name: self.name.into(),
                                start: pos - 1,
                                end: pos + 1,
                                message: format!("invalid string escape `\\{ch}`"),
                            });
                        }
                        None => unimplemented!(),
                    });
                }
                Some((_, ch)) => string.push(ch),
                None => {
                    return Err(ReadError {
                        name: self.name.into(),
                        start,
                        end: self.content.len(),
                        message: "expected closing `\"`, found EOF".into(),
                    });
                }
            }
        }
    }

    fn read_list<List: Fn(Vec<Value>) -> Value>(
        &mut self,
        start: usize,
        open: char,
        close: char,
        list: List,
    ) -> Result<Value, ReadError> {
        self.chars.next();
        let mut items = vec![];
        loop {
            self.skip_whitespace();

            if self.peek() == Some(close) {
                self.chars.next();
                return Ok(list(items));
            }

            match self.read() {
                Some(Ok(value)) => items.push(value),
                Some(Err(err)) => return Err(err),
                None => {
                    return Err(ReadError {
                        name: self.name.into(),
                        start,
                        end: self.content.len(),
                        message: format!("unclosed `{open}`"),
                    })
                }
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.clone().next().map(|(_, ch)| ch)
    }

    fn skip_whitespace(&mut self) {
        loop {
            // Skip whitespace.
            self.advance_while(|ch| ch.is_whitespace());
            // Skip comment if present.
            if self.chars.clone().next().map_or(false, |(_, ch)| ch == ';') {
                self.advance_while(|ch| ch != '\n');
                self.chars.next();
            } else {
                // Otherwise we're done.
                return;
            }
        }
    }

    fn advance_while<F: Fn(char) -> bool>(&mut self, f: F) -> usize {
        loop {
            match self.chars.clone().next() {
                Some((pos, ch)) => {
                    if f(ch) {
                        self.chars.next();
                    } else {
                        return pos;
                    }
                }
                None => {
                    return self.content.len();
                }
            }
        }
    }
}

fn is_symbol_head(ch: char) -> bool {
    match ch {
        'a'..='z'
        | 'A'..='Z'
        | '.'
        | ','
        | ':'
        | '*'
        | '+'
        | '!'
        | '-'
        | '_'
        | '?'
        | '$'
        | '%'
        | '&'
        | '='
        | '<'
        | '>'
        | '@'
        | '#'
        | '~'
        | '^'
        | '`'
        | '|'
        | '\''
        | '\\' => true,
        _ => false,
    }
}

fn is_symbol_tail(ch: char) -> bool {
    is_symbol_head(ch)
        || (match ch {
            '0'..='9' | '/' => true,
            _ => false,
        })
}

fn parse_ratio(s: &str) -> Result<Ratio, String> {
    let mut split = s.splitn(2, '/');
    match (split.next(), split.next(), split.next()) {
        (Some(numer), Some(denom), None) => match (numer.parse::<i64>(), denom.parse::<i64>()) {
            (Ok(n), Ok(d)) => Ok(Ratio::new(n, d)),
            (_, _) => Err("invalid rational".into()),
        },
        _ => Err("invalid rational".into()),
    }
}
