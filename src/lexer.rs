use std::fmt;
use std::error::Error as StdError;
use regex::Regex;

#[allow(dead_code)]
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
    Apostrophe,
    Backquote,
    Unquote,
    UnquoteSplicing,
    Comment,
}

impl Token {
    pub fn is_hidden(&self) -> bool {
        match self {
            &Token::Whitespace => true,
            &Token::Comment => true,
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

#[allow(dead_code)]
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

lazy_static! {
    static ref TOKEN_MATCHES: Regex = Regex::new(r"(?x)
        (?P<whitespace>^\s+)                 |
        (?P<comment>;(.*)\n)                 |
        (?P<oparen>^\()                      |
        (?P<cparen>^\))                      |
        (?P<obracket>^\[)                    |
        (?P<cbracket>^\])                    |
        (?P<integer>^[0-9]+)                 |
        (?P<apostrophe>^')                   |
        (?P<backquote>^`)                    |
        (?P<unquote_splicing>^~@)            |
        (?P<unquote>^~)                      |
        (?P<identifier>^([^\s\(\)\[\]\{\}]+))
    ").unwrap();
}

pub fn lex(content: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens: Vec<Token> = vec![];
    let mut code = content;

    while code != "" {
        let found_token = lex_single_token(code);

        match found_token {
            Ok((name, token)) => {
                // let code_token = &code[0..token.len()];
                // if token.cmp(&code_token.to_string()) != Ordering::Equal {
                //     return Result::Err(LexError::InvalidToken(code_token.to_string(),
                //                                               token.to_string()));
                // }
                code = &code[token.len()..code.len()];

                tokens.push(match name.as_str() {
                    "whitespace" => Token::Whitespace,
                    "oparen" => Token::Oparen,
                    "cparen" => Token::Cparen,
                    "obracket" => Token::Oparen,
                    "cbrakcet" => Token::Cparen,
                    "identifier" => Token::Identifier(token),
                    "integer" => Token::Int(token.parse::<i64>().unwrap()),
                    "apostrophe" => Token::Apostrophe,
                    "backquote" => Token::Backquote,
                    "unquote" => Token::Unquote,
                    "unquote_splicing" => Token::UnquoteSplicing,
                    "comment" => Token::Comment,
                    _ => {
                        println!("NAME: {:?}", name);
                        unreachable!()
                    }

                });
            }
            Err(err) => return Result::Err(err),
        }
    }

    Result::Ok(tokens)
}

fn lex_single_token(str: &str) -> Result<(String, String), LexError> {
    for cap in TOKEN_MATCHES.captures_iter(str) {
        for (name, matched) in cap.iter_named() {
            if let Some(ref token) = matched {
                return Result::Ok((name.to_string(), token.to_string()));
            }
        }
    }

    return Result::Err(LexError::Syntax);
}
