use std::{
    any::{type_name, Any},
    cell::{Cell, RefCell},
    rc::Rc,
};

use slotmap::SecondaryMap;

use crate::nodes::Scope;

pub struct Reciever(Box<dyn Fn(&Box<dyn Any>)>);
impl std::fmt::Debug for Reciever {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", type_name::<Reciever>())
    }
}

#[derive(Default, Debug, Clone)]
pub struct Recievers {
    handlers: Rc<RefCell<SecondaryMap<Scope, Vec<Reciever>>>>,
    borrowed: Rc<Cell<Option<Scope>>>,
}

impl Recievers {
    pub fn create_reciever<T: 'static>(&self, scope: Scope, f: impl Fn(&T) + 'static) {
        let borrowed = self.borrowed.clone();
        let reciever = Reciever(Box::new(move |v| {
            if borrowed.get() != Some(scope) {
                return;
            }
            match v.downcast_ref::<T>() {
                Some(v) => f(v),
                None => {}
            }
        }));
        let mut map = self.handlers.borrow_mut();
        match map.get_mut(scope) {
            Some(v) => v.push(reciever),
            None => {
                map.insert(scope, vec![reciever]);
            }
        };
    }

    pub fn send(&self, scope: Scope, value: &Box<dyn Any>) {
        let recievers = self.handlers.borrow_mut().remove(scope);
        if let Some(recievers) = recievers {
            let previous = self.borrowed.replace(Some(scope));
            for r in &recievers {
                r.0(value)
            }
            if self.borrowed.get() == Some(scope) {
                self.handlers.borrow_mut().insert(scope, recievers);
            }
            self.borrowed.replace(previous);
        }
    }

    pub fn dispose(&self, scope: Scope) {
        match Some(scope) == self.borrowed.get() {
            true => self.borrowed.set(None),
            false => {
                self.handlers.borrow_mut().remove(scope);
            }
        }
    }
}
