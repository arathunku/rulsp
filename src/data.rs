use std::fmt::*;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum AtomType {
    Nil,
    Int(i64),
    Symbol(String),
    List(Vec<AtomVal>),
}

pub type AtomVal = Rc<AtomType>;

pub fn c_nil() -> AtomVal {
    Rc::new(AtomType::Nil)
}

pub fn c_int(num: i64) -> AtomVal {
    Rc::new(AtomType::Int(num))
}

pub fn c_symbol(symbol: String) -> AtomVal {
    if symbol.to_uppercase() == "NIL" {
        c_nil()
    } else {
        Rc::new(AtomType::Symbol(symbol.to_uppercase()))
    }
}

pub fn c_list(seq: Vec<AtomVal>) -> AtomVal {
    Rc::new(AtomType::List(seq))
}

pub fn c_env(parent: AtomVal) -> AtomVal {
    c_list(vec![parent])
}

pub fn c_get(env: AtomVal, symbol: AtomVal) -> AtomVal {}

impl Display for AtomType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            &AtomType::Int(num) => write!(f, "{}", num),
            &AtomType::List(ref seq) => {
                let list = seq.iter()
                    .map(|ref v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(" ");

                write!(f, "({})", list)
            }
            &AtomType::Nil => write!(f, "NIL"),
            &AtomType::Symbol(ref symbol) => write!(f, "{}", symbol),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::c_nil;
    use super::c_int;
    use super::c_symbol;
    use super::c_list;

    #[test]
    fn test_nil() {
        assert_eq!(format!("{}", c_nil()), "NIL");
    }

    #[test]
    fn test_int() {
        assert_eq!(format!("{}", c_int(0)), "0");
    }

    #[test]
    fn test_symbol() {
        assert_eq!(format!("{}", c_symbol(String::from("test"))), "TEST");
    }

    #[test]
    fn test_seq() {
        let foo = c_int(0);
        let bar = c_int(1);
        let list = c_list(vec![foo, bar]);

        assert_eq!(format!("{}", list), "(0 1)");
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
