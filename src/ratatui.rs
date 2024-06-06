use ratatui::widgets::{StatefulWidgetRef, Widget, WidgetRef};
use std::{any::Any, marker::PhantomData};

use crate::{
    environment::*,
    nodes::{IntoScope, Scope},
    prelude::{ReadSignal, Signal, SignalRead, SignalUpdate},
};

// Child
#[derive(Clone, Copy)]
pub struct RatView(Scope);
impl IntoScope for RatView {
    fn into_scope(&self) -> Scope {
        self.0
    }
}

// WidgetRef Wrapper type
pub struct WidgetNode(Box<dyn Fn(ratatui::prelude::Rect, &mut ratatui::prelude::Buffer)>);

impl WidgetRef for RatView {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let signal = ReadSignal::<WidgetNode>(self.into_scope(), PhantomData);
        signal.with_untracked(|v| v.0(area, buf));
    }
}

pub fn render(
    f: impl Fn(ratatui::prelude::Rect, &mut ratatui::prelude::Buffer) + 'static,
) -> RatView {
    let id = with_runtime(|s| s.create_value_node(Box::new(WidgetNode(Box::new(f)))));
    RatView(id)
}

pub fn widget<V: Widget + Any>(f: impl Fn() -> V + 'static) -> RatView {
    let node = Box::new(WidgetNode(Box::new(move |area, buf| f().render(area, buf))));
    let id = with_runtime(|r| r.create_value_node(node));
    RatView(id)
}

pub fn widget_ref<V: WidgetRef + Any>(f: impl Fn() -> V + 'static) -> RatView {
    let memo = create_memo(move || {
        let w = f();
        WidgetNode(Box::new(move |area, buf| w.render_ref(area, buf)))
    });
    RatView(memo.0)
}

impl<T> From<ReadSignal<T>> for RatView
where
    T: WidgetRef + 'static,
{
    fn from(value: ReadSignal<T>) -> Self {
        render(move |area, buf| {
            value.with_untracked(|x| x.render_ref(area, buf));
        })
    }
}

impl<T> From<Signal<T>> for RatView
where
    T: WidgetRef + 'static,
{
    fn from(value: Signal<T>) -> Self {
        render(move |area, buf| {
            value.with_untracked(|x| x.render_ref(area, buf));
        })
    }
}

pub fn statefull_widget_ref<S, State, V>(state: S, f: impl Fn() -> V + 'static) -> RatView
where
    S: SignalUpdate<State> + Clone + 'static,
    State: Clone + Any + 'static,
    V: StatefulWidgetRef<State = State> + 'static,
{
    let memo = create_memo(move || {
        let widget = f();
        let state = state.clone();
        WidgetNode(Box::new(move |area, buf| {
            state
                .clone()
                .update_silent(|state| StatefulWidgetRef::render_ref(&widget, area, buf, state))
        }))
    });
    RatView(memo.0)
}

#[macro_export]
macro_rules! widget {
    ($expr:expr) => {{
        render(move |a, b| $expr(a, b))
    }};
    {$($stmt:tt)*} => {{
        widget(move || {$($stmt)*})
    }};
}
