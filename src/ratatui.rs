use crate::{
    node::{Node, NodeRef},
    scope::{with_scope, NodeId},
};
use ratatui::widgets::WidgetRef;
use std::any::Any;

pub struct Child(Box<dyn WidgetRef>);

impl Child {
    pub fn new<T: WidgetRef + Any>(inner: T) -> Self {
        Self(Box::new(inner))
    }
}

impl WidgetRef for Child {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.0.render_ref(area, buf)
    }
}

#[derive(Clone, Copy)]
pub struct ChildRef(pub NodeId);

impl NodeRef for ChildRef {
    fn get_node(&self) -> NodeId {
        self.0
    }
}

impl WidgetRef for ChildRef {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.0.update();
        self.0.with_value(|n: &Child| n.render_ref(area, buf));
    }
}

pub fn view<V: WidgetRef + Any>(f: impl Fn() -> V + 'static) -> ChildRef {
    let node = Node::create_with_memo(move || Child::new(f()));
    let id = with_scope(|s| s.insert_node(node));
    ChildRef(id)
}
