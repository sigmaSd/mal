use crate::unwrap;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, PartialEq)]
pub enum MalVal {
    Nil,
    Bool(bool),
    Str(String),
    Keyword(String),
    Int(isize),
    Symbol(String),
    List(Vec<MalVal>),
    Vector(Vec<MalVal>),
    Hash(HashMap<String, MalVal>),
    Func(fn(Vec<MalVal>) -> MalVal),
}

macro_rules! impl_ops {
    ($path: path, $name: ident, $op: tt) => {
        impl $path for MalVal {
            type Output = MalVal;
            fn $name(self, rhs: MalVal) -> Self::Output {
                let selfl = unwrap!(self, MalVal::Int);
                let rhs = unwrap!(rhs, MalVal::Int);
                MalVal::Int(selfl $op rhs)
            }
        }
    };
}

impl_ops!(std::ops::Add, add, +);
impl_ops!(std::ops::Sub, sub, -);
impl_ops!(std::ops::Mul, mul, *);
impl_ops!(std::ops::Div, div, /);

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
                if !vals.is_empty() {
                    f.write_str(vals.keys().next().expect("already checked"))?;
                    f.write_char(' ')?;
                    vals.values().next().expect("already checked").fmt(f)?;
                }
                f.write_char('}')?;
                Ok(())
            }
            MalVal::Func(_) => {
                unreachable!();
            }
            MalVal::Nil => {
                unreachable!()
            }
            MalVal::Bool(val) => f.write_str(&val.to_string()),
            MalVal::Str(val) => {
                f.write_char('"')?;
                f.write_str(&val)?;
                f.write_char('"')?;
                Ok(())
            }
            MalVal::Keyword(val) => {
                // replace keyword unicode prefix
                let mut val: String = val.chars().skip(1).collect();
                val.insert(0, ':');
                f.write_str(&val)
            }
        }
    }
}
