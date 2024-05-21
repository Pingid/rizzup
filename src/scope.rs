use super::node::Node;
use slotmap::{new_key_type, SlotMap};
use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

thread_local! {
    static SCOPE: Scope = Scope::default();
}

new_key_type! {
    pub struct NodeId;
}

#[derive(Default)]
pub struct Scope {
    current: Cell<Option<NodeId>>,
    nodes: Rc<RefCell<SlotMap<NodeId, Node>>>,
    layers: Rc<RefCell<HashMap<TypeId, Box<dyn Any>>>>,
}

impl NodeId {
    pub fn with<R>(&self, f: impl FnOnce(&Node) -> R) -> Option<R> {
        with_scope(|s| s.nodes.borrow().get(self.clone()).map(|x| f(x)))
    }

    pub fn with_value<T: 'static, R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        self.with(|n| {
            let value = n.value.borrow();
            let value = value.as_ref().map(|v| v.downcast_ref::<T>());
            value.flatten().map(f)
        })
        .flatten()
    }

    pub fn update(&self) {
        let stale = self.with(|n| n.stale.get()).unwrap_or(false);
        if stale {
            let previous = with_scope(|s| s.set_current_node(Some(self.clone())));
            self.with(|n| n.refresh());
            with_scope(|s| s.set_current_node(previous));
        }
    }
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
