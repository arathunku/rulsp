use data::{AtomVal, AtomType, AtomRet, AtomError, c_int, c_nil, c_list, c_afunc, c_symbol,
           c_macro, AFuncData};
use env::{c_env, env_set, env_get, Env};
use std::fmt;
use lexer::{lex, format_tokens};
use parser::Parser;

fn safe_get(args: &Vec<AtomVal>, index: usize) -> AtomVal {
    args.get(index).map(|v| v.clone()).unwrap_or(c_nil())
}

fn op_quote(args: &Vec<AtomVal>) -> AtomRet {
    Result::Ok(safe_get(args, 1))
}

fn op_def(args: &Vec<AtomVal>, env: Env) -> AtomRet {
    let name = try!(args.get(1)
        .map(|v| v.clone())
        .ok_or(AtomError::InvalidType("Symbol as name of def".to_string(), "nil".to_string())));

    match *name {
        AtomType::Symbol(_) => {
            let value = eval(safe_get(args, 2), env.clone())?;

            let _ = env_set(&env, &name, value);
            Result::Ok(c_symbol(name.to_string()))
        }
        ref v => {
            return Err(AtomError::InvalidType("Symbol as name of def".to_string(), v.format(true)));
        }
    }
}

fn op_lambda(args: &Vec<AtomVal>, env: Env) -> AtomRet {
    Ok(c_afunc(env, safe_get(args, 1), safe_get(args, 2)))
}

fn op_macro(args: &Vec<AtomVal>, env: Env) -> AtomRet {
    let result = eval(safe_get(args, 2), env.clone())?;
    match *result {
        AtomType::AFunc(ref fd) => {
            op_def(&vec![c_nil(), safe_get(args, 1), c_macro(&fd)], env.clone())
        }
        _ => unreachable!(),
    }
}

fn is_macro_call(ast: AtomVal, env: Env) -> bool {
    match *ast {
        AtomType::List(ref args) => {
            if let Some(value) = env_get(&env, &args[0]) {
                match *value {
                    AtomType::AFunc(ref fd) => fd.is_macro,
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

fn op_macroexpand(mut ast: AtomVal, env: Env) -> AtomRet {
    // println!("IS MACRO CALL: {:?}", ast);
    while is_macro_call(ast.clone(), env.clone()) {
        let new_ast = ast.clone();

        let args = match *new_ast {
            AtomType::List(ref args) => args,
            _ => break,
        };

        if let Some(f) = env_get(&env, &args[0]) {
            ast = f.apply(args[1..].to_vec())?;
        } else {
            break;
        }
    }

    // println!("EXPANDED AST: {:?}", ast);
    Ok(ast)
}

fn op_if(args: &Vec<AtomVal>, env: Env) -> AtomRet {
    let result = eval(safe_get(args, 1), env.clone())?;
    match *result {
        AtomType::Nil => eval(safe_get(args, 3), env.clone()),
        _ => eval(safe_get(args, 2), env.clone()),
    }
}

pub fn eval_exp(ast: AtomVal, env: Env) -> AtomRet {
    match *ast {
        AtomType::List(ref args) => {
            let op_name = match args.get(0) {
                None => return Ok(ast.clone()),
                Some(op) => {
                    match **op {
                        AtomType::Symbol(ref v) => v.as_str(),
                        _ => "__func__",
                    }
                }
            };

            match op_name {
                "quote" => op_quote(args),
                "p_env" => {
                    println!("{:?}", env);
                    Ok(c_nil())
                }
                "def" => op_def(args, env),
                "if" => op_if(args, env),
                "fn*" => op_lambda(args, env),
                "defmacro" => op_macro(args, env),
                "eval" => eval(eval(safe_get(args, 1), env.clone())?, env),
                "do" => {
                    let evaled_args = eval_ast(c_list(args[1..].to_vec()), env)?;
                    match *evaled_args {
                        AtomType::List(ref args) => Ok(args.last().unwrap_or(&c_nil()).clone()),
                        _ => Ok(c_nil()),
                    }
                }
                "macroexpand" => {
                    op_macroexpand(eval_exp(safe_get(args, 1), env.clone())?, env.clone())
                }
                // Some function call with evaled arguments
                _ => {
                    let evaled_args = eval_ast(ast.clone(), env)?;
                    let args = match *evaled_args {
                        AtomType::List(ref args) => args,
                        _ => return Err(AtomError::InvalidOperation(op_name.to_string())),
                    };

                    let subject_func = &args[0].clone();
                    subject_func.apply(args[1..].to_vec())
                }

            }
        }
        _ => unreachable!(),
    }
}

fn eval_ast(ast: AtomVal, env: Env) -> AtomRet {
    match *ast {
        AtomType::List(ref args) => {
            let mut evaled_args = vec![];

            for arg in args {
                evaled_args.push(eval(arg.clone(), env.clone())?);
            }

            Ok(c_list(evaled_args))
        }
        AtomType::Symbol(ref name) => {
            if let Some(atom) = env_get(&env, &ast) {
                Ok(atom.clone())
            } else {
                Err(AtomError::UndefinedSymbol(name.to_string()))
            }
        }
        _ => Ok(ast.clone()),
    }
}

pub fn eval(ast: AtomVal, env: Env) -> AtomRet {
    match *ast {
        AtomType::List(_) => {
            let ast = op_macroexpand(ast, env.clone())?;
            match *ast {
                AtomType::List(_) => eval_exp(ast, env),
                _ => eval_ast(ast, env),
            }
        }
        _ => eval_ast(ast, env),
    }
}

pub fn eval_str(str: &str, env: Env) -> AtomRet {
    let tokens = lex(str);
    match tokens {
        Ok(ref tokens) => {
            let prefix = format!("exp: {} -> lex: {}", str, format_tokens(tokens));
            let parser = Parser::new(tokens);
            match parser.start() {
                Ok(ast) => {
                    // print!("{} -> ast: {}\n", prefix, ast.format(true));

                    match eval(ast, env.clone()) {
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



#[cfg(test)]
mod tests {
    use super::eval;
    use super::super::data::{c_symbol, c_int, c_list, AtomRet, AtomError};
    use super::super::env::{c_env, Env};

    pub fn print(v: AtomRet) -> String {
        match v {
            Ok(ref atom) => format!("{}", atom),
            Err(err) => format!("{}", err),
        }
    }


    fn env() -> Env {
        super::super::core::build()
    }

    #[test]
    fn eval_symbol() {
        eval(c_symbol("Test".to_string()), env()).unwrap_err();
    }

    #[test]
    fn eval_int() {
        assert_eq!("2", print(eval(c_int(2), env())));
    }

    #[test]
    fn eval_list_invalid_type_because_operation_is_int() {
        match eval(c_list(vec![c_int(1), c_int(2)]), env()) {
            Err(AtomError::InvalidType(_, _)) => {}
            Err(_) => unreachable!(),
            Ok(_) => unreachable!(),
        }
    }

    #[test]
    fn eval_list_invalid_operation() {
        match eval(c_list(vec![c_symbol("undefined".to_string()), c_int(2)]),
                   env()) {
            Err(AtomError::UndefinedSymbol(_)) => {}
            Err(_) => unreachable!(),
            Ok(_) => unreachable!(),
        }
    }

    #[test]
    fn eval_list_add() {
        assert_eq!("3",
                   print(eval(c_list(vec![c_symbol("+".to_string()), c_int(1), c_int(2)]),
                              env())));
    }

    #[test]
    fn eval_list_div() {
        assert_eq!("2",
                   print(eval(c_list(vec![c_symbol("/".to_string()), c_int(4), c_int(2)]),
                              env())));
    }
}
