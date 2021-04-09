#![allow(non_snake_case)]
use std::io::Write;
use std::{collections::HashMap, io};

pub mod reader;
use reader::*;
pub mod types;
use types::*;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[macro_export]
macro_rules! error {
    ($e: expr) => {
        format!("{} line {}", $e, line!())
    };
}

#[macro_export]
macro_rules! unwrap {
    ($var: expr, $varient: path) => {
        if let $varient(var) = $var {
            var
        } else {
            unreachable!();
        }
    };
}

fn main() -> Result<()> {
    let mut input = String::new();
    loop {
        print!("user> ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        let result = rep(input.drain(..).collect());
        match result {
            Ok(result) => println!("{}", result),
            Err(e) => println!("{}", e),
        }
    }
}
//READ, EVAL, PRINT, and rep

fn READ(s: String) -> Result<MalVal> {
    read_str(s)
}
fn EVAL(val: MalVal, repl_env: &HashMap<String, MalVal>) -> Result<MalVal> {
    match val {
        MalVal::List(ref vals) => {
            if vals.is_empty() {
                return Ok(val);
            }
            //ast is a list: call eval_ast to get a new evaluated list. Take the first item of the evaluated list and call it as function using the rest of the evaluated list as its arguments.
            let list = eval_ast(val, repl_env)?;
            let list = unwrap!(list, MalVal::List);

            let fun = &list[0];
            if let MalVal::Func(fun) = fun {
                let args = list[1..].to_vec();

                Ok(fun(args))
            }
            // custom user function??
            else if let MalVal::Str(_name) = fun {
                //args are in list[1] ?
                eval_ast(list[1].clone(), repl_env)
            } else {
                unreachable!()
            }
        }
        _ => eval_ast(val, repl_env),
    }
}

fn eval_ast(val: MalVal, repl_env: &HashMap<String, MalVal>) -> Result<MalVal> {
    match val {
        MalVal::Symbol(sym) => repl_env.get(&sym).cloned().ok_or("value not found".into()),
        MalVal::List(vals) => {
            let mut list = vec![];
            for val in vals {
                list.push(EVAL(val, repl_env)?);
            }
            Ok(MalVal::List(list))
        }
        MalVal::Vector(vals) => {
            let mut list = vec![];
            for val in vals {
                list.push(EVAL(val, repl_env)?);
            }
            Ok(MalVal::Vector(list))
        }
        MalVal::Hash(vals) => {
            let mut hash = HashMap::new();
            for (key, val) in vals {
                hash.insert(key, EVAL(val, repl_env)?);
            }
            Ok(MalVal::Hash(hash))
        }
        _ => Ok(val),
    }
}

fn PRINT(val: MalVal) -> String {
    val.to_string()
}

fn rep(s: String) -> Result<String> {
    let mut repl_env: HashMap<String, MalVal> = HashMap::new();
    macro_rules! add_op_to_env {
        ($op: tt) => (
            repl_env.insert(
                stringify!($op).into(),
                MalVal::Func(|vals| {
                    let mut result = unwrap!(vals[0],MalVal::Int);

                    for val in vals.iter().skip(1) {
                        let val = unwrap!(val, MalVal::Int);
                         result = result $op val;
                    }
                    MalVal::Int(result)
                }),
            );
    )}

    add_op_to_env!(+);
    add_op_to_env!(-);
    add_op_to_env!(*);
    add_op_to_env!(/);

    let ast = READ(s)?;
    let eval_ast = EVAL(ast, &repl_env)?;
    Ok(PRINT(eval_ast))
}
// tests
#[test]
fn integration() {
    // (+ 1 2) -> ['',(+ 1 2)]
    assert_eq!(
        READ("(+ 1 2)".into()).unwrap(),
        MalVal::List(vec![
            MalVal::Symbol("+".into()),
            MalVal::Int(1),
            MalVal::Int(2)
        ])
    );
}

#[test]
fn unterminated_quotes() {
    let test = READ("\"abc".into());
    assert!(test.is_err());
    assert!(unwrap!(test, Result::Err).to_string().contains("EOF"));
}

#[test]
fn backslash() {
    assert_eq!(READ("\"\\\\\"".into()).unwrap(), MalVal::Str("\\\\".into()));
}

#[test]
fn quote_plus_one() {
    assert_eq!(
        READ("'1".into()).unwrap(),
        MalVal::List(vec!(MalVal::Symbol("'".into()), MalVal::Int(1)))
    );
}

#[test]
fn presendence() {
    let s = "(- (+ 5 (* 2 3)) 3)".into();
    assert_eq!(rep(s).unwrap(), "8");
}
