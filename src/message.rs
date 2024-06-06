use std::any::Any;

use crate::{
    node::{Callback, Node, NodeType, Scope},
    scope::{with_scope, Runtime},
};

impl Runtime {
    /// Create a node that recieves external data in its callback
    pub(crate) fn create_receiver_node<T: Any + 'static>(&self, f: impl Fn(&T) + 'static) {
        let cb: Box<dyn Fn(&Box<dyn Any>)> =
            Box::new(move |value| match value.downcast_ref::<T>() {
                Some(x) => f(x),
                None => {}
            });
        {
            let mut recievers = self.recievers.borrow_mut();
            let scope = self.tracker.get().expect("Missing tracking scope");
            match recievers.get_mut(scope) {
                Some(r) => r.push(cb),
                None => {
                    recievers.insert(scope, vec![cb]);
                }
            };
        }

        // let callback = Callback(Box::new(move |value| {
        //     let value = value?;
        //     let value = value.downcast_ref::<T>()?;
        //     Some(Box::new(f(value)))
        // }));
        // let mut node = Node::default();
        // node.node_type = NodeType::Reciever;
        // node.parent = self.tracker.get();
        // node.callback = Some(callback);
        // self.nodes.borrow_mut().insert(node);
    }

    pub(crate) fn send_to_node(&self, id: Scope, message: &Box<dyn Any>) {
        let recievers = self.recievers.borrow();
        let r = recievers.get(id);
        if let Some(recievers) = r {
            for r in recievers {
                let previous = self.tracker.replace(Some(id));
                r(message);
                self.tracker.replace(previous);
            }
        }
        // if let Some(cb) = self.with_node_mut(id, |n| match n.node_type {
        //     NodeType::Reciever => n.callback.take(),
        //     _ => None,
        // }) {
        //     let previous = self.tracker.replace(Some(id));
        //     cb.0(Some(message));
        //     self.with_node_mut(id, |n| n.callback.replace(cb));
        //     self.tracker.replace(previous);
        // }
    }

    pub(crate) fn send_to_children(&self, id: Scope, message: &Box<dyn Any>) {
        let childs = self.get_children(id);
        for reciever_id in childs {
            self.send_to_node(reciever_id, message)
        }
    }

    pub(crate) fn send_to_scope(&self, id: Scope, message: &Box<dyn Any>) {
        let childs = self.get_scope_children(id);
        for reciever_id in childs {
            self.send_to_node(reciever_id, message)
        }
    }
}

pub fn on<T: 'static>(f: impl Fn(&T) + 'static) {
    with_scope(|s| s.create_receiver_node(f));
}

pub fn send_boxed(node: impl Into<Scope>, message: Box<dyn Any>) {
    with_scope(|s| {
        let scope = s
            .with_node_ref(node.into(), |n| n.parent)
            .expect("Missing parent tracking scope");
        s.send_to_scope(scope, &message)
    });
}

pub fn send_boxed_all(node: impl Into<Scope>, message: Box<dyn Any>) {
    with_scope(|s| {
        let scope = s
            .with_node_ref(node.into(), |n| n.parent)
            .expect("Missing parent tracking scope");
        s.send_to_children(scope, &message)
    });
}

pub fn send<T: Any + 'static>(node: impl Into<Scope>, message: T) {
    send_boxed(node.into(), Box::new(message) as Box<dyn Any>);
}

pub fn send_all<T: 'static>(node: impl Into<Scope>, message: T) {
    send_boxed_all(node.into(), Box::new(message) as Box<dyn Any>)
}
