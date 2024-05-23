use crate::{
    node::{NodeId, NodeRef},
    scope::with_scope,
};
use std::{any::Any, marker::PhantomData};

pub trait SignalReader<T: Clone + 'static>: NodeRef {
    fn get_untracked(&self) -> T {
        self.node_id_ref().with_value_ref(|v: &T| v.clone())
    }

    fn get(&self) -> T {
        let node = self.node_id_ref();
        let parent = with_scope(|s| s.get_current_node()).expect("Missing tracking scope");
        node.add_dependancy(parent);
        self.get_untracked()
    }
}

pub trait SignalWriter<T: Clone + 'static>: NodeRef {
    fn update_untracked(&self, f: impl FnOnce(&mut T)) {
        self.node_id_ref().with_value_mut(f);
    }

    fn update(&self, f: impl FnOnce(&mut T)) {
        let node = self.node_id_ref();
        node.with_value_mut(f);
        node.mark_stale();
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

impl<T> From<NodeId> for ReadSignal<T> {
    fn from(value: NodeId) -> Self {
        Self(value, PhantomData)
    }
}

impl<T> NodeRef for ReadSignal<T> {
    fn node_id_ref(&self) -> &NodeId {
        &self.0
    }
}

impl<T: Clone + 'static> SignalReader<T> for ReadSignal<T> {}

#[derive(Clone, Copy)]
pub struct WriteSignal<T>(NodeId, PhantomData<T>);

impl<T> From<NodeId> for WriteSignal<T> {
    fn from(value: NodeId) -> Self {
        Self(value, PhantomData)
    }
}

impl<T> NodeRef for WriteSignal<T> {
    fn node_id_ref(&self) -> &NodeId {
        &self.0
    }
}

impl<T: Clone + 'static> SignalWriter<T> for WriteSignal<T> {}

#[derive(Clone, Copy)]
pub struct RwSignal<T>(NodeId, PhantomData<T>);

impl<T> From<NodeId> for RwSignal<T> {
    fn from(value: NodeId) -> Self {
        Self(value, PhantomData)
    }
}

impl<T> NodeRef for RwSignal<T> {
    fn node_id_ref(&self) -> &NodeId {
        &self.0
    }
}
impl<T: Clone + 'static> SignalReader<T> for RwSignal<T> {}
impl<T: Clone + 'static> SignalWriter<T> for RwSignal<T> {}

pub fn create_signal<T: Clone + 'static>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let id = NodeId::from_value(value);
    (id.into(), id.into())
}

pub fn create_rw_signal<T: Clone + 'static>(value: T) -> RwSignal<T> {
    let id = NodeId::from_value(value);
    id.into()
}

pub fn create_memo<T: Any + Clone + 'static>(f: impl Fn() -> T + 'static) -> ReadSignal<T> {
    let id = NodeId::from_memo(f);
    id.into()
}

#[cfg(test)]
mod tests {
    use super::super::node::*;
    use super::*;

    #[test]
    fn test_memo_inherits_value() {
        with_scope(|s| s.set_current_node(Some(s.insert_node(Node::default()))));
        let sig = create_rw_signal("foo");
        let value = create_memo(move || sig.get().to_uppercase());
        assert_eq!(sig.get(), "foo");
        assert_eq!(value.get(), "FOO");
        sig.set("bar");
        assert_eq!(sig.get(), "bar");
        assert_eq!(value.get(), "BAR");
    }

    #[test]
    fn nested_memo_inherits_value() {
        with_scope(|s| s.set_current_node(Some(s.insert_node(Node::default()))));
        let value = create_memo(move || {
            let sig = create_rw_signal("foo");
            assert_eq!(sig.get(), "foo");
            let value = create_memo(move || sig.get().to_uppercase());
            assert_eq!(value.get(), "FOO");
            sig.set("bar");
            assert_eq!(sig.get(), "bar");
            assert_eq!(value.get(), "BAR");
            "foo"
        });
        assert_eq!(value.get(), "foo")
    }
}
