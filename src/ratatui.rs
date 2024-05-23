use crate::{
    node::{NodeId, NodeRef},
    signal::{RwSignal, SignalWriter},
};
use ratatui::widgets::{StatefulWidgetRef, WidgetRef};
use std::any::Any;

// WidgetRef Wrapper type
pub struct WidgetNode(Box<dyn Fn(ratatui::prelude::Rect, &mut ratatui::prelude::Buffer)>);

// Child
#[derive(Clone, Copy)]
pub struct Child(pub NodeId);

impl NodeRef for Child {
    fn node_id_ref(&self) -> &NodeId {
        &self.0
    }
}

impl WidgetRef for Child {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        self.0.with_value_ref(|n: &WidgetNode| n.0(area, buf));
    }
}

pub fn view_fn(
    f: impl Fn(ratatui::prelude::Rect, &mut ratatui::prelude::Buffer) + 'static,
) -> Child {
    let id = NodeId::from_value(WidgetNode(Box::new(f)));
    Child(id)
}

pub fn view_widget<V: WidgetRef + Any>(f: impl Fn() -> V + 'static) -> Child {
    let id = NodeId::from_memo(move || {
        let w = Box::new(f());
        WidgetNode(Box::new(move |area, buf| w.render_ref(area, buf)))
    });
    Child(id)
}

pub fn view_statefull_widget<
    V: StatefulWidgetRef<State = S> + Any,
    S: std::fmt::Debug + Clone + Any + 'static,
>(
    s: RwSignal<S>,
    f: impl Fn() -> V + 'static,
) -> Child {
    let id = NodeId::from_memo(move || {
        let widget = f();
        let state = s.clone();
        WidgetNode(Box::new(move |area, buf| {
            RwSignal::update(&state, |state| widget.render_ref(area, buf, state))
        }))
    });
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
