use crate::scope::NodeId;
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};

pub struct Node {
    pub f: Option<Box<dyn Fn() -> Box<dyn Any> + 'static>>,
    pub value: Rc<RefCell<Option<Box<dyn Any>>>>,
    pub stale: Cell<bool>,
    pub dependants: Rc<RefCell<Vec<NodeId>>>,
}

impl Node {
    pub fn create_with_value<V: Any + 'static>(value: V) -> Self {
        Self {
            f: None,
            value: Rc::new(RefCell::new(Some(Box::new(value)))),
            stale: Cell::new(false),
            dependants: Rc::new(RefCell::new(vec![])),
        }
    }

    pub fn create_with_memo<V: Any + 'static>(f: impl Fn() -> V + 'static) -> Self {
        Self {
            f: Some(Box::new(move || Box::new(f()))),
            value: Rc::new(RefCell::new(None)),
            stale: Cell::new(true),
            dependants: Rc::new(RefCell::new(vec![])),
        }
    }

    pub fn with_value<T: 'static, R>(&self, f: impl Fn(&T) -> R) -> R {
        let value = self.value.borrow();
        let value = value.as_ref().expect("Missing value");
        let value = value.downcast_ref::<T>().expect("Incorrect type");
        f(value)
    }

    pub fn with_value_mut<T: Any, R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut value = self.value.borrow_mut();
        let value = value.as_mut().expect("Should have a value");
        let value = value.downcast_mut::<T>().expect("Should have correct type");
        f(value)
    }

    pub fn mark_stale(&self) {
        self.stale.set(true);
        *self.dependants.borrow_mut() = vec![];
    }

    pub fn refresh(&self) {
        *self.value.borrow_mut() = self.f.as_ref().map(|f| f());
        self.stale.set(false);
    }
}

pub trait NodeRef {
    fn get_node(&self) -> NodeId;
}
