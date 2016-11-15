use env::{c_env, env_set, env_get, Env};
use data::{AtomVal, AtomType, AtomRet, AtomError, c_int, c_nil, c_list, c_symbol, c_func};

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
    int_op(|values| values.iter().fold(0i64, |acc, v| acc - v), args)
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

pub fn build() -> Env {
    let env = c_env(None);

    env_set(&env, &c_symbol("+".to_string()), c_func(add));
    env_set(&env, &c_symbol("-".to_string()), c_func(sub));
    env_set(&env, &c_symbol("*".to_string()), c_func(mul));
    env_set(&env, &c_symbol("/".to_string()), c_func(div));

    env
}
