extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate rustyline;

mod data;
mod lexer;
mod parser;
mod env;
mod eval;
mod core;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use data::{c_int, c_list, c_nil, c_symbol};
use env::{c_env, Env};
use lexer::{lex, format_tokens};
use parser::Parser;

fn eval(str: &str, env: Env) {
    let tokens = lex(str);
    match tokens {
        Ok(ref tokens) => {
            let prefix = format!("exp: {} -> lex: {}", str, format_tokens(tokens));
            let parser = Parser::new(tokens);
            match parser.start() {
                Ok(ast) => {
                    print!("{} -> ast: {}", prefix, ast.format(true));

                    match eval::eval(ast, env.clone()) {
                        Ok(result) => println!(" -> {}", result),
                        Err(err) => println!(" -> {}", err),
                    }

                    println!("{}", *(*env).borrow());
                }
                Err(err) => println!("{} -> ast: {}", prefix, err),
            }
        }
        Err(err) => println!("lex: {} {}", str, err),
    }
}

fn repl(env: Env) {
    use std::io;

    let mut rl = Editor::<()>::new();
    if let Err(_) = rl.load_history("history.txt") {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                eval(line.as_str(), env.clone());
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}

fn main() {
    let env = core::build();
    // c_list(vec![c_list(vec![c_int(1), c_symbol(String::from("ok"))]),
    // c_list(vec![c_int(1), c_nil()])]);

    // eval("(+ 2 3 (5 4 5 (+ 2 7) (+ 2 2) (+ 3 4)))", env.clone());
    // eval("(()");
    // eval("()");
    // eval("))");
    // eval("1");
    // eval("(1 2)");
    // eval("(test NIl)");
    // eval("(test nil)");
    // eval("(- 2 3)");
    // eval("(+ 2 3)", env.clone());
    // eval("(+ 0 (+ 2 2) (- 1 1) (* 2 2) (/ 2 2))");
    // eval("foo", env.clone());
    // eval("(quote foo)", env.clone());
    // eval("(def foo 99)", env.clone());
    // eval("foo", env.clone());
    // eval("(def foo (quote bar))", env.clone());
    // eval("foo", env.clone());

    repl(env);
}
