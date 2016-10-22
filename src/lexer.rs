use std::fmt;
use std::error::Error as StdError;
use regex::Regex;
use std::cmp::Ordering;

#[derive(Debug)]
pub enum LexError {
    Syntax,
    InvalidToken(String, String),
}


impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LexError::Syntax => format!("{}", self.description()).fmt(f),
            LexError::InvalidToken(ref code_token, ref found_token) => {
                format!("{} expected: `{}`, found: `{}`",
                        self.description(),
                        code_token,
                        found_token)
                    .fmt(f)
            }
        }
    }
}

impl StdError for LexError {
    fn description(&self) -> &str {
        match *self {
            LexError::Syntax => "Syntax error",
            LexError::InvalidToken(_, _) => "Invalid token",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            _ => None,
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_hidden() {
            write!(f, "")
        } else {
            write!(f, "{:?}", self)
        }
    }
}

pub fn format_tokens(tokens: &Vec<Token>) -> String {
    let mut output = String::new();

    output.push('[');
    for token in tokens.iter() {
        let formatted_token = &format!("{}", token);
        if formatted_token != "" {
            output.push_str(formatted_token);
            output.push_str(", ")
        }
    }
    output.push(']');

    output
}

// Use regex set?
lazy_static! {
    static ref TOKEN_MATCHES: Vec<(&'static str, Regex)> = {
        vec![
            ("whitespace", Regex::new(r"^\s+").unwrap()),
            ("oparen", Regex::new(r"^\(").unwrap()),
            ("cparen", Regex::new(r"^\)").unwrap()),
            ("integer", Regex::new(r"^[0-9]+").unwrap()),
            ("identifier", Regex::new(r"^([^\s\(\)\[\]\{\}]+)").unwrap()),
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
                let code_token = &code[0..token.len()];

                if token.cmp(code_token) != Ordering::Equal {
                    return Result::Err(LexError::InvalidToken(code_token.to_string(),
                                                              token.to_string()));
                }

                code = &code[token.len()..code.len()];

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
        let result = matcher.captures(str)
            .and_then(|caps| caps.at(0))
            .and_then(|token_content| Some((*name, token_content)));

        if let Some(res) = result {
            return Result::Ok(res);
        }
    }

    Result::Err(LexError::Syntax)
}
