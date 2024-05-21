use crate::{
    node::{Node, NodeRef},
    scope::{with_scope, NodeId},
};
use std::marker::PhantomData;

pub trait SignalReader<T: Clone + 'static>: NodeRef {
    fn get_untracked(&self) -> T {
        self.get_node()
            .with(|n| n.with_value(|v: &T| v.clone()))
            .expect("Missing signal value")
    }

    fn get(&self) -> T {
        let node = self.get_node();
        let parent = with_scope(|s| s.get_current_node()).expect("Missing tracking scope");
        node.with(|n| {
            n.dependants.borrow_mut().push(parent);
            n.with_value(|v: &T| v.clone())
        })
        .expect("Missing signal value")
    }
}

pub trait SignalWriter<T: Clone + 'static>: NodeRef {
    fn update_untracked(&self, f: impl FnOnce(&mut T)) {
        self.get_node().with(|n| n.with_value_mut(f));
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        self.get_node().with(|n| {
            n.with_value_mut(f);
            for dep in n.dependants.borrow().iter() {
                dep.with(|n| n.mark_stale());
            }
        });
    }

    fn set_untracked(&self, value: T) {
        self.update_untracked(|v| *v = value);
    }

    fn set(&self, value: T) {
        self.update(|v| *v = value);
    }
}

#[derive(Clone, Copy)]
pub struct ReadSignal<T>(NodeId, PhantomData<T>);
impl<T> NodeRef for ReadSignal<T> {
    fn get_node(&self) -> NodeId {
        self.0.clone()
    }
}
impl<T: Clone + 'static> SignalReader<T> for ReadSignal<T> {}

#[derive(Clone, Copy)]
pub struct WriteSignal<T>(NodeId, PhantomData<T>);
impl<T> NodeRef for WriteSignal<T> {
    fn get_node(&self) -> NodeId {
        self.0.clone()
    }
}
impl<T: Clone + 'static> SignalWriter<T> for WriteSignal<T> {}

#[derive(Clone, Copy)]
pub struct RwSignal<T>(NodeId, PhantomData<T>);
impl<T> NodeRef for RwSignal<T> {
    fn get_node(&self) -> NodeId {
        self.0.clone()
    }
}
impl<T: Clone + 'static> SignalReader<T> for RwSignal<T> {}
impl<T: Clone + 'static> SignalWriter<T> for RwSignal<T> {}

pub fn create_signal<T: Clone + 'static>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let node = Node::create_node_value(value);
    let id = with_scope(|s| s.insert_node(node));
    (ReadSignal(id, PhantomData), WriteSignal(id, PhantomData))
}

pub fn create_rw_signal<T: Clone + 'static>(value: T) -> RwSignal<T> {
    let node = Node::create_node_value(value);
    let id = with_scope(|s| s.insert_node(node));
    RwSignal(id, PhantomData)
}
