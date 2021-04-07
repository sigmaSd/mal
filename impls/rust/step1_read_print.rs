#![allow(non_snake_case)]
use std::{cell::RefCell, collections::HashMap, io, iter::Peekable};
use std::{io::prelude::*, rc::Rc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    loop {
        print!("user> ");
        io::stdout().flush();
        io::stdin().read_line(&mut input)?;
        println!("{}", rep(input.drain(..).collect()));
    }
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

struct Reader<I: Iterator> {
    tokens: Peekable<I>,
}

impl<I: Iterator<Item = String>> Iterator for Reader<I> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next()
    }
}

impl<I: Iterator<Item = String>> Reader<I> {
    fn peek(&mut self) -> Option<&String> {
        self.tokens.peek()
    }
}

fn read_str(s: String) -> MalVal {
    let tokens = tokenize(s);
    read_from(Reader {
        tokens: tokens.into_iter().peekable(),
    })
}

fn read_from(peekable: Reader<std::vec::IntoIter<String>>) -> MalVal {
    todo!()
}

fn tokenize(s: String) -> Vec<String> {
    let mut siter = s.chars().peekable();
    let mut tokens: Vec<String> = vec![];
    (|| {
        loop {
            let c = siter.next()?;
            match c {
                // ignore whitespace and commas
                c if c.is_whitespace() || c == ',' => (),
                //~@ token
                '~' if siter.peek() == Some(&'@') => {
                    siter.next();
                    tokens.push("~@".into());
                }
                '[' | ']' | '{' | '}' | '(' | ')' | '\'' | '`' | '~' | '^' | '@' => {
                    tokens.push(c.to_string())
                }
                '"' => {
                    let mut string = String::new();
                    string.push('"');
                    let mut last_char = None;
                    while let Some(current_char) = siter.next() {
                        if current_char == '"' && last_char != Some('\\') {
                            string.push('"');
                            break;
                        } else {
                            string.push(current_char);
                            last_char = Some(current_char);
                        }
                    }
                    tokens.push(string);
                }
                ';' => {
                    let mut string = String::new();
                    string.push(';');
                    while let Some(current_char) = siter.next() {
                        if current_char.is_whitespace() {
                            break;
                        } else {
                            string.push(current_char);
                        }
                    }
                    tokens.push(string);
                }
                c if c.is_alphanumeric() => {
                    let mut string = String::new();
                    string.push(c);
                    while let Some(current_char) = siter.next() {
                        if current_char.is_whitespace() {
                            break;
                        } else {
                            string.push(current_char);
                        }
                    }
                    tokens.push(string);
                }
                _ => unreachable!(),
            }
        }
        Some(())
    })();
    tokens
}
#[test]
fn tokenize_test() {
    dbg!(tokenize("    ".into()));
    dbg!(tokenize("~@".into()));
    dbg!(tokenize("[hello](world)'".into()));
    dbg!(tokenize(";test".into()));
}

#[derive(Debug, Clone)]
enum MalVal {
    Nil,
    Bool(bool),
    Int(i64),
    //Float(f64),
    Str(String),
    Sym(String),
    List(Rc<Vec<MalVal>>, Rc<MalVal>),
    Vector(Rc<Vec<MalVal>>, Rc<MalVal>),
    Hash(Rc<HashMap<String, MalVal>>, Rc<MalVal>),
    Atom(Rc<RefCell<MalVal>>),
}
