use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use slotmap::{new_key_type, SlotMap};

use super::{node::Child, node::Node, view::View};

thread_local! {
    static SCOPE: RefCell<Scope> = RefCell::new(Scope::default());
}

new_key_type! {
    pub struct NodeId;
}

#[derive(Default)]
pub struct Scope {
    parent: Option<NodeId>,
    nodes: SlotMap<NodeId, Node>,
    layers: HashMap<TypeId, Box<dyn Any>>,
}

impl Scope {
    pub fn with_node(&self, id: NodeId, f: impl FnOnce(&Node)) {
        if let Some(c) = self.nodes.get(id) {
            f(c)
        }
    }

    pub fn with_node_mut(&mut self, id: NodeId, f: impl FnOnce(&mut Node)) {
        if let Some(c) = self.nodes.get_mut(id) {
            f(c)
        }
    }

    pub fn view(&mut self, f: impl Fn() -> Box<dyn View> + 'static) -> Child {
        let id = self.get_parent();
        self.with_node_mut(id, |c| c.set_render_fn(f));
        Child::new(id)
    }

    pub fn get_parent(&mut self) -> NodeId {
        match self.parent {
            Some(id) => id,
            None => {
                let id = self.nodes.insert(Node::default());
                self.parent.replace(id);
                id
            }
        }
    }

    pub fn take_parent(&mut self) -> Option<NodeId> {
        self.parent.take()
    }

    pub fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent
    }

    pub fn provide_layer<T: Clone + Any + 'static>(&mut self, x: T) {
        self.layers.insert(TypeId::of::<T>(), Box::new(x));
    }
    pub fn use_layer_option<T: Clone + Any + 'static>(&self) -> Option<T> {
        self.layers
            .get(&TypeId::of::<T>())
            .map(|x| x.downcast_ref::<T>())
            .flatten()
            .map(|x| x.clone())
    }
    pub fn use_layer<T: Clone + Any + 'static>(&self) -> T {
        self.layers
            .get(&TypeId::of::<T>())
            .map(|x| x.downcast_ref::<T>())
            .flatten()
            .expect("Missing context")
            .clone()
    }
    pub fn use_layer_or_default<T: Clone + Default + Any + 'static>(&mut self) -> T {
        let id = TypeId::of::<T>();
        if !self.layers.contains_key(&id) {
            self.layers.insert(id, Box::new(T::default()));
        }
        self.layers
            .get(&TypeId::of::<T>())
            .map(|x| x.downcast_ref::<T>())
            .flatten()
            .expect("Missing context")
            .clone()
    }
}

pub fn with_scope<R>(f: impl FnOnce(&RefCell<Scope>) -> R) -> R {
    SCOPE.with(f)
}

pub fn with_scope_ref<R>(f: impl FnOnce(Ref<Scope>) -> R) -> R {
    with_scope(|s| f(s.borrow()))
}

pub fn with_scope_mut<R>(f: impl FnOnce(&mut RefMut<Scope>) -> R) -> R {
    with_scope(|s| f(&mut s.borrow_mut()))
}

pub fn with_node(id: NodeId, f: impl FnOnce(&Node)) {
    with_scope_ref(|s| s.with_node(id, f))
}

pub fn bind_node(f: impl Fn() -> Child) -> Child {
    with_scope(|scope| {
        let parent = { scope.borrow_mut().take_parent() };
        let child = f();
        scope.borrow_mut().set_parent(parent);
        child
    })
}

pub fn view(f: impl Fn() -> Box<dyn View> + 'static) -> Child {
    with_scope_mut(|s| s.view(f))
}

pub fn provide_layer<T: Clone + Any + 'static>(x: T) {
    with_scope_mut(|s| s.provide_layer(x))
}

pub fn use_layer<T: Clone + Any + 'static>() -> T {
    with_scope_ref(|s| s.use_layer::<T>())
}

pub fn use_layer_option<T: Clone + Any + 'static>() -> Option<T> {
    with_scope_ref(|s| s.use_layer_option::<T>())
}

pub fn use_layer_or_default<T: Clone + Default + Any + 'static>() -> T {
    with_scope_mut(|s| s.use_layer_or_default::<T>())
}
