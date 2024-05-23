use super::node::{Node, NodeId};
use slotmap::{SecondaryMap, SlotMap};
use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

thread_local! {
    static SCOPE: Scope = Scope::default();
}

#[derive(Default)]
pub struct Scope {
    current: Cell<Option<NodeId>>,
    nodes: Rc<RefCell<SlotMap<NodeId, Node>>>,
    layers: Rc<RefCell<HashMap<TypeId, Box<dyn Any>>>>,
    cleanup: Rc<RefCell<SecondaryMap<NodeId, Box<dyn FnOnce()>>>>,
}

impl Scope {
    pub fn insert_node(&self, node: Node) -> NodeId {
        self.nodes.borrow_mut().insert(node)
    }

    pub fn get_current_node(&self) -> Option<NodeId> {
        self.current.get()
    }

    pub fn set_current_node(&self, current: Option<NodeId>) -> Option<NodeId> {
        self.current.replace(current)
    }

    pub fn with_node<R>(&self, id: NodeId, f: impl FnOnce(&Node) -> R) -> R {
        let nodes = self.nodes.borrow();
        let value = nodes.get(id).map(|n| f(n));
        value.expect("Node has been disposed")
    }

    pub fn with_node_mut<R>(&self, id: NodeId, f: impl FnOnce(&mut Node) -> R) -> R {
        let mut nodes = self.nodes.borrow_mut();
        let value = nodes.get_mut(id).map(|n| f(n));
        value.expect("Node has been disposed")
    }

    pub fn provide_layer<T: Clone + Any + 'static>(&self, x: T) {
        self.layers
            .borrow_mut()
            .insert(TypeId::of::<T>(), Box::new(x));
    }

    pub fn use_layer_option<T: Clone + Any + 'static>(&self) -> Option<T> {
        self.layers
            .borrow()
            .get(&TypeId::of::<T>())
            .map(|x| x.downcast_ref::<T>())
            .flatten()
            .map(|x| x.clone())
    }

    pub fn use_layer<T: Clone + Any + 'static>(&self) -> T {
        self.use_layer_option().expect("Missing context")
    }

    pub fn use_layer_or_default<T: Clone + Default + Any + 'static>(&self) -> T {
        {
            let id = TypeId::of::<T>();
            let mut layers = self.layers.borrow_mut();
            if !layers.contains_key(&id) {
                layers.insert(id, Box::new(T::default()));
            }
        }
        use_layer()
    }
}

pub fn with_scope<R>(f: impl FnOnce(&Scope) -> R) -> R {
    SCOPE.with(f)
}

pub fn provide_layer<T: Clone + Any + 'static>(x: T) {
    with_scope(|s| s.provide_layer(x))
}

pub fn use_layer<T: Clone + Any + 'static>() -> T {
    with_scope(|s| s.use_layer::<T>())
}

pub fn use_layer_option<T: Clone + Any + 'static>() -> Option<T> {
    with_scope(|s| s.use_layer_option::<T>())
}

pub fn use_layer_or_default<T: Clone + Default + Any + 'static>() -> T {
    with_scope(|s| s.use_layer_or_default::<T>())
}

pub fn create_scope<T>(f: impl FnOnce() -> T) -> T {
    let id = with_scope(|s| s.insert_node(Node::default()));
    with_scope(|s| s.set_current_node(Some(id)));
    let node = f();
    id.cleanup();
    node
}

impl NodeId {
    pub fn children(&self) -> Vec<NodeId> {
        with_scope(|s| {
            s.nodes
                .borrow()
                .iter()
                .filter(|(_, n)| n.parent == Some(*self))
                .map(|(id, _)| id)
                .collect()
        })
    }

    pub fn cleanup(&self) {
        for child in self.children() {
            child.cleanup()
        }
        if let Some(s) = with_scope(|s| s.cleanup.borrow_mut().remove(*self)) {
            s()
        }
    }
}

pub fn on_cleanup(f: impl FnOnce() + 'static) {
    with_scope(|s| {
        let parent = s.get_current_node().expect("Outside tracking scope");
        s.cleanup.borrow_mut().insert(parent, Box::new(f))
    });
}
