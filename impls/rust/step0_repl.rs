use std::io;
use std::io::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    loop {
        print!("user> ");
        io::stdout().flush();
        io::stdin().read_line(&mut input)?;
        println!("{}", rep(input.drain(..).collect()));
    }
    Ok(())
}
//READ, EVAL, PRINT, and rep

fn READ(s: String) -> String {
    s
}
fn EVAL(s: String) -> String {
    s
}
fn PRINT(s: String) -> String {
    s
}
fn rep(s: String) -> String {
    PRINT(EVAL(READ(s)))
}
