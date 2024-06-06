use slotmap::{new_key_type, SlotMap};
use std::{any::Any, cell::RefCell, collections::HashSet, rc::Rc};

new_key_type! {
    pub struct Scope;
}

pub trait IntoScope {
    fn into_scope(&self) -> Scope;
}

impl IntoScope for Scope {
    fn into_scope(&self) -> Scope {
        *self
    }
}

#[derive(Default, Debug)]
pub struct ReactiveNode {
    /// Any value
    pub(crate) value: Option<Box<dyn Any>>,
    /// Any Fn that accepts a Box<dyn Any> and returns a Box<dyn Any>
    pub(crate) callback: Option<Callback>,
    /// Node of the parent scope
    pub(crate) parent: Option<Scope>,
    /// Nodes who depend on the value from this node
    pub(crate) dependants: HashSet<Scope>,
}

#[derive(Default, Debug, Clone)]
pub struct ReactiveNodes(pub Rc<RefCell<SlotMap<Scope, ReactiveNode>>>);

impl ReactiveNodes {
    pub(crate) fn insert(&self, node: ReactiveNode) -> Scope {
        self.0.borrow_mut().insert(node)
    }

    pub(crate) fn create(
        &self,
        scope: Scope,
        cb: Option<Callback>,
        value: Option<Box<dyn Any>>,
    ) -> Scope {
        let mut node = ReactiveNode::default();
        node.parent = Some(scope);
        node.callback = cb;
        node.value = value;
        self.insert(node)
    }

    pub(crate) fn with<R>(&self, id: Scope, f: impl FnOnce(&mut ReactiveNode) -> R) -> Option<R> {
        self.0.borrow_mut().get_mut(id).map(|n| f(n))
    }

    pub(crate) fn get_children(&self, scope: Scope) -> Vec<Scope> {
        let nodes = self.0.borrow();
        let children = nodes.iter().filter(|x| x.1.parent == Some(scope));
        let children = children.map(|x| x.0);
        children.collect()
    }

    pub(crate) fn get_scope_children(&self, scope: Scope) -> Vec<Scope> {
        let mut all = vec![];
        for child in self.get_children(scope) {
            all.extend(self.get_scope_children(child));
            all.push(child)
        }
        all
    }

    pub(crate) fn get_parent(&self, id: Scope) -> Option<Scope> {
        self.0.borrow().get(id).map(|n| n.parent).flatten()
    }

    pub(crate) fn take_recompute(&self, scope: Scope) -> (Option<Callback>, Option<Box<dyn Any>>) {
        self.with(scope, |n| match n.callback.is_some() {
            true => (n.callback.take(), n.value.take()),
            false => (None, None),
        })
        .unwrap_or((None, None))
    }

    /// Update value if there is a new value and return dependants otherwise set value to previous
    pub(crate) fn update(
        &self,
        scope: Scope,
        cb: Callback,
        value: Option<Box<dyn Any>>,
        previous: Option<Box<dyn Any>>,
    ) -> Vec<Scope> {
        self.with(scope, |n| {
            n.callback = Some(cb);
            if let Some(val) = value {
                n.value = Some(val);
                return n.dependants.drain().collect::<Vec<_>>();
            }
            n.value = previous;
            vec![]
        })
        .expect("Disposed")
    }

    pub(crate) fn take_dependants(&self, scope: Scope) -> Vec<Scope> {
        self.with(scope, |n| n.dependants.drain().collect())
            .unwrap_or_default()
    }

    pub(crate) fn remove_scope_from_dependants(&self, scope: Scope) {
        let mut nodes = self.0.borrow_mut();
        for (_, node) in nodes.iter_mut() {
            node.dependants.remove(&scope);
        }
    }

    pub(crate) fn dispose(&self, id: Scope) {
        self.remove_scope_from_dependants(id);
        self.0.borrow_mut().remove(id);
    }

    pub(crate) fn with_value<T: 'static, R>(
        &self,
        scope: Scope,
        f: impl FnOnce(&mut T) -> R,
    ) -> Option<R> {
        let value = self.with(scope, |n| n.value.take()).flatten();
        let mut result = None;
        if let Some(Ok(mut value)) = value.map(|x| x.downcast::<T>()) {
            result = Some(f(&mut value));
            self.with(scope, |n| n.value.replace(value));
        }
        return result;
    }
}

pub(crate) struct Callback(pub(crate) Box<dyn Fn(Option<&Box<dyn Any>>) -> Option<Box<dyn Any>>>);
impl std::fmt::Debug for Callback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fn")
    }
}
