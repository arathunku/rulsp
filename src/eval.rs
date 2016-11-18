use data::{AtomVal, AtomType, AtomRet, AtomError, c_int, c_nil, c_list, c_afunc, c_symbol,
           AFuncData};
use env::{c_env, env_set, env_get, Env};
use std::fmt;

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

            env_set(&env, &name, value);
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

fn op_if(args: &Vec<AtomVal>, env: Env) -> AtomRet {
    let result = eval(safe_get(args, 1), env.clone())?;
    match *result {
        AtomType::Nil => eval(safe_get(args, 3), env.clone()),
        _ => eval(safe_get(args, 2), env.clone()),
    }
}

fn eval_each(args: &Vec<AtomVal>, env: Env) -> Result<Vec<AtomVal>, AtomError> {
    let mut evaled_args = vec![];
    for arg in args {
        evaled_args.push(eval(arg.clone(), env.clone())?);
    }

    Ok(evaled_args)
}

pub fn eval_exp(ast: AtomVal, env: Env) -> AtomRet {
    match *ast {
        AtomType::List(ref args) => {
            let opName = match args.get(0) {
                None => return Ok(ast.clone()),
                Some(op) => {
                    match **op {
                        AtomType::Symbol(ref v) => v.as_str(),
                        _ => "__func__",
                    }
                }
            };

            match opName {
                "quote" => op_quote(args),
                "def" => op_def(args, env),
                "if" => op_if(args, env),
                "fn*" => op_lambda(args, env),
                // Some function call with evaled arguments
                _ => {
                    let evaled_args = eval_ast(ast.clone(), env)?;
                    let args = match *evaled_args {
                        AtomType::List(ref args) => args,
                        _ => return Err(AtomError::InvalidOperation(opName.to_string())),
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
        AtomType::List(ref seq) => {
            let args = eval_each(seq, env)?;
            Ok(c_list(args))
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
        AtomType::List(_) => eval_exp(ast, env),
        _ => eval_ast(ast, env),
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
