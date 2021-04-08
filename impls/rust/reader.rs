use crate::error;
use crate::types::MalVal;
use crate::unwrap;
use crate::Result;
use std::collections::HashMap;
use std::iter::Peekable;

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

pub fn read_str(s: String) -> Result<MalVal> {
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
        token if token.starts_with("\"") => Ok(MalVal::Str(token[1..token.len() - 1].to_owned())),
        token if token.starts_with(":") => Ok(MalVal::Keyword(format!("\u{29e}{}", &token[1..]))),
        "nil" => Ok(MalVal::Nil),
        "true" => Ok(MalVal::Bool(true)),
        "false" => Ok(MalVal::Bool(false)),
        token if token.chars().all(char::is_alphabetic) => Ok(MalVal::Str(token.to_string())),
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
        '}' => {
            let mut hm = HashMap::new();
            if list.is_empty() {
                return Ok(MalVal::Hash(hm));
            }

            match &list[0] {
                MalVal::Str(_string) => {
                    hm.insert(list[0].to_string(), list[1].clone());
                    Ok(MalVal::Hash(hm))
                }
                MalVal::Keyword(_keyword) => {
                    hm.insert(list[0].to_string(), list[1].clone());
                    Ok(MalVal::Hash(hm))
                }
                _ => unreachable!(),
            }
        }
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
                    #[allow(unused_assignments)]
                    {
                        tokenize_non_special_character_if_present!();
                    }
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
                }
            }
        }
    })()?;
    Ok(tokens)
}

#[test]
fn tokenize_test() {
    dbg!(tokenize("    ".into()));
    dbg!(tokenize("~@".into()));
    dbg!(tokenize("[hello](world)'".into()));
    dbg!(tokenize(";test".into()));
}

#[test]
fn hashmap() {
    let s = "{\"a\" (+ 7 8)}".into();
    dbg!(read_str(s));
}

#[test]
fn unicode() {
    let s = "{:a (+ 7 8)}";
    dbg!(read_str(s.into()));
}
