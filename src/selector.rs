use std::{any::Any, marker::PhantomData, rc::Rc};

use crate::{
    environment::with_runtime,
    nodes::{IntoScope, Scope},
    signal::{Signal, SignalGet, SignalRead, SignalSet, SignalUpdate},
};

#[derive(Clone)]
pub struct Select<R> {
    id: Scope,
    phantom: PhantomData<R>,
    getter: Rc<Box<dyn Fn(&Box<dyn Any>) -> Box<dyn Any>>>,
    updator: Rc<Box<dyn Fn(&mut Box<dyn Any>) -> Box<&mut dyn Any>>>,
}

impl<R> IntoScope for Select<R> {
    fn into_scope(&self) -> Scope {
        self.id
    }
}

impl<T: 'static> SignalRead<T> for Select<T> {
    fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        let getter = self.getter.clone();
        with_runtime(|r| {
            r.nodes.with_node(self.into_scope(), |n| {
                let value = (*getter)(&n.value.as_ref().unwrap());
                let value = value.downcast_ref::<T>().unwrap();
                f(&value)
            })
        })
    }
    fn with<R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        with_runtime(|r| r.track_dependant(self.into_scope()));
        self.with_untracked(f)
    }
}

impl<T: Clone + 'static> SignalGet<T> for Select<T> {
    default fn get_untracked(&self) -> T {
        self.with_untracked::<T>(|v| v.clone())
            .expect(format!("Node {:?} has been disposed", self.into_scope()).as_str())
    }

    default fn get(&self) -> T {
        self.with::<T>(|v| v.clone())
            .expect(format!("Node {:?} has been disposed", self.into_scope()).as_str())
    }
}

impl<T: Clone + 'static> SignalUpdate<T> for Select<T> {
    fn update_silent(&self, f: impl FnOnce(&mut T)) {
        let updator = self.updator.clone();
        with_runtime(|s| {
            s.nodes.with_node(self.id, |n| {
                f(updator(n.value.as_mut().unwrap())
                    .downcast_mut::<T>()
                    .unwrap())
            })
        });
    }
    fn update(&self, f: impl FnOnce(&mut T)) {
        self.update_silent(f);
        with_runtime(|s| s.update_dependants(self.id));
    }
}

impl<T: Clone + 'static> SignalSet<T> for Select<T> {}

pub fn create_signal_selector<T: 'static, R: Any + 'static>(
    signal: &Signal<T>,
    getter: impl Fn(&T) -> R + 'static,
    updator: impl Fn(&mut T) -> &mut R + 'static,
) -> Select<R> {
    Select {
        id: signal.0,
        phantom: PhantomData,
        getter: Rc::new(Box::new(move |x| {
            Box::new(getter(x.downcast_ref::<T>().unwrap()))
        })),
        updator: Rc::new(Box::new(move |x| {
            Box::new(updator(x.downcast_mut::<T>().unwrap()))
        })),
    }
}

pub fn create_select_selector<T: 'static, R: 'static>(
    selection: &Select<T>,
    getter: impl Fn(&T) -> R + 'static,
    updator: impl Fn(&mut T) -> &mut R + 'static,
) -> Select<R> {
    let s_getter = selection.getter.clone();
    let s_updator = selection.updator.clone();
    Select {
        id: selection.id,
        phantom: PhantomData,
        getter: Rc::new(Box::new(move |x| {
            Box::new(getter(s_getter(x).downcast_ref::<T>().unwrap()))
        })),
        updator: Rc::new(Box::new(move |x| {
            Box::new(updator(s_updator(x).downcast_mut::<T>().unwrap()))
        })),
    }
}
