extern crate regex;
#[macro_use]
extern crate lazy_static;

mod data;
mod lexer;
mod parser;
mod env;
mod eval;

use data::{c_int, c_list, c_nil, c_symbol};
use lexer::lex;
use parser::Parser;

fn eval(str: &str) {
    let tokens = lex(str);
    match tokens {
        Ok(ref tokens) => {
            let prefix = format!("{} -> {:?}", str, tokens);
            let parser = Parser::new(tokens);
            match parser.start() {
                Ok(ast) => {
                    print!("{} -> {}", prefix, ast);
                    match eval::eval(ast) {
                        Ok(result) => println!(" -> {}", result),
                        Err(err) => println!(" -> {:?}", err),
                    }
                }
                Err(err) => println!("{} -> error: {}", prefix, err),
            }
        }
        Err(err) => println!("lex error: {} {}", str, err),
    }
}

fn main() {
    c_list(vec![c_list(vec![c_int(1), c_symbol(String::from("ok"))]),
                c_list(vec![c_int(1), c_nil()])]);

    eval("(1 2 3 (4 5 (6, 7) (1 2) (3 4)))");
    eval("(");
    eval("()");
    eval("))");
    eval("1");
    eval("(1 2)");
    eval("(test NIl)");
    eval("(- 2 3)");
    eval("(+ 2 3)");
    eval("(+ 0 (+ 2 2) (+ 1 1))");
}
