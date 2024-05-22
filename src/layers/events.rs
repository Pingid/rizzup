use crate::node::NodeRef;
use crate::scope::{use_layer, use_layer_option, with_scope, NodeId};
use slotmap::SecondaryMap;
use std::{any::Any, cell::RefCell, rc::Rc};

#[derive(Default, Clone)]
pub struct Events {
    listeners: Rc<RefCell<SecondaryMap<NodeId, Vec<Box<dyn Fn(&Box<dyn Any>)>>>>>,
}

impl Events {
    fn on<T: 'static>(&self, f: impl Fn(&T) + 'static) {
        let listener = Box::new(move |x: &Box<dyn Any>| match x.downcast_ref::<T>() {
            Some(x) => f(x),
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

    pub fn dispatch_boxed(&self, e: Box<dyn Any>) {
        let listeners = self.listeners.borrow();
        for (_, listeners) in listeners.iter() {
            for l in listeners {
                l(&e);
            }
        }
    }

    pub fn dispatch_to<T: Any + 'static>(&self, id: NodeId, e: T) {
        let listeners = self.listeners.borrow();
        let ev = Box::new(e) as Box<dyn Any>;
        if let Some(c) = listeners.get(id) {
            for l in c.iter() {
                l(&ev);
            }
        }
    }

    pub fn dispatch<T: Any + 'static>(&self, e: T) {
        self.dispatch_boxed(Box::new(e) as Box<dyn Any>);
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

pub fn on<T: Clone + 'static>(f: impl Fn(&T) + 'static) {
    use_layer_option::<Events>()
        .expect("App is missing Events layer")
        .on(f);
}

#[macro_export]
macro_rules! match_on {
    ($tp:ty, $pt:pat => $exp:expr) => {{
        on::<$tp>(move |ev| {
            match ev {
                $pt => $exp,
                _ => {},
            };
        });
    }};
    ($tp:ty, { $($pt:pat => $exp:expr,)* }) => {{
        on::<$tp>(move |ev| {
            match ev {
                $($pt => $exp,)*
                _ => {},
            };
        });
    }};
}
pub use match_on;
