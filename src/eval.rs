use data::{AtomVal, AtomType, AtomRet, AtomError, c_int, c_nil, c_list};
use env::{c_env, env_set, env_get, Env};
use std::fmt;

fn safe_get(args: Vec<AtomVal>, index: usize) -> AtomVal {
    args.get(index).map(|v| v.clone()).unwrap_or(c_nil())
}

fn quote(args: Vec<AtomVal>) -> AtomRet {
    Result::Ok(safe_get(args, 0))
}

fn def(args: Vec<AtomVal>, env: Env) -> AtomRet {
    let name = try!(args.get(0)
        .map(|v| v.clone())
        .ok_or(AtomError::InvalidType("Symbol as name of def".to_string(), "nil".to_string())));

    match *name {
        AtomType::Symbol(_) => {
            let value = try!(eval(safe_get(args, 1).clone(), c_env(Some(env.clone()))));

            env_set(&env.clone(), &name, value);
            Result::Ok(c_nil())
        }
        ref v => {
            return Err(AtomError::InvalidType("Symbol as name of def".to_string(), v.format(true)));
        }
    }
}
fn eval_each(args: Vec<AtomVal>, env: Env) -> Result<Vec<AtomVal>, AtomError> {
    let mut evaled_args = vec![];
    for arg in args {
        evaled_args.push(eval(arg.clone(), env.clone())?);
    }

    Ok(evaled_args)
}

fn eval_list(ast: AtomVal, env: Env) -> AtomRet {
    match *ast {
        AtomType::List(ref seq) => {
            let (op, opName) = match seq.get(0) {
                None => return Ok(ast.clone()),
                Some(op) => {
                    match **op {
                        AtomType::Symbol(ref v) => (op, v.as_str()),
                        ref v => {
                            return Err(AtomError::InvalidType("Symbol".to_string(), v.format(true)))
                        }
                    }
                }
            };

            let args = seq[1..seq.len()].iter().map(|v| v.clone()).collect();

            match opName {
                "quote" => quote(args),
                "def" => def(args, env),
                // Some function call with evaled arguments
                _ => {
                    let evaled_args = eval_each(args, env.clone())?;
                    if let Some(value) = env_get(&env, op) {
                        match *value {
                            AtomType::Func(f) => f(evaled_args),
                            _ => Err(AtomError::InvalidOperation(opName.to_string())),
                        }
                    } else {
                        Err(AtomError::InvalidOperation(opName.to_string()))
                    }
                }

            }
        }
        _ => unreachable!(),
    }
}

pub fn eval(ast: AtomVal, env: Env) -> AtomRet {
    match *ast {
        AtomType::List(_) => eval_list(ast.clone(), env),
        AtomType::Symbol(ref name) => {
            if let Some(atom) = env_get(&env, &ast.clone()) {
                Ok(atom.clone())
            } else {
                Err(AtomError::UndefinedSymbol(name.to_string()))
            }
        }
        _ => Ok(ast.clone()),
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
            Err(AtomError::InvalidOperation(_)) => {}
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
