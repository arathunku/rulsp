use std::fmt;
use std::error::Error as StdError;
use lexer::Token;
use data::{AtomVal, c_int, c_nil, c_list, c_symbol};

#[derive(Debug)]
pub enum ParseError {
    Syntax,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ParseError::Syntax => write!(f, "Syntax error"),
        }
    }
}

impl StdError for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::Syntax => "Syntax error",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            ParseError::Syntax => None,
        }
    }
}


pub struct Parser {
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        // TODO: avoid clone!!!
        Parser { tokens: tokens.clone() }
    }

    pub fn start(&self) -> Result<AtomVal, ParseError> {
        match self.parse(0) {
            Err(err) => Result::Err(err),
            Ok((atom, _end)) => Result::Ok(atom),
        }
    }

    pub fn parse(&self, pos: usize) -> Result<(AtomVal, usize), ParseError> {
        if let Some(token) = self.pop(pos) {
            if token.is_hidden() {
                self.parse(pos + 1)
            } else {
                match token {
                    &Token::Oparen => self.read_list(pos + 1),
                    &Token::Cparen => Result::Ok((c_nil(), pos)),
                    &Token::Int(num) => Result::Ok((c_int(num), pos)),
                    &Token::Identifier(ref str) => Result::Ok((c_symbol(str.clone()), pos)),
                    _ => Result::Err(ParseError::Syntax),
                }
            }
        } else {
            Result::Err(ParseError::Syntax)
        }
    }

    fn pop(&self, pos: usize) -> Option<&Token> {
        self.tokens.get(pos)
    }

    fn read_list(&self, pos: usize) -> Result<(AtomVal, usize), ParseError> {
        let mut atoms = vec![];
        let mut pos = pos;

        loop {
            if let Some(token) = self.pop(pos) {
                if !token.is_hidden() {
                    match token {
                        &Token::Cparen => {
                            break;
                        }
                        _other => {
                            match self.parse(pos) {
                                Ok((atom, end)) => {
                                    pos = end;
                                    atoms.push(atom)
                                }
                                Err(err) => return Result::Err(err),
                            }
                        }
                    }
                }
            } else {
                return Result::Err(ParseError::Syntax);
            }

            pos += 1;
        }

        Result::Ok((c_list(atoms), pos))
    }
}
