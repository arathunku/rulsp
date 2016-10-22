extern crate regex;
#[macro_use]
extern crate lazy_static;

mod data;
mod lexer;
mod parser;
mod env;
mod eval;

use data::{c_int, c_list, c_nil, c_symbol};
use lexer::{lex, format_tokens};
use parser::Parser;

fn eval(str: &str) {
    let tokens = lex(str);
    match tokens {
        Ok(ref tokens) => {
            let prefix = format!("exp: {} -> lex: {}", str, format_tokens(tokens));
            let parser = Parser::new(tokens);
            match parser.start() {
                Ok(ast) => {
                    print!("{} -> ast: {}", prefix, ast.format(true));
                    match eval::eval(ast) {
                        Ok(result) => println!(" -> eval: {}", result),
                        Err(err) => println!(" -> eval: {}", err),
                    }
                }
                Err(err) => println!("{} -> ast: {}", prefix, err),
            }
        }
        Err(err) => println!("lex: {} {}", str, err),
    }
}

fn main() {
    c_list(vec![c_list(vec![c_int(1), c_symbol(String::from("ok"))]),
                c_list(vec![c_int(1), c_nil()])]);

    eval("(+ 2 3 (5 4 5 (+ 2 7) (+ 2 2) (+ 3 4)))");
    eval("(()");
    eval("()");
    eval("))");
    eval("1");
    eval("(1 2)");
    eval("(test NIl)");
    eval("(test nil)");
    eval("(- 2 3)");
    eval("(+ 2 3)");
    eval("(+ 0 (+ 2 2) (- 1 1) (* 2 2) (/ 2 2))");
}
