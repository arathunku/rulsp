use std::fmt::*;
use std::rc::Rc;
use std::error::Error as StdError;
use std::result;

#[derive(Debug, PartialEq)]
pub enum AtomType {
    Nil,
    Int(i64),
    Symbol(String),
    List(Vec<AtomVal>),
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
                &AtomType::Nil => format!("Nil()"),
                &AtomType::Symbol(ref symbol) => format!("Symbol({})", symbol),
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
                &AtomType::Nil => format!("nil"),
                &AtomType::Symbol(ref symbol) => format!("{}", symbol),
            }
        }
    }
}


#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::c_nil;
    use super::c_int;
    use super::c_symbol;
    use super::c_list;

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
