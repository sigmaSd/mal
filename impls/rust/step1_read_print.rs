#![allow(non_snake_case)]
use std::io::Write;
use std::{fmt::Display, io, iter::Peekable};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

macro_rules! error {
    ($e: expr) => {
        format!("{} line {}", $e, line!())
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

fn read_str(s: String) -> Result<MalVal> {
    let tokens = tokenize(s)?;
    let mut reader = Reader {
        tokens: tokens.into_iter().peekable(),
    };
    read_from(&mut reader)
}

fn read_from(reader: &mut Reader<std::vec::IntoIter<String>>) -> Result<MalVal> {
    fn matching_symbol(c: char) -> char {
        match c {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            _ => unreachable!(),
        }
    }

    let token = reader.peek().ok_or("EOF")?;
    match token
        .chars()
        .next()
        .expect("tokenize guarantees that there is no empty token")
    {
        c @ '(' | c @ '[' | c @ '{' => read_seq(reader, matching_symbol(c)),
        _ => read_atom(reader),
    }
}

fn read_atom(reader: &mut Reader<std::vec::IntoIter<String>>) -> Result<MalVal> {
    let token = reader
        .next()
        .expect("first token is guaranteed by the caller function (read_from)");
    //
    match token.as_str() {
        token if token.parse::<isize>().is_ok() => Ok(MalVal::Int(token.parse::<isize>().unwrap())),
        "'" | "`" | "~" | "~@" | "@" => Ok(MalVal::List(vec![
            MalVal::Symbol(token.clone()),
            read_from(reader)?,
        ])),
        //macro with-meta
        "^" => {
            let meta = read_from(reader)?;
            Ok(MalVal::List(vec![
                MalVal::Symbol("with-meta".into()),
                read_from(reader)?,
                meta,
            ]))
        }
        _ => Ok(MalVal::Symbol(token.clone())),
    }
}

fn read_seq(reader: &mut Reader<std::vec::IntoIter<String>>, end: char) -> Result<MalVal> {
    // (
    let _token = reader
        .next()
        .expect("first token is guaranteed by the caller function (read_from)");
    let mut list = vec![];

    loop {
        let val = read_from(reader)?;

        if val == MalVal::Symbol(end.into()) {
            break;
        }
        list.push(val);
    }
    match end {
        ')' => Ok(MalVal::List(list)),
        ']' => Ok(MalVal::Vector(list)),
        '}' => Ok(MalVal::Hash(list)),
        _ => unreachable!(),
    }
}

fn tokenize(s: String) -> Result<Vec<String>> {
    let mut siter = s.chars().peekable();
    let mut tokens: Vec<String> = vec![];
    (|| -> Result<()> {
        enum State {
            Base,
            NonSpecialCharacter(String),
        }
        let mut state = State::Base;

        macro_rules! tokenize_non_special_character_if_present {
            () => {
                if let State::NonSpecialCharacter(string) = state {
                    tokens.push(string);
                    state = State::Base;
                }
            };
        }

        loop {
            // the only exit of the loop
            let c = match siter.next() {
                Some(c) => c,
                None => {
                    tokenize_non_special_character_if_present!();
                    break Ok(());
                }
            };

            match c {
                // ignore whitespace and commas
                c if c.is_whitespace() || c == ',' => tokenize_non_special_character_if_present!(),
                //~@ token
                '~' if siter.peek() == Some(&'@') => {
                    tokenize_non_special_character_if_present!();
                    siter.next();
                    tokens.push("~@".into());
                }
                '[' | ']' | '{' | '}' | '(' | ')' | '\'' | '`' | '~' | '^' | '@' => {
                    tokenize_non_special_character_if_present!();
                    tokens.push(c.to_string())
                }
                '"' => {
                    tokenize_non_special_character_if_present!();
                    let mut string = String::new();
                    string.push('"');
                    let mut escape = false;
                    loop {
                        let current_char = siter.next().ok_or(error!("EOF"))?;
                        if current_char == '"' && !escape {
                            string.push('"');
                            break;
                        } else {
                            string.push(current_char);
                            if current_char == '\\' {
                                escape = !escape;
                            } else {
                                escape = false;
                            }
                        }
                    }
                    tokens.push(string);
                }
                ';' => {
                    tokenize_non_special_character_if_present!();
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
                c => {
                    if let State::NonSpecialCharacter(ref mut string) = state {
                        string.push(c);
                    } else {
                        state = State::NonSpecialCharacter(c.to_string());
                    }
                    //string.push(c);
                    // while let Some(current_char) = siter.next() {
                    //     if current_char.is_whitespace() {
                    //         break;
                    //     } else {
                    //         string.push(current_char);
                    //     }
                    // }
                    // tokens.push(string);
                }
            }
        }
    })()?;
    Ok(tokens)
}

#[derive(Debug, Clone, PartialEq)]
enum MalVal {
    Int(isize),
    Symbol(String),
    List(Vec<MalVal>),
    Vector(Vec<MalVal>),
    Hash(Vec<MalVal>),
}

impl Display for MalVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        macro_rules! write_seq {
            ($vals: expr) => {
                let len = $vals.len();
                for (idx, val) in $vals.iter().enumerate() {
                    val.fmt(f)?;
                    // if its not the last value, add a space sperator
                    if idx + 1 != len {
                        f.write_char(' ')?;
                    }
                }
            };
        }
        match self {
            MalVal::Int(int) => f.write_str(&int.to_string()),
            MalVal::Symbol(sym) => {
                match sym.as_str() {
                    "'" => f.write_str("quote")?,
                    "`" => f.write_str("quasiquote")?,
                    "~" => f.write_str("unquote")?,
                    "@" => f.write_str("deref")?,
                    "@~" => f.write_str("splice")?,
                    "~@" => f.write_str("splice-unquote")?,
                    _ => f.write_str(sym)?,
                }
                Ok(())
            }
            MalVal::List(vals) => {
                f.write_char('(')?;
                write_seq!(vals);
                f.write_char(')')?;
                Ok(())
            }
            MalVal::Vector(vals) => {
                f.write_char('[')?;
                write_seq!(vals);
                f.write_char(']')?;
                Ok(())
            }
            MalVal::Hash(vals) => {
                f.write_char('{')?;
                write_seq!(vals);
                f.write_char('}')?;
                Ok(())
            }
        }
    }
}

// tests
#[test]
fn tokenize_test() {
    dbg!(tokenize("    ".into()));
    dbg!(tokenize("~@".into()));
    dbg!(tokenize("[hello](world)'".into()));
    dbg!(tokenize(";test".into()));
}
#[test]
fn integration() {
    // (+ 1 2) -> ['',(+ 1 2)]
    dbg!(READ("(+ 1 2)".into()));
}

#[test]
fn unterminated_quotes() {
    dbg!(READ("\"abc".into()));
}

#[test]
fn backslash() {
    dbg!(READ("\"\\\\\"".into()));
}

#[test]
fn quote_plus_one() {
    dbg!(READ("'1".into()));
}
