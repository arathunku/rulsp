#![feature(field_init_shorthand)]

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
use data::{c_int, c_list, c_nil, c_symbol, AtomRet};
use env::{c_env, Env};
use lexer::{lex, format_tokens};
use parser::Parser;

fn eval(str: &str, env: Env) -> AtomRet {
    let tokens = lex(str);
    match tokens {
        Ok(ref tokens) => {
            let prefix = format!("exp: {} -> lex: {}", str, format_tokens(tokens));
            let parser = Parser::new(tokens);
            match parser.start() {
                Ok(ast) => {
                    // print!("{} -> ast: {}\n", prefix, ast.format(true));

                    match eval::eval(ast, env.clone()) {
                        Ok(result) => {
                            println!("=> {}", result);
                            // println!("{}", *(*env).borrow());
                            return Ok(result);
                        }
                        Err(err) => {
                            println!("=> {}", err);
                            return Err(err);
                        }
                    }

                }
                Err(err) => {
                    println!("{} -> ast: {}", prefix, err);
                    // FIXME: should return correct err;
                    return Ok(c_nil());
                }
            }
        }
        Err(err) => {
            println!("lex: {} {}", str, err);
            // FIXME: should return correct err;
            return Ok(c_nil());
        }
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
    repl(env);
}

#[cfg(test)]
mod tests {
    use super::eval;
    use super::core;
    use ::env::{Env, env_get};
    use ::data::{AtomRet, c_int, c_symbol, c_list, c_nil};

    pub fn print(v: AtomRet) -> String {
        match v {
            Ok(ref atom) => format!("{}", atom),
            Err(err) => format!("{}", err),
        }
    }

    fn env() -> Env {
        core::build()
    }

    #[test]
    fn eval_define() {
        let env = env();
        eval("(def foo 1)", env.clone());

        assert_eq!(env_get(&env, &c_symbol("foo".to_string())).unwrap(),
                   c_int(1));
    }

    #[test]
    fn eval_lambda() {
        let env = env();
        assert_eq!(eval("((fn* (x) (- x 2)) 7)", env.clone()).unwrap(),
                   c_int(5));
    }

    #[test]
    fn eval_lambda_nested() {
        let env = env();
        eval("(def make-adder (fn* (x) (fn* (y) (+ x y))))", env.clone());
        eval("(def add-two (make-adder 2))", env.clone());

        assert_eq!(eval("(add-two 5)", env.clone()).unwrap(),
                   c_int(7));
    }

    #[test]
    fn eval_simple_if() {
        let env = env();

        assert_eq!(eval("(if nil (+ 2 2) (+ 1 1))", env.clone()).unwrap(),
                   c_int(2));
        assert_eq!(eval("(if 1 (+ 2 2) (+ 1 1))", env.clone()).unwrap(),
                   c_int(4));
    }

    #[test]
    fn eval_predicated() {
        assert_eq!(eval("(= 2 2)", env()).unwrap(),
                   c_int(1));

        let env = env();
        eval("(def foo 2)", env.clone());
        assert_eq!(eval("(= 2 foo)", env.clone()).unwrap(),
                   c_int(1));
    }

    #[test]
    fn eval_variadic_func() {
        assert_eq!(eval("((fn* (x & y) y) 1)", env()).unwrap(),
                   c_nil());
        assert_eq!(eval("((fn* (x & y) y) 1 2 3)", env()).unwrap(),
                   c_list(vec![c_int(2), c_int(3)]));
        assert_eq!(eval("((fn* (x & y) x) 2)", env()).unwrap(),
                   c_int(2));

        let env = env();
        eval("(def sum-list (fn* (xs) (if xs (+ (car xs) (sum-list (cdr xs))) 0)))", env.clone());
        eval("(def add (fn* (& xs) (sum-list xs)))", env.clone());

        assert_eq!(eval("(add 3 4 5)", env.clone()).unwrap(),
                   c_int(12));
    }
}
