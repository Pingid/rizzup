use crate::{
    node::{Node, NodeRef},
    scope::{provide_layer, with_scope},
    signal::{create_signal, ReadSignal},
};
use std::any::Any;

pub struct App<T> {
    f: Option<Box<dyn FnOnce() -> T>>,
}

impl<T: NodeRef + 'static> App<T> {
    pub fn new<P: Clone + 'static>(f: impl FnOnce(ReadSignal<P>) -> T + 'static, props: P) -> Self {
        Self {
            f: Some(Box::new(move || {
                let (props, _) = create_signal(props);
                f(props)
            })),
        }
    }
    pub fn with_layer<L: Clone + Any + 'static>(mut self, layer: &L) -> Self {
        let original = self.f.take().expect("Should always have render fn");
        let layer = layer.clone();
        let next = Box::new(move || {
            provide_layer(layer);
            original()
        });
        self.f.replace(next);
        self
    }

    pub fn render(mut self) -> T {
        let f = self.f.take().expect("Should always have render fn");
        with_scope(|s| s.set_current_node(Some(s.insert_node(Node::create_with_value(())))));
        f()
    }
}
