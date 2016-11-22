use super::data::{AtomVal, AtomType, AtomRet, AtomError, c_nil};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

#[derive(PartialEq)]
pub struct EnvType {
    parent: Option<Env>,
    data: HashMap<String, AtomVal>,
}

pub type Env = Rc<RefCell<EnvType>>;


impl fmt::Display for EnvType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str: Vec<String> = vec![];

        for (ref key, ref value) in self.data.iter() {
            str.push(format!("{} {}", key, value));
        }

        str.sort();

        write!(f, "{{{}}}", str.join(" "))
    }
}

impl fmt::Debug for EnvType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str: Vec<String> = vec![];

        for (ref key, ref value) in self.data.iter() {
            str.push(format!("{} {}", key, value.format(true)));
        }

        str.sort();

        write!(f, "Env {{ data: {{{}}} }}", str.join(" "))
    }
}


pub fn c_env(env: Option<Env>) -> Env {
    Rc::new(RefCell::new(EnvType {
        parent: env,
        data: HashMap::new(),
    }))
}

pub fn env_find(env: &Env, key: &AtomVal) -> Option<Env> {
    match **key {
        AtomType::Symbol(ref str) => {
            let env_borrow = env.borrow();
            match env_borrow.data.get(str) {
                Some(_) => Some(env.clone()),
                None => {
                    if let Some(ref parent) = env_borrow.parent {
                        env_find(parent, key)
                    } else {
                        None
                    }
                }
            }
        }
        _ => None,
    }
}

pub fn env_set(env: &Env, key: &AtomVal, value: AtomVal) -> AtomRet {
    match **key {
        AtomType::Symbol(ref str) => {
            env.borrow_mut().data.insert(str.to_string(), value);
            Ok(c_nil())
        }
        _ => unreachable!(),
    }

}

pub fn env_get(env: &Env, key: &AtomVal) -> Option<AtomVal> {
    match env_find(env, key) {
        None => None,
        Some(env) => {
            match **key {
                AtomType::Symbol(ref k) => {
                    match env.borrow().data.get(k) {
                        Some(v) => Some(v.clone()),
                        None => None,
                    }
                }
                _ => None,
            }
        }
    }
}

pub fn env_bind(env: &Env, params: &Vec<AtomVal>, args: &Vec<AtomVal>) -> AtomRet {
    for (index, param) in params.iter().enumerate() {
        env_set(env, param, args.get(0).unwrap_or(&c_nil()).clone())?;
    }
    Ok(c_nil())
}

#[cfg(test)]
mod tests {
    use super::{c_env, env_set, env_get};
    use super::super::data::{c_symbol, c_int};

    #[test]
    fn test_c_env() {
        let env = c_env(None);

        assert_eq!(format!("{}", *env.borrow()), "{}");
    }

    #[test]
    fn test_set() {
        let env = c_env(None);
        env_set(&env, &c_symbol(String::from("Test")), c_int(10));
        env_set(&env, &c_symbol(String::from("Gra")), c_int(5));

        assert_eq!(format!("{}", *env.borrow()), "{Gra 5 Test 10}");
    }

    #[test]
    fn test_get() {
        let env = c_env(None);
        let key = c_symbol(String::from("Test"));
        env_set(&env, &key.clone(), c_int(10));

        let child = c_env(Some(env));
        env_set(&child, &c_symbol(String::from("TestChild")), c_int(20));


        let grandchild = c_env(Some(child));

        assert_eq!(format!("{}", env_get(&grandchild, &key).unwrap()), "10");
        assert_eq!(format!("{}",
                           env_get(&grandchild, &c_symbol(String::from("TestChild"))).unwrap()),
                   "20");
    }

    #[test]
    fn test_get_missing_value() {
        let env = c_env(None);

        assert!(env_get(&env, &c_symbol(String::from("Missing"))).is_none());
    }
}
