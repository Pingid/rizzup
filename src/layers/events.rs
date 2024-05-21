use crate::node::NodeRef;
use crate::scope::{use_layer, use_layer_option, with_scope, NodeId};
use slotmap::SecondaryMap;
use std::{any::Any, cell::RefCell, rc::Rc};

#[derive(Default, Clone)]
pub struct Events {
    listeners: Rc<RefCell<SecondaryMap<NodeId, Vec<Box<dyn Fn(Box<dyn Any>)>>>>>,
}

impl Events {
    fn on<T: Clone + 'static>(&self, f: impl Fn(T) + 'static) {
        let listener = Box::new(move |x: Box<dyn Any>| match x.downcast_ref::<T>() {
            Some(x) => f(x.clone()),
            None => {}
        });
        let id = with_scope(|s| s.get_current_node()).expect("Missing current node");
        let mut listeners_map = self.listeners.borrow_mut();
        listeners_map
            .entry(id)
            .unwrap()
            .or_insert(vec![])
            .push(listener);
    }

    pub fn dispatch_to<T: Clone + Any + 'static>(&self, id: NodeId, e: T) {
        let listeners = self.listeners.borrow();
        if let Some(c) = listeners.get(id) {
            for l in c.iter() {
                l(Box::new(e.clone()));
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

pub trait Dispatch {
    fn dispatch<T: Clone + Any + 'static>(&self, e: T);
}

impl<T: NodeRef> Dispatch for T {
    fn dispatch<E: Clone + Any + 'static>(&self, e: E) {
        use_layer::<Events>().dispatch_to(self.get_node(), e)
    }
}

pub fn on<T: Clone + 'static>(f: impl Fn(T) + 'static) {
    use_layer_option::<Events>()
        .expect("App is missing Events layer")
        .on(f);
}
