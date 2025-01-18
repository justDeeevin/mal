use regex::Regex;
use std::{collections::VecDeque, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Tat,
    // TODO: unsure if I actually need to store the contents of the comment
    Comment(Rc<str>),
    Special(Special),
    Operator(Operator),
    // TODO: depending on my use case, it might be better to have a tuple-like field containing
    // Option<Rc<str>>, and contain None for an unclosed string
    String { contents: Rc<str>, closed: bool },
    Nonspecials(Rc<str>),
}

impl Eq for Token {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

impl Eq for Operator {}

impl TryFrom<char> for Operator {
    type Error = ();
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '+' => Ok(Operator::Add),
            '-' => Ok(Operator::Sub),
            '*' => Ok(Operator::Mul),
            '/' => Ok(Operator::Div),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Special {
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Tick,
    Backtick,
    Tilde,
    Caret,
    At,
}

impl Eq for Special {}

impl TryFrom<char> for Special {
    type Error = ();
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '[' => Ok(Special::OpenBracket),
            ']' => Ok(Special::CloseBracket),
            '{' => Ok(Special::OpenBrace),
            '}' => Ok(Special::CloseBrace),
            '(' => Ok(Special::OpenParen),
            ')' => Ok(Special::CloseParen),
            '\'' => Ok(Special::Tick),
            '`' => Ok(Special::Backtick),
            '~' => Ok(Special::Tilde),
            '^' => Ok(Special::Caret),
            '@' => Ok(Special::At),
            _ => Err(()),
        }
    }
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        if value == "~@" {
            return Token::Tat;
        }
        if let Some(comment) = value.strip_prefix(";") {
            return Token::Comment(Rc::from(comment));
        }
        if let Ok(special) = Special::try_from(value.chars().next().unwrap()) {
            return Token::Special(special);
        }
        if let Ok(operator) = Operator::try_from(value.chars().next().unwrap()) {
            return Token::Operator(operator);
        }
        if let Some(stripped) = value.strip_prefix('"') {
            let closed = value.ends_with('"');
            let end = if closed {
                value.len() - 2
            } else {
                value.len() - 1
            };
            return Token::String {
                contents: Rc::from(&stripped[..end]),
                closed,
            };
        }

        Token::Nonspecials(Rc::from(value))
    }
}

pub enum Atom {
    Operator(Operator),
    Literal(Rc<str>),
}

pub enum Type {
    Atom(Atom),
    List(Rc<[Type]>),
}

pub struct Reader {
    tokens: VecDeque<Token>,
}

impl Iterator for Reader {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.pop_front()
    }
}

impl Reader {
    pub fn read_str(input: &str) -> Self {
        Self {
            tokens: Self::tokenize(input).into(),
        }
    }

    fn tokenize(input: &str) -> Vec<Token> {
        let regex =
            Regex::new(r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]*)"#)
                .unwrap();
        regex
            .captures_iter(input)
            .map(|caps| {
                dbg!(&caps);
                caps.get(1).unwrap().as_str().into()
            })
            .collect()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn read_form(&mut self) -> Option<Type> {
        Some(match self.peek()? {
            Token::Special(Special::OpenParen) => {
                self.next();
                Type::List(self.read_list()?.into())
            }
            _ => Type::Atom(self.read_atom()?),
        })
    }

    fn read_list(&mut self) -> Option<Vec<Type>> {
        let mut out = Vec::new();
        while self.peek()? != &Token::Special(Special::CloseParen) {
            out.push(self.read_form()?)
        }

        Some(out)
    }

    fn read_atom(&mut self) -> Option<Atom> {
        match self.next()? {
            Token::Operator(operator) => Some(Atom::Operator(operator)),
            Token::Nonspecials(contents) => Some(Atom::Literal(contents.clone())),
            Token::String { contents, .. } => {
                Some(Atom::Literal(Rc::from(format!("\"{}\"", contents))))
            }
            Token::Tat | Token::Comment(_) => None,
            Token::Special(special) => match special {
                Special::OpenParen | Special::CloseParen => unreachable!(),
                _ => None,
            },
        }
    }
}
