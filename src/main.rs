mod data;
mod parser;

use data::{create_int, create_pair, create_nil, create_symbol};
use parser::lex;


fn main() {
    let num = create_int(0);
    let pair = create_pair(create_pair(create_int(1), create_symbol(String::from("ok"))),
                           create_pair(create_int(1), create_nil()));

    println!("{}", num);
    println!("{}", pair);

    println!("{:?}", lex("()".to_string()));
}
