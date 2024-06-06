use std::marker::PhantomData;

use crate::environment::with_runtime;
use crate::nodes::{IntoScope, Scope};

pub trait SignalRead<T: 'static>: IntoScope {
    fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        with_runtime(|r| r.nodes.with_value::<T, R>(self.into_scope(), |n| f(n)))
    }
    fn with<R>(&self, f: impl FnOnce(&T) -> R) -> Option<R> {
        with_runtime(|r| r.track_dependant(self.into_scope()));
        self.with_untracked(f)
    }
}

pub trait SignalGet<T: 'static>: SignalRead<T> {
    fn get_untracked(&self) -> T;
    fn get(&self) -> T;
}

pub trait SignalUpdate<T: 'static>: IntoScope {
    fn update_silent(&self, f: impl FnOnce(&mut T)) {
        with_runtime(|r| r.nodes.with_value(self.into_scope(), f));
    }
    fn update(&self, f: impl FnOnce(&mut T)) {
        self.update_silent(f);
        with_runtime(|r| r.update_dependants(self.into_scope()));
    }
}

pub trait SignalSet<T: 'static>: SignalUpdate<T> {
    fn set_silent(&self, new: T) {
        self.update_silent(|v| *v = new);
    }
    fn set(&self, new: T) {
        self.update(|v| *v = new)
    }
}

macro_rules! impl_signal_get {
    ($iden:ident) => {
        impl<T: std::fmt::Debug + Clone + 'static> std::fmt::Debug for $iden<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let value = self.get_untracked();
                f.debug_struct(stringify!($type))
                    .field("value", &value)
                    .finish()
            }
        }

        impl<T: Clone + 'static> SignalGet<T> for $iden<T> {
            default fn get_untracked(&self) -> T {
                self.with_untracked(|v| v.clone())
                    .expect(format!("Node {:?} has been disposed", self.into_scope()).as_str())
            }
            default fn get(&self) -> T {
                self.with(|v| v.clone())
                    .expect(format!("Node {:?} has been disposed", self.into_scope()).as_str())
            }
        }

        impl<T: Copy + 'static> SignalGet<T> for $iden<T> {
            fn get_untracked(&self) -> T {
                self.with_untracked(|v| *v)
                    .expect(format!("Node {:?} has been disposed", self.into_scope()).as_str())
            }
            fn get(&self) -> T {
                self.with(|v| *v)
                    .expect(format!("Node {:?} has been disposed", self.into_scope()).as_str())
            }
        }
    };
}

#[derive(Clone, Copy)]
pub struct ReadSignal<T>(pub Scope, pub PhantomData<T>);

impl<T> IntoScope for ReadSignal<T> {
    fn into_scope(&self) -> Scope {
        self.0
    }
}
impl<T: 'static> SignalRead<T> for ReadSignal<T> {}
impl_signal_get!(ReadSignal);

#[derive(Clone, Copy)]
pub struct WriteSignal<T>(pub Scope, pub PhantomData<T>);

impl<T> IntoScope for WriteSignal<T> {
    fn into_scope(&self) -> Scope {
        self.0
    }
}
impl<T: 'static> SignalUpdate<T> for WriteSignal<T> {}
impl<T: 'static> SignalSet<T> for WriteSignal<T> {}

#[derive(Clone, Copy)]
pub struct Signal<T>(pub Scope, pub PhantomData<T>);
impl<T> IntoScope for Signal<T> {
    fn into_scope(&self) -> Scope {
        self.0
    }
}
impl<T: 'static> SignalRead<T> for Signal<T> {}
impl_signal_get!(Signal);
impl<T: 'static> SignalUpdate<T> for Signal<T> {}
impl<T: 'static> SignalSet<T> for Signal<T> {}
