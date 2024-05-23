use crate::scope::with_scope;
use slotmap::new_key_type;
use std::{any::Any, cell::RefCell, rc::Rc};

new_key_type! {
    pub struct NodeId;
}

pub struct Node {
    pub f: Option<Box<dyn Fn() -> Box<dyn Any> + 'static>>,
    pub value: Rc<RefCell<Option<Box<dyn Any>>>>,
    pub stale: bool,
    pub parent: Option<NodeId>,
    pub dependants: Vec<NodeId>,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn some<T>(x: &Option<T>) -> &'static str {
            match x {
                Some(_) => "Some",
                None => "None",
            }
        }
        let value = self.value.borrow();
        write!(
            f,
            "Node {{ f: {}, value: {}, stale: {}, parent {:?}, dependants: {:?} }}",
            some(&self.f),
            some(&value),
            self.stale,
            &self.parent,
            self.dependants
        )
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            f: None,
            value: Rc::new(RefCell::new(None)),
            stale: false,
            parent: None,
            dependants: vec![],
        }
    }
}

pub trait NodeRef {
    fn node_id_ref(&self) -> &NodeId;
    fn node_id(&self) -> NodeId {
        *self.node_id_ref()
    }
}

impl NodeId {
    pub fn from_value<V: Any + 'static>(value: V) -> NodeId {
        let mut node = Node::default();
        node.parent = with_scope(|s| s.get_current_node());
        node.value.borrow_mut().replace(Box::new(value));
        with_scope(|s| s.insert_node(node))
    }

    pub fn from_memo<T: Any + 'static>(f: impl Fn() -> T + 'static) -> NodeId {
        let mut node = Node::default();
        node.parent = with_scope(|s| s.get_current_node());
        let id = with_scope(|s| s.insert_node(node));

        let f = Box::new(move || {
            let previous = with_scope(|s| s.set_current_node(Some(id.clone())));
            let value = f();
            with_scope(|s| s.set_current_node(previous));
            Box::new(value) as Box<dyn Any>
        });

        id.with_mut(|n| {
            n.f = Some(Box::new(f));
            n.stale = true
        });

        id
    }

    pub fn with<R>(&self, f: impl FnOnce(&Node) -> R) -> R {
        with_scope(move |s| s.with_node(self.clone(), f))
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut Node) -> R) -> R {
        with_scope(move |s| s.with_node_mut(self.clone(), f))
    }

    pub fn with_value<T: Any + 'static, R>(&self, f: impl FnOnce(T) -> T) {
        if self.with(|s| s.stale) {
            self.update();
        }
        let value = self
            .with_mut(move |n| n.value.take())
            .expect("Missing value");
        let value = value.downcast::<T>().expect("Incorrect type");
        let value = Box::new(f(*value)) as Box<dyn Any>;
        self.with_mut(move |n| n.value.replace(Some(value)));
    }

    pub fn with_value_ref<T: 'static, R>(&self, f: impl FnOnce(&T) -> R) -> R {
        if self.with(|s| s.stale) {
            self.update();
        }
        let value = self.with(move |n| n.value.clone());
        let value = value.borrow();
        let value = value.as_ref().expect("Missing value");
        let value = value.downcast_ref::<T>().expect("Incorrect type");
        f(value)
    }

    pub fn with_value_mut<T: 'static, R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        if self.with(|s| s.stale) {
            self.update();
        }
        let value = self.with(move |n| n.value.clone());
        let mut value = value.borrow_mut();
        let value = value.as_mut().expect("Missing value");
        let value = value.downcast_mut::<T>().expect("Incorrect type");
        f(value)
    }

    pub fn mark_stale(&self) {
        let deps = self.with_mut(|n| {
            n.stale = true;
            std::mem::replace(&mut n.dependants, Vec::new())
        });
        for dep in deps {
            dep.mark_stale()
        }
    }

    pub fn update(&self) {
        let f = with_scope(|s| s.with_node_mut(self.clone(), |n| n.f.take()));

        if let Some(f) = f {
            let value = f();
            with_scope(|s| {
                s.with_node_mut(self.clone(), |n| {
                    n.f.replace(f);
                    n.value.borrow_mut().replace(value);
                    n.stale = false
                })
            });
        }
    }

    pub fn add_dependancy(&self, id: NodeId) {
        self.with_mut(|n| n.dependants.push(id))
    }
}
