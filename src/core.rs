use std::fs::File;
use std::io::prelude::*;

use env::{c_env, env_set, env_get, Env};
use data::{AtomVal, AtomType, AtomRet, AtomError, c_int, c_nil, c_list, c_symbol, c_func, c_vec};
use eval::eval_str;
use parser::Parser;


fn safe_get(args: Vec<AtomVal>, index: usize) -> AtomVal {
    args.get(index).map(|v| v.clone()).unwrap_or(c_nil())
}

fn int_op<F>(f: F, args: Vec<AtomVal>) -> AtomRet
    where F: FnOnce(Vec<i64>) -> i64
{

    let ints: Result<Vec<i64>, _> = args.iter().map(|v| v.get_int()).collect();
    Result::Ok(c_int(f(ints?)))
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
    let mut list = safe_get(args.clone(), 1).get_list()?.clone();
    list.insert(0, safe_get(args.clone(), 0));
    Ok(c_list(list))
}

fn list(args: Vec<AtomVal>) -> AtomRet {
    Ok(c_list(args))
}

fn is_list(args: Vec<AtomVal>) -> AtomRet {
    match *safe_get(args.clone(), 0) {
        AtomType::List(_) => Ok(c_int(1)),
        _ => Ok(c_nil()),
    }
}

fn is_nil(args: Vec<AtomVal>) -> AtomRet {
    match *safe_get(args.clone(), 0) {
        AtomType::Nil => Ok(c_int(1)),
        _ => Ok(c_nil()),
    }
}


fn count(args: Vec<AtomVal>) -> AtomRet {
    Ok(c_int(safe_get(args.clone(), 0).get_list()?.len() as i64))
}


fn nth(args: Vec<AtomVal>) -> AtomRet {
    trace!("action=nth args={:?}", args);
    let n = safe_get(args.clone(), 1).get_int().unwrap_or(0);

    Ok(safe_get(safe_get(args.clone(), 0).get_list()?.clone(), n as usize))
}


fn rest(args: Vec<AtomVal>) -> AtomRet {
    match safe_get(args.clone(), 0).get_list() {
        Ok(seq) => {
            if seq.len() > 0 {
                Ok(c_list(seq[1..seq.len()].iter().map(|v| v.clone()).collect::<Vec<AtomVal>>()))
            } else {
                Ok(c_list(vec![]))
            }
        }
        _ => Ok(c_nil()),
    }
}

fn partialeq(args: Vec<AtomVal>) -> AtomRet {
    let mut output = c_int(1);
    for (i, arg) in args.iter().enumerate() {
        let next_arg = args.get(i + 1);
        if next_arg.is_some() {
            if next_arg.unwrap() != arg {
                output = c_nil();
            };
        }
    }

    Ok(output)
}


fn format_args(args: &Vec<AtomVal>, format: bool) -> String {
    args.iter()
        .map(|ref v| v.format(format))
        .collect::<Vec<_>>()
        .join(" ")
}

fn println(args: Vec<AtomVal>) -> AtomRet {
    println!("{}", format_args(&args, false));
    Ok(safe_get(args, 0))
}

fn print(args: Vec<AtomVal>) -> AtomRet {
    print!("{}", format_args(&args, false));
    Ok(safe_get(args, 0))
}

fn _println(args: Vec<AtomVal>) -> AtomRet {
    println!("{}", format_args(&args, true));
    Ok(safe_get(args, 0))
}

fn _print(args: Vec<AtomVal>) -> AtomRet {
    print!("{}", format_args(&args, true));
    Ok(safe_get(args, 0))
}


pub fn build() -> Env {
    let env = c_env(None);

    env_set(&env, &c_symbol("print".to_string()), c_func(print));
    env_set(&env, &c_symbol("println".to_string()), c_func(println));
    env_set(&env, &c_symbol("_print".to_string()), c_func(_print));
    env_set(&env, &c_symbol("_println".to_string()), c_func(_println));
    env_set(&env, &c_symbol("+".to_string()), c_func(add));
    env_set(&env, &c_symbol("-".to_string()), c_func(sub));
    env_set(&env, &c_symbol("*".to_string()), c_func(mul));
    env_set(&env, &c_symbol("/".to_string()), c_func(div));
    env_set(&env, &c_symbol("cons".to_string()), c_func(cons));
    env_set(&env, &c_symbol("list".to_string()), c_func(list));
    env_set(&env, &c_symbol("list?".to_string()), c_func(is_list));
    env_set(&env, &c_symbol("nil?".to_string()), c_func(is_nil));
    env_set(&env, &c_symbol("nth".to_string()), c_func(nth));
    env_set(&env, &c_symbol("rest".to_string()), c_func(rest));
    env_set(&env, &c_symbol("count".to_string()), c_func(count));

    // predicates
    env_set(&env, &c_symbol("=".to_string()), c_func(partialeq));
    // env_set(&env, &c_symbol("=".to_string()), c_func(partialeq));


    let mut f = File::open("src/core.clrs").expect("core.clrs has to be openable");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("Couldn't read core.clrs");

    eval_str(s.as_str(), env.clone()).expect("Problem loading core.clrs into ENV");

    env
}

#[cfg(test)]
mod tests {
    use super::add;
    use data::c_int;

    use test::Bencher;
    use std;

    #[bench]
    fn bench_adding(b: &mut Bencher) {
        let args = vec![c_int(1), c_int(1)];

        b.iter(|| add(args.clone()));
    }

}
