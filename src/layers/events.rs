use crate::node::{set_requires_update, Child};
use crate::scope::{use_layer, use_layer_option, with_scope_mut, NodeId};

use slotmap::SecondaryMap;
use std::{any::Any, cell::RefCell, rc::Rc};

#[derive(Default, Clone)]
pub struct Events {
    listeners: Rc<RefCell<SecondaryMap<NodeId, Vec<Box<dyn Fn(Box<dyn Any>) -> bool>>>>>,
}

impl Events {
    fn on<T: Clone + 'static>(&self, f: impl Fn(T) + 'static) {
        let listener = Box::new(move |x: Box<dyn Any>| match x.downcast_ref::<T>() {
            Some(x) => {
                f(x.clone());
                true
            }
            None => false,
        });
        let id = with_scope_mut(|s| s.get_parent());
        let mut listeners_map = self.listeners.borrow_mut();
        if let Some(listeners) = listeners_map.get_mut(id) {
            listeners.push(listener)
        } else {
            listeners_map.insert(id, vec![listener]);
        }
    }

    pub fn dispatch_to<T: Clone + Any + 'static>(&self, id: NodeId, e: T) {
        let listeners = self.listeners.borrow();
        if let Some(c) = listeners.get(id) {
            for l in c.iter() {
                if l(Box::new(e.clone())) {
                    set_requires_update(id, true)
                }
            }
        }
    }

    pub fn dispatch<T: Clone + Any + 'static>(&self, e: T) {
        let listeners = self.listeners.borrow();
        for (id, _) in listeners.iter() {
            self.dispatch_to(id, e.clone());
        }
    }
}

impl Child {
    pub fn dispatch<T: Clone + Any + 'static>(&self, e: T) {
        use_layer::<Events>().dispatch_to(self.id, e)
    }
}

pub fn on<T: Clone + 'static>(f: impl Fn(T) + 'static) {
    use_layer_option::<Events>()
        .expect("App is missing Events layer")
        .on(f);
}
