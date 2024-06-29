use crate::{
    node::{Child, Node},
    scope::with_node,
};

use ratatui::widgets::WidgetRef;

pub trait View: WidgetRef {}

impl<T: WidgetRef> View for T {}

impl WidgetRef for Node {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.update();
        if let Some(v) = self.value.borrow().as_ref() {
            v.render_ref(area, buf)
        }
    }
}

impl WidgetRef for Child {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        with_node(self.id, |n| n.render_ref(area, buf))
    }
}
