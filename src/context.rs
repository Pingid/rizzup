use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use crate::nodes::Scope;

#[derive(Default, Debug, Clone)]
pub struct Contexts(pub(crate) Rc<RefCell<HashMap<(Scope, TypeId), Box<dyn Any>>>>);

impl Contexts {
    pub(crate) fn provide_context<T: Clone + Any + 'static>(&self, scope: Scope, x: T) {
        let mut layers = self.0.borrow_mut();
        layers.insert((scope, TypeId::of::<T>()), Box::new(x));
    }

    pub(crate) fn use_context_from_scope<T: Clone + Any + 'static>(&self, id: Scope) -> Option<T> {
        let layers = self.0.borrow();
        let ctx = layers.get(&(id, TypeId::of::<T>()));
        if let Some(value) = ctx {
            let s = value.downcast_ref::<T>().expect("Failed to downcast");
            return Some(s.clone());
        }
        None
    }
}

// pub fn with_context<R>(f: impl FnOnce(&Context) -> R) -> R {
//     CONTEXT.with(f)
// }

// pub fn provide_context<T: Clone + Any + 'static>(x: T) {
//     with_context(|s| s.provide_context(x))
// }

// pub fn use_context_option<T: Clone + Any + 'static>() -> Option<T> {
//     with_context(|s| s.use_context_option())
// }

// pub fn use_context<T: Clone + Any + 'static>() -> T {
//     with_context(|s| s.use_context())
// }
