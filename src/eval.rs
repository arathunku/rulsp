use data::{AtomVal, AtomType, AtomRet, AtomError, c_int};


fn add(args: Vec<AtomVal>) -> AtomRet {
    let mut acc = 0;
    for arg in args.iter() {
        match **arg {
            AtomType::Int(i) => acc += i,
            _ => return Err(AtomError::ErrEval),
        }
    }

    Ok(c_int(acc))
}


fn eval_list(ast: AtomVal) -> AtomRet {
    match *ast {
        AtomType::List(ref seq) => {
            let op = match seq.get(0) {
                None => return Ok(ast.clone()),
                Some(op) => {
                    match **op {
                        AtomType::Symbol(ref v) => v,
                        _ => return Err(AtomError::ErrEval),
                    }
                }
            };
            let mut args: Vec<AtomVal> = vec![];
            for atom in seq[1..seq.len()].iter() {
                match eval(atom.clone()) {
                    Ok(result) => args.push(result),
                    Err(err) => return Err(err),
                }
            }

            match op.as_str() {
                "+" => add(args),
                _ => Err(AtomError::ErrEval),
            }
        }
        _ => unreachable!(),
    }
}

pub fn eval(ast: AtomVal) -> AtomRet {
    match *ast {
        AtomType::List(_) => eval_list(ast),
        _ => Ok(ast.clone()),
    }
}


pub fn print(v: AtomRet) -> String {
    match v {
        Ok(ref atom) => format!("{}", atom),
        Err(err) => format!("{}", err),
    }
}


#[cfg(test)]
mod tests {
    use super::{eval, print};
    use super::super::data::{c_symbol, c_int, c_list};
    use super::super::env::{c_env, Env};

    fn env() -> Env {
        c_env(None)
    }

    #[test]
    fn eval_symbol() {
        // TODO: error, because it's not defined in env
        assert_eq!("Test", print(eval(c_symbol("Test".to_string()))));
    }

    #[test]
    fn eval_int() {
        assert_eq!("2", print(eval(c_int(2))));
    }

    #[test]
    fn eval_list_error() {
        assert_eq!("Eval Error", print(eval(c_list(vec![c_int(1), c_int(2)]))));
    }

    #[test]
    fn eval_list_add() {
        assert_eq!("3",
                   print(eval(c_list(vec![c_symbol("+".to_string()), c_int(1), c_int(2)]))));
    }
}
