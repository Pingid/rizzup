use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};

use slotmap::SecondaryMap;

use crate::{
    context::Contexts,
    nodes::{Callback, ReactiveNodes, Scope},
    recievers::Recievers,
};

#[derive(Default, Clone)]
pub struct Runtime {
    pub tracker: Rc<Cell<Option<Scope>>>,
    pub nodes: ReactiveNodes,
    pub cleanup: Rc<RefCell<SecondaryMap<Scope, Vec<Box<dyn FnOnce()>>>>>,
    pub context: Contexts,
    pub recievers: Recievers,
}

impl Runtime {
    pub fn create_cb_node(
        &self,
        cb: impl Fn(Option<&Box<dyn Any>>) -> Option<Box<dyn Any>> + 'static,
    ) -> Scope {
        let scope = self.get_current_scope();
        let cb = Callback(Box::new(cb));
        let id = self.nodes.add_node(scope, None, None);
        let value = self.with_tracking_scope(id, || cb.0(None));
        self.nodes.with_node(id, |n| {
            n.callback = Some(cb);
            n.value = value
        });
        id
    }

    pub fn create_value_node(&self, value: Box<dyn Any>) -> Scope {
        let scope = self.get_current_scope();
        self.nodes.add_node(scope, None, Some(value))
    }

    pub fn update_dependants(&self, node: Scope) {
        self.recompute(node);
    }

    pub fn track_dependant(&self, scope: Scope) {
        let parent = self.get_current_scope();
        self.nodes.with_node(scope, |n| n.dependants.insert(parent));
    }

    pub fn get_current_scope(&self) -> Scope {
        self.tracker.get().expect("Missing scope")
    }

    pub fn with_tracking_scope<R>(&self, id: Scope, f: impl FnOnce() -> R) -> R {
        let previous = self.tracker.replace(Some(id));
        let result = f();
        self.tracker.replace(previous);
        result
    }

    pub fn recompute(&self, id: Scope) {
        let deps = self.recompute_node(id);
        for dep in deps {
            self.recompute(dep);
        }
    }

    fn recompute_node(&self, id: Scope) -> Vec<Scope> {
        if let (Some(callback), previous_value) = self.nodes.take_node_callback_and_value(id) {
            self.run_cleanups(id);
            self.dispose_of_children(id);
            self.nodes.remove_scope_from_dependants(id);
            self.recievers.dispose(id);

            let new_value = match &previous_value {
                Some(val) => self.with_tracking_scope(id, || callback.0(Some(val))),
                None => self.with_tracking_scope(id, || callback.0(None)),
            };
            return self.nodes.update(id, callback, new_value, previous_value);
        }
        self.nodes.take_dependants(id)
    }

    pub fn add_cleanup(&self, id: Scope, f: impl FnOnce() + 'static) {
        let mut cleanups = self.cleanup.borrow_mut();
        match cleanups.get_mut(id) {
            Some(v) => v.push(Box::new(f)),
            None => {
                cleanups.insert(id, vec![Box::new(f)]);
            }
        }
    }

    pub fn run_cleanups(&self, id: Scope) {
        let mut children = self.nodes.get_node_children_recursive(id);
        children.push(id);
        for child in children {
            for cleanup in self.cleanup.borrow_mut().remove(child).unwrap_or_default() {
                cleanup()
            }
        }
    }

    pub fn dispose_of_children(&self, scope: Scope) {
        for child in self.nodes.get_node_children_recursive(scope) {
            self.recievers.dispose(child);
            self.nodes.dispose(child);
        }
    }

    pub fn cleanup_child_scope(&self, scope: Scope) {
        let children = self.nodes.get_node_children_recursive(scope);
        for child in &children {
            for cleanup in self.cleanup.borrow_mut().remove(*child).unwrap_or_default() {
                cleanup()
            }
        }
        for child in children {
            self.recievers.dispose(child);
            self.nodes.dispose(child);
        }
    }

    pub fn get_context<T: Clone + Any + 'static>(&self, id: Scope) -> Option<T> {
        match self.context.use_context_from_scope(id) {
            Some(v) => Some(v),
            None => match self.nodes.get_parent(id) {
                Some(id) => self.get_context(id),
                None => None,
            },
        }
    }

    pub fn send(&self, scope: Scope, value: &Box<dyn Any>, deep: bool) {
        self.recievers.send(scope, value);
        if deep {
            for c in self.nodes.get_node_children_recursive(scope) {
                self.recievers.send(c, value);
            }
        }
    }
}
