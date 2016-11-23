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
    match *safe_get(args.clone(), 1) {
        AtomType::List(ref seq) => {
            let mut list = seq.clone();
            list.insert(0, safe_get(args.clone(), 0));
            Ok(c_list(list))
        }
        ref v => Err(AtomError::InvalidType("List".to_string(), v.format(true))),
    }
}

fn list(args: Vec<AtomVal>) -> AtomRet {
    Ok(c_list(args))
}

fn count(args: Vec<AtomVal>) -> AtomRet {
    match *safe_get(args.clone(), 0) {
        AtomType::List(ref seq) => Ok(c_int(seq.len() as i64)),
        ref v => Err(AtomError::InvalidType("List".to_string(), v.format(true))),
    }
}


fn nth(args: Vec<AtomVal>) -> AtomRet {
    let ref list = *safe_get(args.clone(), 0);
    let ref el = *safe_get(args.clone(), 1);

    match (list, el) {
        (&AtomType::List(ref seq), &AtomType::Int(ref n)) => Ok(safe_get(seq.clone(), *n as usize)),
        (_, _) => Ok(c_nil()),
    }
}


fn rest(args: Vec<AtomVal>) -> AtomRet {
    match *safe_get(args.clone(), 0) {
        AtomType::List(ref seq) => {
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


fn format_args(args: Vec<AtomVal>, format: bool) -> String {
    args.iter()
        .map(|ref v| v.format(format))
        .collect::<Vec<_>>()
        .join(" ")
}

fn println(args: Vec<AtomVal>) -> AtomRet {
    println!("{}", format_args(args, false));
    Ok(c_nil())
}

fn print(args: Vec<AtomVal>) -> AtomRet {
    print!("{}", format_args(args, false));
    Ok(c_nil())
}

fn _println(args: Vec<AtomVal>) -> AtomRet {
    println!("{}", format_args(args, true));
    Ok(c_nil())
}

fn _print(args: Vec<AtomVal>) -> AtomRet {
    print!("{}", format_args(args, true));
    Ok(c_nil())
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
    env_set(&env, &c_symbol("nth".to_string()), c_func(nth));
    env_set(&env, &c_symbol("rest".to_string()), c_func(rest));
    env_set(&env, &c_symbol("count".to_string()), c_func(count));

    // predicates
    env_set(&env, &c_symbol("=".to_string()), c_func(partialeq));
    // env_set(&env, &c_symbol("=".to_string()), c_func(partialeq));

    env
}
