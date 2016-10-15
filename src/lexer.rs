use std::fmt;
use std::error::Error as StdError;
use regex::Regex;

#[derive(Debug)]
pub enum LexError {
    Syntax,
}


impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &LexError::Syntax => write!(f, "Syntax error"),
        }
    }
}

impl StdError for LexError {
    fn description(&self) -> &str {
        match *self {
            LexError::Syntax => "Syntax error",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            LexError::Syntax => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Oparen,
    Cparen,
    Identifier(String),
    Int(i64),
    Whitespace,
}

impl Token {
    pub fn is_hidden(&self) -> bool {
        match self {
            &Token::Whitespace => true,
            _ => false,
        }
    }
}

// Use regex set?
lazy_static! {
    static ref TOKEN_MATCHES: Vec<(&'static str, Regex)> = {
        vec![
            ("whitespace", Regex::new(r"^\s+").unwrap()),
            ("oparen", Regex::new(r"^\(").unwrap()),
            ("cparen", Regex::new(r"^\)").unwrap()),
            ("identifier", Regex::new(r"^[A-Za-z\+-]+").unwrap()),
            ("integer", Regex::new(r"[0-9]+").unwrap()),
        ]
    };
}

pub fn lex(content: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens: Vec<Token> = vec![];
    let mut code = content;

    while code != "" {
        let found_token = lex_single_token(code);

        match found_token {
            Ok((name, token)) => {
                code = &code[token.len()..code.len()];
                // println!("Token: {:?}, code: |{}|", token, code);

                tokens.push(match name {
                    "whitespace" => Token::Whitespace,
                    "oparen" => Token::Oparen,
                    "cparen" => Token::Cparen,
                    "identifier" => Token::Identifier(token.to_string()),
                    "integer" => Token::Int(token.parse::<i64>().unwrap()),
                    _ => unreachable!(),

                });
            }
            Err(err) => return Result::Err(err),
        }
    }

    Result::Ok(tokens)
}

fn lex_single_token(str: &str) -> Result<(&str, &str), LexError> {
    for &(ref name, ref matcher) in TOKEN_MATCHES.iter() {
        if let Some(caps) = matcher.captures(str) {
            if let Some(token_content) = caps.at(0) {
                return Result::Ok((name, token_content));
            }
        }

    }

    Result::Err(LexError::Syntax)
}
