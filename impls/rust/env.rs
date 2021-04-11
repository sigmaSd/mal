use crate::Result;
use crate::{types::MalVal, unwrap};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Env(Rc<RefCell<_Env>>);

#[derive(Debug)]
struct _Env {
    data: HashMap<String, MalVal>,
    outer: Option<Env>,
}

impl Env {
    pub fn new(outer: Option<Env>) -> Self {
        Env(Rc::new(RefCell::new(_Env {
            data: HashMap::new(),
            outer,
        })))
    }
    pub fn new_with_bindings(outer: Option<Env>, binds: MalVal, exprs: Vec<MalVal>) -> Self {
        let mut data = HashMap::new();
        let binds = unwrap!(binds, MalVal::List);
        for (key, val) in binds.into_iter().zip(exprs.into_iter()) {
            let key = unwrap!(key, MalVal::Symbol);
            data.insert(key, val);
        }

        Env(Rc::new(RefCell::new(_Env { data, outer })))
    }
    pub fn set(&self, key: String, val: MalVal) -> MalVal {
        self.0.borrow_mut().data.insert(key, val.clone());
        val
    }

    pub fn find(&self, key: &str) -> Option<Env> {
        if self.0.borrow().data.get(key).is_some() {
            return Some(self.clone());
        }
        if let Some(ref env) = self.0.borrow().outer {
            return env.find(key);
        }
        None
    }
    pub fn get(&self, key: &str) -> Result<MalVal> {
        self.find(key)
            .ok_or(format!("{} not found", key).into())
            .map(|env| env.0.borrow().data[key].clone())
    }
}
