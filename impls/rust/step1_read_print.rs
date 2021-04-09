#![allow(non_snake_case)]
use std::io;
use std::io::Write;

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
        println!("{}", rep(input.drain(..).collect()));
    }
}
//READ, EVAL, PRINT, and rep

fn READ(s: String) -> Result<MalVal> {
    read_str(s)
}
fn EVAL(s: Result<MalVal>) -> Result<MalVal> {
    s
}
fn PRINT(val: Result<MalVal>) -> String {
    match val {
        Ok(val) => val.to_string(),
        Err(e) => e.to_string(),
    }
}
fn rep(s: String) -> String {
    PRINT(EVAL(READ(s)))
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
