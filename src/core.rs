use env::{c_env, env_set, env_get, Env};
use data::{AtomVal, AtomType, AtomRet, AtomError, c_int, c_nil, c_list, c_symbol, c_func, c_vec};

fn safe_get(args: Vec<AtomVal>, index: usize) -> AtomVal {
    args.get(index).map(|v| v.clone()).unwrap_or(c_nil())
}

fn int_op<F>(f: F, args: Vec<AtomVal>) -> AtomRet
    where F: FnOnce(Vec<i64>) -> i64
{

    let mut ints = vec![];
    for arg in args.iter() {
        match **arg {
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
    int_op(|values| {
               values.iter()
                   .fold(None, |acc, x| acc.map_or(Some(*x), |y| Some(y - x)))
                   .unwrap_or(0)
           },
           args)
}

fn mul(args: Vec<AtomVal>) -> AtomRet {
    int_op(|values| values.iter().fold(1i64, |acc, v| acc * v), args)
}

fn div(args: Vec<AtomVal>) -> AtomRet {
    int_op(|values| {
               values.iter()
                   .fold(None, |acc, x| acc.map_or(Some(*x), |y| Some(y / x)))
                   .unwrap_or(0)
           },
           args)
}

fn cons(args: Vec<AtomVal>) -> AtomRet {
    Ok(c_list(args))
}

fn car(args: Vec<AtomVal>) -> AtomRet {
    match *safe_get(args.clone(), 0) {
        AtomType::List(ref seq) => Ok(safe_get(seq.clone(), 0)),
        _ => Ok(c_nil()),
    }
}

fn cdr(args: Vec<AtomVal>) -> AtomRet {
    match *safe_get(args.clone(), 0) {
        AtomType::List(ref seq) => {
            if seq.len() == 2 {
                Ok(safe_get(seq.clone(), 1))
            } else {
                let cdr = seq[1..seq.len()].iter().map(|v| v.clone()).collect::<Vec<AtomVal>>();
                Ok(c_list(cdr))
            }
        }
        _ => Ok(c_nil()),
    }
}



pub fn build() -> Env {
    let env = c_env(None);

    env_set(&env, &c_symbol("+".to_string()), c_func(add));
    env_set(&env, &c_symbol("-".to_string()), c_func(sub));
    env_set(&env, &c_symbol("*".to_string()), c_func(mul));
    env_set(&env, &c_symbol("/".to_string()), c_func(div));
    env_set(&env, &c_symbol("cons".to_string()), c_func(cons));
    env_set(&env, &c_symbol("car".to_string()), c_func(car));
    env_set(&env, &c_symbol("cdr".to_string()), c_func(cdr));

    env
}
