use data::{AtomVal, AtomType, AtomRet, AtomError, c_nil, c_list, c_afunc, c_symbol, c_macro};
use env::{env_set, env_get, env_bind, Env};
use lexer::lex;
use parser::Parser;

fn safe_get(args: &[AtomVal], index: usize) -> AtomVal {
    args.get(index).unwrap_or(&c_nil()).clone()
}

fn op_quote(args: &[AtomVal]) -> AtomRet {
    Result::Ok(safe_get(args, 1))
}

fn op_def(args: &[AtomVal], env: &Env) -> AtomRet {
    trace!("action=op_def args={:?}", args);
    let name_atom = safe_get(args, 1);
    let name = name_atom.get_symbol()?;
    let value = eval(safe_get(args, 2), env)?;

    let _ = env_set(&env, &name_atom, value);
    Result::Ok(c_symbol(name.to_string()))
}

fn op_lambda(args: &[AtomVal], env: &Env) -> AtomRet {
    Ok(c_afunc(env.clone(), safe_get(args, 1), safe_get(args, 2)))
}

fn op_macro(args: &[AtomVal], env: &Env) -> AtomRet {
    let result = eval(safe_get(args, 2), env)?;
    match *result {
        AtomType::AFunc(ref fd) => op_def(&vec![c_nil(), safe_get(args, 1), c_macro(&fd)], env),
        _ => unreachable!(),
    }
}

fn is_macro_call(ast: AtomVal, env: &Env) -> bool {
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

fn op_macroexpand(mut ast: AtomVal, env: &Env) -> AtomRet {
    // println!("IS MACRO CALL: {:?}", ast);
    while is_macro_call(ast.clone(), env) {
        let new_ast = ast.clone();

        let args = match *new_ast {
            AtomType::List(ref args) => args,
            _ => break,
        };

        if let Some(f) = env_get(&env, &args[0]) {
            ast = f.apply(&args[1..])?;
        } else {
            break;
        }
    }

    // println!("EXPANDED AST: {:?}", ast);
    Ok(ast)
}

fn op_if(args: &[AtomVal], env: &Env) -> AtomRet {
    let result = eval(safe_get(args, 1), env)?;
    match *result {
        AtomType::Nil => eval(safe_get(args, 3), env),
        _ => eval(safe_get(args, 2), env),
    }
}

// [loop (args...) (body)]
#[allow(unused_assignments)]
fn op_loop(args: &[AtomVal], env: &Env) -> AtomRet {
    trace!("fn=op_loop args={:?}", args);

    let body = safe_get(args, 2);
    let _loop_args = safe_get(args, 1);
    let loop_args = _loop_args.get_list()?;

    if loop_args.len() % 2 == 1 {
        return Err(AtomError::InvalidArgument("Loop is missing value for one of the \
                                               param"
            .to_string()));
    }

    let arguments_chunks = loop_args.chunks(2);
    let mut arguments_names: Vec<AtomVal> = Vec::with_capacity(arguments_chunks.len());
    let mut values_for_eval: Vec<AtomVal> = Vec::with_capacity(arguments_chunks.len());

    for chunk in arguments_chunks {
        arguments_names.push(chunk[0].clone());
        values_for_eval.push(chunk[1].clone());
    }
    let mut arguments_values = eval_list_elements(&values_for_eval, &env)?;

    let mut result = None;
    let recur_symbol = c_symbol("recur".to_string());
    loop {
        env_bind(&env, &arguments_names, &arguments_values)?;
        result = Some(eval(body.clone(), &env)?);

        if let Some(ref result) = result {
            match result.get_list() {
                Ok(list) => {
                    if safe_get(list, 0) == recur_symbol {
                        arguments_values = eval_list_elements(&list[1..], &env)?;
                    } else {
                        break;
                    }
                }
                _ => {
                    break;
                }
            }
        }

    }

    Ok(result.unwrap_or(c_nil()))
}

pub fn eval_exp(ast: AtomVal, env: &Env) -> AtomRet {
    let args = ast.get_list()?;
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
        "loop" => op_loop(args, env),
        "recur" => Ok(ast.clone()),
        "defmacro" => op_macro(args, env),
        "eval" => eval(eval(safe_get(args, 1), env)?, env),
        "do" => {
            let evaled_args = eval_ast(c_list(&args[1..]), env)?;
            match evaled_args.get_list() {
                Ok(args) => Ok(args.last().unwrap_or(&c_nil()).clone()),
                _ => Ok(c_nil()),
            }
        }
        "macroexpand" => op_macroexpand(eval_exp(safe_get(args, 1), env)?, env),
        // Some function call with evaled arguments
        _ => {
            let evaled_args = eval_ast(ast.clone(), env)?;
            let args = match evaled_args.get_list() {
                Ok(args) => args,
                _ => return Err(AtomError::InvalidOperation(op_name.to_string())),
            };

            trace!("fn=eval_exp op_name={} args={:?}",
                   op_name,
                   args[1..].to_vec());
            let subject_func = &args[0].clone();
            subject_func.apply(&args[1..])
        }

    }
}

