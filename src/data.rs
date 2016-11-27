use std::fmt::*;
use std::rc::Rc;
use std::error::Error as StdError;
use std::result;
use env::Env;

#[derive(Debug, PartialEq)]
pub enum AtomType {
    Nil,
    Int(i64),
    Symbol(String),
    List(Vec<AtomVal>),
    Vec(Vec<AtomVal>),
    Func(fn(Vec<AtomVal>) -> AtomRet),
    AFunc(AFuncData), // user defined function
}


#[derive(Clone, Debug, PartialEq)]
pub struct AFuncData {
    pub exp: AtomVal,
    pub env: Env,
    pub params: AtomVal,
    pub is_macro: bool
}

impl Display for AtomType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.format(false))
    }
}

impl AtomType {
    pub fn format(&self, with_type: bool) -> String {
        if with_type {
            match self {
                &AtomType::Int(num) => format!("Int({})", num),
                &AtomType::List(ref seq) => {
                    let list = seq.iter()
                        .map(|ref v| v.format(true))
                        .collect::<Vec<_>>()
                        .join(" ");

                    format!("List({})", list)
                }
                &AtomType::Vec(ref vec) => {
                    let list = vec.iter()
                        .map(|ref v| v.format(true))
                        .collect::<Vec<_>>()
                        .join(" ");

                    format!("Vec({})", list)
                }
                &AtomType::Nil => format!("Nil()"),
                &AtomType::Symbol(ref symbol) => format!("Symbol({})", symbol),
                &AtomType::Func(_) => format!("#func()"),
                &AtomType::AFunc(ref data) => {
                    let _type = if data.is_macro {
                        "macro"
                    } else {
                        "builtin_func"
                    };

                    format!("#{}(exp={} params={})",
                            _type,
                            data.exp,
                            data.params.format(true))
                }
            }
        } else {
            match self {
                &AtomType::Int(num) => format!("{}", num),
                &AtomType::List(ref seq) => {
                    let list = seq.iter()
                        .map(|ref v| v.format(false))
                        .collect::<Vec<_>>()
                        .join(" ");

                    format!("({})", list)
                }
                &AtomType::Vec(ref vec) => {
                    let list = vec.iter()
                        .map(|ref v| v.format(false))
                        .collect::<Vec<_>>()
                        .join(" ");

                    format!("({})", list)
                }
                &AtomType::Nil => format!("nil"),
                &AtomType::Symbol(ref symbol) => format!("{}", symbol),
                &AtomType::Func(_) => format!("#func()"),
                &AtomType::AFunc(ref data) => {
                    if data.is_macro {
                        format!("#macro()")
                    } else {
                        format!("#builtin_func()")
                    }
                },
            }
        }
    }


    pub fn apply(&self, args: Vec<AtomVal>) -> AtomRet {
        use eval::eval;
        use env::{c_env, env_bind, env_set, Env};

        match *self {
            AtomType::Func(f) => f(args),
            AtomType::AFunc(ref fd) => {
                let fd = fd.clone();
                let func_env = c_env(Some(fd.env.clone()));
                match *fd.params {
                    AtomType::List(ref params) => {
                        let ampersand = c_symbol("&".to_string());
                        // println!("params={:?} args={:?}", params, args);
                        env_bind(&func_env, params, &args)?;

                        let args_count = params.clone().iter().take_while(|v| **v != ampersand).count();
                        let rest = args.into_iter().skip(args_count).collect::<Vec<AtomVal>>();

                        if args_count != params.iter().count() && params.get(args_count + 1).is_some() {
                            if rest.len() > 0 {
                                env_set(&func_env, params.get(args_count + 1).unwrap(), c_list(rest))?;
                            } else {
                                env_set(&func_env, params.get(args_count + 1).unwrap(), c_nil())?;

                            }
                        }

                    },
                    ref v => return Err(AtomError::InvalidType("list".to_string(), v.format(true)))
                }

                // println!("ENV FUNC: {:?}", func_env);
                eval(fd.exp, func_env)
            },
            _ => Err(AtomError::InvalidType("function".to_string(), self.format(true)))
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum AtomError {
    // expected, received
    InvalidType(String, String),
    // operation name
    InvalidOperation(String),
    UndefinedSymbol(String),
}


impl Display for AtomError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use self::AtomError::*;

        let output = match *self {
            InvalidType(ref expected, ref got) => {
                format!("expected: {}, received: {}", expected, got)
            }
            InvalidOperation(ref op) => format!("invalid operation: {}", op),
            UndefinedSymbol(ref op) => format!("undefined symbol: {}", op),
        };

        write!(f, "{}", output)
    }
}

pub type AtomVal = Rc<AtomType>;
pub type AtomRet = result::Result<AtomVal, AtomError>;

pub fn c_nil() -> AtomVal {
    Rc::new(AtomType::Nil)
}

pub fn c_int(num: i64) -> AtomVal {
    Rc::new(AtomType::Int(num))
}

pub fn c_symbol(symbol: String) -> AtomVal {
    if symbol.to_uppercase() == "nil" {
        c_nil()
    } else {
        Rc::new(AtomType::Symbol(symbol))
    }
}

pub fn c_list(seq: Vec<AtomVal>) -> AtomVal {
    Rc::new(AtomType::List(seq))
}

pub fn c_func(f: fn(Vec<AtomVal>) -> AtomRet) -> AtomVal {
    Rc::new(AtomType::Func(f))
}


pub fn c_vec(seq: Vec<AtomVal>) -> AtomVal {
    Rc::new(AtomType::Vec(seq))
}

pub fn c_afunc(env: Env, params: AtomVal, exp: AtomVal) -> AtomVal {
    Rc::new(AtomType::AFunc(AFuncData { exp, env, params, is_macro: false }))
}

pub fn c_macro(fd: &AFuncData) -> AtomVal {
    let mut fd = (*fd).clone();
    fd.is_macro = true;

    Rc::new(AtomType::AFunc(fd))
}


#[cfg(test)]
mod tests {
    use super::c_nil;
    use super::c_int;
    use super::c_symbol;
    use super::c_list;
    use super::c_vec;

    #[test]
    fn test_nil() {
        assert_eq!(format!("{}", c_nil()), "nil");
    }

    #[test]
    fn test_int() {
        assert_eq!(format!("{}", c_int(0)), "0");
    }

    #[test]
    fn test_symbol() {
        assert_eq!(format!("{}", c_symbol(String::from("test"))), "test");
    }

    #[test]
    fn test_list() {
        let foo = c_int(0);
        let bar = c_int(1);
        let list = c_list(vec![foo, bar]);

        assert_eq!(format!("{}", list), "(0 1)");
    }

    #[test]
    fn test_vec() {
        let vec = c_vec(vec![c_int(0), c_int(1)]);

        assert_eq!(format!("{}", vec), "(0 1)");
    }

    #[test]
    fn test_nested_seq() {
        let foo = c_int(0);
        let bar = c_int(1);
        let baz = c_int(2);
        let list = c_list(vec![foo, bar]);
        let list2 = c_list(vec![list, baz]);


        assert_eq!(format!("{}", list2), "((0 1) 2)");
    }
}
