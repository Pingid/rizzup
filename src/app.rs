use std::any::Any;

use super::{node::Child, scope::provide_layer};

pub struct App {
    f: Option<Box<dyn FnOnce() -> Child>>,
}

impl App {
    pub fn new(f: impl FnOnce() -> Child + 'static) -> Self {
        Self {
            f: Some(Box::new(f)),
        }
    }
    pub fn with_layer<T: Clone + Any + 'static>(mut self, layer: &T) -> Self {
        let original = self.f.take().expect("Should always have render fn");
        let layer = layer.clone();
        let next = Box::new(move || {
            provide_layer(layer);
            original()
        });
        self.f.replace(next);
        self
    }
    pub fn render(mut self) -> Child {
        let f = self.f.take().expect("Should always have render fn");
        f()
    }
}
