#![feature(field_init_shorthand)]

extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate rustyline;

#[allow(dead_code)]
#[allow(unused_imports)]
mod data;
#[allow(dead_code)]
#[allow(unused_imports)]
mod lexer;
#[allow(dead_code)]
#[allow(unused_imports)]
mod parser;
#[allow(dead_code)]
#[allow(unused_imports)]
#[allow(unused_variables)]
mod env;
#[allow(dead_code)]
#[allow(unused_imports)]
#[allow(unused_variables)]
mod eval;
#[allow(unused_must_use)]
#[allow(dead_code)]
#[allow(unused_imports)]
mod core;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use data::{c_nil, AtomRet};
use env::{Env};
use eval::eval_str;


fn repl(env: Env) {
    let mut rl = Editor::<()>::new();
    if let Err(_) = rl.load_history("history.txt") {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                let _ = eval_str(line.as_str(), env.clone());
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
    use ::eval::eval_str;
    use ::core;
    use ::env::{Env, env_get};
    use ::data::{AtomRet, AtomError, c_int, c_symbol, c_list, c_nil};

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
    fn eval_ast_define() {
        let env = env();
        eval_str("(def foo 1)", env.clone());

        assert_eq!(env_get(&env, &c_symbol("foo".to_string())).unwrap(),
                   c_int(1));
    }

    #[test]
    fn eval_ast_lambda() {
        let env = env();
        assert_eq!(eval_str("((fn* (x) (- x 2)) 7)", env.clone()).unwrap(),
                   c_int(5));
    }

    #[test]
    fn eval_str_lambda_nested() {
        let env = env();
        eval_str("(def make-adder (fn* (x) (fn* (y) (+ x y))))", env.clone());
        eval_str("(def add-two (make-adder 2))", env.clone());

        assert_eq!(eval_str("(add-two 5)", env.clone()).unwrap(),
                   c_int(7));
    }

    #[test]
    fn eval_str_simple_if() {
        let env = env();

        assert_eq!(eval_str("(if nil (+ 2 2) (+ 1 1))", env.clone()).unwrap(),
                   c_int(2));
        assert_eq!(eval_str("(if 1 (+ 2 2) (+ 1 1))", env.clone()).unwrap(),
                   c_int(4));
    }

    #[test]
    fn eval_str_predicated() {
        assert_eq!(eval_str("(= 2 2)", env()).unwrap(),
                   c_int(1));

        let env = env();
        eval_str("(def foo 2)", env.clone());
        assert_eq!(eval_str("(= 2 foo)", env.clone()).unwrap(),
                   c_int(1));
    }

    #[test]
    fn eval_str_variadic_func() {
        assert_eq!(eval_str("((fn* (x y) y) 2 3)", env()).unwrap(),
                   c_int(3));

        assert_eq!(eval_str("((fn* (x & y) y) 1)", env()).unwrap(),
                   c_nil());
        assert_eq!(eval_str("((fn* (x & y) y) 1 2 3)", env()).unwrap(),
                   c_list(vec![c_int(2), c_int(3)]));
        assert_eq!(eval_str("((fn* (x & y) x) 2)", env()).unwrap(),
                   c_int(2));

        let env = env();
        eval_str("(def sum-list (fn* (xs) (if (= 0 (count xs)) 0 (+ (nth xs 0) (sum-list (rest xs))))))", env.clone());
        eval_str("(def add (fn* (& xs) (sum-list xs)))", env.clone());

        assert_eq!(eval_str("(add 3 4 5)", env.clone()).unwrap(),
                   c_int(12));
    }

    #[test]
    fn eval_str_macro() {
        let env = env();
        eval_str("(defmacro ignore (fn* (x) (list 'quote x))))", env.clone());

        assert_eq!(eval_str("(ignore foo)", env.clone()).expect("This shouldn't fail because foo is ignored"),
                   c_symbol("foo".to_string()));

        assert_eq!(eval_str("foo", env.clone()).unwrap_err(), AtomError::UndefinedSymbol("foo".to_string()));
    }


    #[test]
    fn eval_str_eval_str_backquote_splicing() {
        let env = env();

        assert_eq!(eval_str("(eval `(+ ~@(list 1 2 3)))", env.clone()).unwrap(), c_int(6));
    }





    #[test]
    fn eval_loop_recur() {
        let env = env();

        assert_eq!(eval_str("(loop (x 2 acc 0) (if (= x 1) acc (recur (- x 1) (+ acc x))))", env.clone()).unwrap(), c_int(2));
    }
}
