use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{
    scope::{with_node, NodeId},
    view::View,
};

#[derive(Default)]
pub struct Node {
    pub update_fn: Option<Box<dyn Fn() -> Box<dyn View> + 'static>>,
    pub value: Rc<RefCell<Option<Box<dyn View>>>>,
    pub requires_update: Cell<bool>,
}

impl Node {
    pub fn set_requires_update(&self, requires_update: bool) {
        self.requires_update.set(requires_update);
    }

    pub fn set_render_fn(&mut self, f: impl Fn() -> Box<dyn View> + 'static) {
        self.update_fn.replace(Box::new(f));
    }

    pub fn update(&self) {
        let mut view = self.value.borrow_mut();
        if let Some(update_fn) = &self.update_fn {
            if self.requires_update.get() || view.is_none() {
                view.replace(update_fn());
            };
        }
    }
}

pub fn set_requires_update(id: NodeId, value: bool) {
    with_node(id, |c| c.set_requires_update(value))
}

#[derive(Clone, Copy)]
pub struct Child {
    pub id: NodeId,
}

impl Child {
    pub fn new(id: NodeId) -> Self {
        Self { id }
    }
}
