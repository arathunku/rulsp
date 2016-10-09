use std::fmt::*;

#[derive(Debug, PartialEq)]
pub enum Atom {
    Nil,
    Int(i64),
    Symbol(Box<str>),
    Pair(Box<Atom>, Box<Atom>),
}

pub fn create_nil() -> Atom {
    Atom::Nil
}

pub fn create_int(num: i64) -> Atom {
    Atom::Int(num)
}

pub fn create_symbol(symbol: String) -> Atom {
    Atom::Symbol(symbol.to_uppercase().into_boxed_str())
}

pub fn create_pair(car: Atom, cdr: Atom) -> Atom {
    Atom::Pair(Box::new(car), Box::new(cdr))
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            &Atom::Int(num) => write!(f, "{}", num),
            &Atom::Pair(ref car, ref cdr) => {
                match cdr.as_ref() {
                    &Atom::Nil => write!(f, "({})", car),
                    _ => write!(f, "({} {})", car, cdr),

                }
            }
            &Atom::Nil => write!(f, ""),
            &Atom::Symbol(ref symbol) => write!(f, "{}", symbol),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::create_nil;
    use super::create_int;
    use super::create_symbol;
    use super::create_pair;
    use super::Atom;

    #[test]
    fn test_nil() {
        assert_eq!(create_nil(), Atom::Nil);
    }

    #[test]
    fn test_int() {
        let foo = create_int(0);

        assert_eq!(foo, Atom::Int(0));
    }

    #[test]
    fn test_symbol() {
        assert_eq!(create_symbol(String::from("test")),
                   Atom::Symbol(String::from("TEST").into_boxed_str()));
    }

    #[test]
    fn test_simple_pair() {
        let foo = create_int(0);
        let bar = create_int(1);
        let pair = create_pair(foo, bar);

        match pair {
            Atom::Pair(a, b) => {
                assert_eq!(*a, Atom::Int(0));
                assert_eq!(*b, Atom::Int(1))
            }
            _ => unimplemented!(),
        }
    }

    #[test]
    fn test_nested_pair() {
        let foo = create_int(0);
        let bar = create_int(1);
        let baz = create_int(2);
        let pair = create_pair(foo, bar);
        let pair2 = create_pair(pair, baz);

        match pair2 {
            Atom::Pair(a, b) => {
                let pair = *a;
                match pair {
                    Atom::Pair(c, d) => {
                        assert_eq!(*c, Atom::Int(0));
                        assert_eq!(*d, Atom::Int(1));
                    }
                    _ => unimplemented!(),
                }

                assert_eq!(*b, Atom::Int(2));
            }
            _ => unimplemented!(),
        }
    }
}
