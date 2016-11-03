use data::{AtomVal, AtomType, AtomRet, AtomError, c_int, c_nil};
use std::fmt;

fn int_op<F>(f: F, args: Vec<AtomVal>) -> AtomRet
    where F: FnOnce(Vec<i64>) -> i64
{

    let mut ints = vec![];
    for arg in args.iter() {
        let value = try!(eval(arg.clone()));

        match *value {
            AtomType::Int(i) => ints.push(i),
            ref v => return Err(AtomError::InvalidType("Int".to_string(), v.format(true))),
        }
    }

    Result::Ok(c_int(f(ints)))
}

fn add(args: Vec<AtomVal>) -> AtomRet {
    int_op(|values| values.iter().fold(0i64, |acc, v| acc + v), args)
}

fn sub(args: Vec<AtomVal>) -> AtomRet {
    int_op(|values| values.iter().fold(0i64, |acc, v| acc - v), args)
}

fn mul(args: Vec<AtomVal>) -> AtomRet {
    int_op(|values| values.iter().fold(0i64, |acc, v| acc * v), args)
}

fn div(args: Vec<AtomVal>) -> AtomRet {
    int_op(|values| values.iter().fold(0i64, |acc, v| acc / v), args)
}

fn quote(args: Vec<AtomVal>) -> AtomRet {
    Result::Ok(args.get(0).map(|v| v.clone()).unwrap_or(c_nil()))
}

fn eval_list(ast: AtomVal) -> AtomRet {
    match *ast {
        AtomType::List(ref seq) => {
            let op = match seq.get(0) {
                None => return Ok(ast.clone()),
                Some(op) => {
                    match **op {
                        AtomType::Symbol(ref v) => v,
                        ref v => {
                            return Err(AtomError::InvalidType("Symbol".to_string(), v.format(true)))
                        }
                    }
                }
            };

            let args = seq[1..seq.len()].iter().map(|v| v.clone()).collect();

            match op.as_str() {
                "+" => add(args),
                "-" => sub(args),
                "*" => mul(args),
                "/" => div(args),
                "quote" => quote(args),
                op => Err(AtomError::InvalidOperation(op.to_string())),
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


    fn env() {
        // c_env(None)
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
    fn eval_list_invalid_type_because_operation_is_int() {
        match eval(c_list(vec![c_int(1), c_int(2)])) {
            Err(AtomError::InvalidType(_, _)) => {}
            Err(_) => unreachable!(),
            Ok(_) => unreachable!(),
        }
    }

    #[test]
    fn eval_list_invalid_operation() {
        match eval(c_list(vec![c_symbol("undefined".to_string()), c_int(2)])) {
            Err(AtomError::InvalidOperation(_)) => {}
            Err(_) => unreachable!(),
            Ok(_) => unreachable!(),
        }
    }

    #[test]
    fn eval_list_add() {
        assert_eq!("3",
                   print(eval(c_list(vec![c_symbol("+".to_string()), c_int(1), c_int(2)]))));
    }
}
