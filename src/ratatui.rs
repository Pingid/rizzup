use crate::{
    node::{Node, NodeRef},
    scope::{with_scope, NodeId},
};
use ratatui::widgets::WidgetRef;
use std::any::Any;

pub struct ViewWidget(Box<dyn WidgetRef>);

impl ViewWidget {
    pub fn new<T: WidgetRef + Any>(inner: T) -> Self {
        Self(Box::new(inner))
    }
}

impl WidgetRef for ViewWidget {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.0.render_ref(area, buf)
    }
}

#[derive(Clone, Copy)]
pub struct Child(pub NodeId);

impl NodeRef for Child {
    fn get_node(&self) -> NodeId {
        self.0
    }
}

impl WidgetRef for Child {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.0.update();
        self.0.with_value(|n: &ViewWidget| n.render_ref(area, buf));
    }
}

pub fn view_widget<V: WidgetRef + Any>(f: impl Fn() -> V + 'static) -> Child {
    let node = Node::create_with_memo(move || ViewWidget::new(f()));
    let id = with_scope(|s| s.insert_node(node));
    Child(id)
}

struct ViewRenderWidget(Box<dyn Fn(ratatui::prelude::Rect, &mut ratatui::prelude::Buffer)>);
impl WidgetRef for ViewRenderWidget {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.0(area, buf)
    }
}

pub fn view_render(
    f: impl Fn(ratatui::prelude::Rect, &mut ratatui::prelude::Buffer) + 'static,
) -> Child {
    let node = Node::create_with_value(ViewRenderWidget(Box::new(f)));
    let id = with_scope(|s| s.insert_node(node));
    Child(id)
}

#[macro_export]
macro_rules! widget {
    ($expr:expr) => {{
        view_render(move |a, b| $expr(a, b))
    }};
    {$($stmt:tt)*} => {{
        view_widget(move || {$($stmt)*})
    }};
}
pub use widget;
