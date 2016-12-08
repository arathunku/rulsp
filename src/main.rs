#![feature(field_init_shorthand)]
#![feature(test)]

#![feature(alloc_system)]
extern crate alloc_system;

extern crate test;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate rustyline;
extern crate fnv;
#[macro_use]
extern crate log;
extern crate env_logger;

mod data;
mod lexer;
mod parser;
mod env;
mod eval;
mod core;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use env::{Env};
use eval::eval_str;

#[allow(dead_code)]
fn repl(env: &Env) {
    let mut rl = Editor::<()>::new();
    if let Err(_) = rl.load_history("history.txt") {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                let result = eval_str(line.as_str(), env);
                match result {
                    Ok(result) => println!(">> {}", result),
                    Err(err) => println!(">> {:?}", err)
                };
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRLs-D");
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

#[allow(unused_must_use)]
fn count(n: String, env: &Env) {
    eval_str("(def count-1 (fn* (n) (loop (n n acc 0) (if (= n 0) acc (recur (- n 1) (+ acc 1))))))", env);
    eval_str(&format!("(count-1 {})", n), env);
}

fn main() {
    env_logger::init().unwrap();
    let env = core::build();

    match std::env::args().nth(1) {
        Some(value) => {
            if "repl" == value  {
                repl(&env);
            } else {
                count(value, &env)
            }
        },
        None => {
            println!("Pass repl or any number as a first param to count")
        }
    };
}

#[allow(unused_must_use)]
#[cfg(test)]
mod tests {
    use ::eval::eval_str;
    use ::core;
    use ::env::{Env, env_get};
    use ::data::{AtomError, c_int, c_symbol, c_list, c_nil};

    fn env() -> Env {
        core::build()
    }

    #[test]
    fn eval_ast_define() {
        let ref env = env();
        eval_str("(def foo 1)", env);

        assert_eq!(env_get(&env, &c_symbol("foo".to_string())).unwrap(),
                   c_int(1));
    }

    #[test]
    fn eval_ast_lambda() {
        let ref env = env();
        assert_eq!(eval_str("((fn* (x) (- x 2)) 7)", env).unwrap(),
                   c_int(5));
    }

    #[test]
    fn eval_str_lambda_nested() {
        let ref env = env();
        eval_str("(def make-adder (fn* (x) (fn* (y) (+ x y))))", env);
        eval_str("(def add-two (make-adder 2))", env);

        assert_eq!(eval_str("(add-two 5)", env).unwrap(),
                   c_int(7));
    }

    #[test]
    fn eval_str_simple_if() {
        let ref env = env();

        assert_eq!(eval_str("(if nil (+ 2 2) (+ 1 1))", env).unwrap(),
                   c_int(2));
        assert_eq!(eval_str("(if 1 (+ 2 2) (+ 1 1))", env).unwrap(),
                   c_int(4));
    }

    #[test]
    fn eval_str_predicated() {
        assert_eq!(eval_str("(= 2 2)", &env()).unwrap(),
                   c_int(1));

        let env = env();
        eval_str("(def foo 2)", &env);
        assert_eq!(eval_str("(= 2 foo)", &env).unwrap(),
                   c_int(1));
    }

    #[test]
    fn eval_str_variadic_func() {
        assert_eq!(eval_str("((fn* (x y) y) 2 3)", &env()).unwrap(),
                   c_int(3));

        assert_eq!(eval_str("((fn* (x & y) y) 1)", &env()).unwrap(),
                   c_nil());
        assert_eq!(eval_str("((fn* (x & y) y) 1 2 3)", &env()).unwrap(),
                   c_list(&[c_int(2), c_int(3)]));
        assert_eq!(eval_str("((fn* (x & y) x) 2)", &env()).unwrap(),
                   c_int(2));

        let ref env = env();
        eval_str("(def sum-list (fn* (xs) (if (= 0 (count xs)) 0 (+ (nth xs 0) (sum-list (rest xs))))))", env);
        eval_str("(def add (fn* (& xs) (sum-list xs)))", env);

        assert_eq!(eval_str("(add 3 4 5)", env).unwrap(),
                   c_int(12));
    }

    #[test]
    fn eval_str_macro() {
        let env = env();
        eval_str("(defmacro ignore (fn* (x) (list 'quote x))))", &env);

        assert_eq!(eval_str("(ignore foo)", &env).expect("This shouldn't fail because foo is ignored"),
                   c_symbol("foo".to_string()));

        assert_eq!(eval_str("foo", &env).unwrap_err(), AtomError::UndefinedSymbol("foo".to_string()));
    }


    #[test]
    fn eval_str_eval_str_backquote_splicing() {
        let env = env();

        assert_eq!(eval_str("(eval `(+ ~@(list 1 2 3)))", &env).unwrap(), c_int(6));
    }


    #[test]
    fn eval_loop_recur() {
        let env = env();

        assert_eq!(eval_str("(loop (x 2 acc 0) (if (= x 1) acc (recur (- x 1) (+ acc x))))", &env).unwrap(), c_int(2));
    }


    use test::Bencher;

    #[bench]
    fn bench_counting(b: &mut Bencher) {
        let env = env();
        eval_str("(def count-1 (fn* (n) (loop (n n acc 0) (if (= n 0) acc (recur (- n 1) (+ acc 1))))))", &env);

        b.iter(|| {
            eval_str("(count-1 1000)", &env);
        });
    }
}
