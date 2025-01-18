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

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
        }
    }
}

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
        let Some(first_char) = value.chars().next() else {
            return Token::Nonspecials(Rc::from(value));
        };
        if value == "~@" {
            return Token::Tat;
        }
        if let Some(comment) = value.strip_prefix(";") {
            return Token::Comment(Rc::from(comment));
        }
        if let Ok(special) = Special::try_from(first_char) {
            return Token::Special(special);
        }
        if let Ok(operator) = Operator::try_from(first_char) {
            return Token::Operator(operator);
        }
        if let Some(stripped) = value.strip_prefix('"') {
            let closed = stripped.ends_with('"');
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

impl std::fmt::Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Operator(op) => write!(f, "{}", op),
            Atom::Literal(lit) => write!(f, "{}", lit),
        }
    }
}

pub enum Type {
    Atom(Atom),
    List(Rc<[Type]>),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Atom(atom) => write!(f, "{}", atom),
            Type::List(list) => {
                let joined = list
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "({})", joined)
            }
        }
    }
}

pub struct Tokens {
    tokens: VecDeque<Token>,
}

impl Iterator for Tokens {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.pop_front()
    }
}

impl Tokens {
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
            .map(|caps| caps.get(1).unwrap().as_str().into())
            .collect()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn read_form(&mut self) -> Option<Type> {
        Some(if self.peek()? == &Token::Special(Special::OpenParen) {
            self.next();
            Type::List(self.read_list()?.into())
        } else {
            Type::Atom(self.read_atom()?)
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
            Token::String { contents, closed } => Some(Atom::Literal(Rc::from(format!(
                "\"{}{}{}",
                contents,
                if closed { "\"" } else { "" },
                if closed { "" } else { "unbalanced" }
            )))),
            Token::Tat | Token::Comment(_) => None,
            Token::Special(special) => match special {
                Special::OpenParen | Special::CloseParen => unreachable!(),
                _ => None,
            },
        }
    }

    pub fn pr_str(&mut self) -> Option<String> {
        self.read_form().map(|t| t.to_string())
    }
}