fn eval_list_elements(list: &[AtomVal], env: &Env) -> Result<Vec<AtomVal>, AtomError> {
    let mut evaled_elements = Vec::with_capacity(list.len());

    for element in list {
        evaled_elements.push(eval(element.clone(), env)?);
    }

    Ok(evaled_elements)
}

fn eval_ast(ast: AtomVal, env: &Env) -> AtomRet {
    trace!("fn=eval_ast ast={}", ast.format(true));

    match *ast {
        AtomType::List(ref args) => Ok(c_list(&eval_list_elements(args, env)?)),
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

pub fn eval(ast: AtomVal, env: &Env) -> AtomRet {
    match *ast {
        AtomType::List(_) => {
            let ast = op_macroexpand(ast, env)?;
            match *ast {
                AtomType::List(_) => eval_exp(ast, env),
                _ => eval_ast(ast, env),
            }
        }
        _ => eval_ast(ast, env),
    }
}

pub fn eval_str(str: &str, env: &Env) -> AtomRet {
    let tokens = lex(str);
    match tokens {
        Ok(ref tokens) => {
            // let prefix = format!("exp: {} -> lex: {}", str, format_tokens(tokens));
            let parser = Parser::new(tokens);
            match parser.start() {
                Ok(ast) => {
                    // print!("{} -> ast: {}\n", prefix, ast.format(true));

                    match eval(ast, env) {
                        Ok(result) => {
                            return Ok(result);
                        }
                        Err(err) => {
                            println!("=> {}", err);
                            return Err(err);
                        }
                    }

                }
                Err(err) => {
                    println!("{} -> ast: {}", str, err);
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
    use data::{c_symbol, c_int, c_list, AtomRet, AtomError};
    use env::Env;

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
        eval(c_symbol("Test".to_string()), &env()).unwrap_err();
    }

    #[test]
    fn eval_int() {
        assert_eq!("2", print(eval(c_int(2), &env())));
    }

    #[test]
    fn eval_list_invalid_type_because_operation_is_int() {
        match eval(c_list(&[c_int(1), c_int(2)]), &env()) {
            Err(AtomError::InvalidType(_, _)) => {}
            Err(_) => unreachable!(),
            Ok(_) => unreachable!(),
        }
    }

    #[test]
    fn eval_list_invalid_operation() {
        match eval(c_list(&[c_symbol("undefined".to_string()), c_int(2)]),
                   &env()) {
            Err(AtomError::UndefinedSymbol(_)) => {}
            Err(_) => unreachable!(),
            Ok(_) => unreachable!(),
        }
    }

    #[test]
    fn eval_list_add() {
        assert_eq!("3",
                   print(eval(c_list(&[c_symbol("+".to_string()), c_int(1), c_int(2)]),
                              &env())));
    }

    #[test]
    fn eval_list_div() {
        assert_eq!("2",
                   print(eval(c_list(&[c_symbol("/".to_string()), c_int(4), c_int(2)]),
                              &env())));
    }
}
