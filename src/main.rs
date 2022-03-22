use std::{collections::HashMap, fs};

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub parser);

pub mod ast;
mod eval;
pub mod value;

use eval::Env;
use value::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test = fs::read_to_string("lang/test.txt")?;
    println!("source: {}", &test);

    let ast = parser::ProgParser::new().parse(&test).unwrap();
    println!("ast: {:?}", &ast);

    let mut env: Env = HashMap::new();
    env.insert("print", Value::Func(print_));

    let result = eval::run_prog(&mut env, &ast);
    println!("result: {:?}", result);

    Ok(())
}

fn print_(args: Vec<Value>) -> Result<Value, String> {
    println!("{:?}", args);
    Ok(Value::Unit)
}
