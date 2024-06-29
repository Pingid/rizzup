use std::{any::Any, cell::RefCell, marker::PhantomData, rc::Rc};

use crate::scope::{use_layer, use_layer_or_default, with_scope_mut, NodeId};
use slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub struct ValueId;
}

#[derive(Default, Clone)]
pub struct State {
    states: Rc<RefCell<SlotMap<ValueId, (NodeId, Box<dyn Any>)>>>,
}

impl State {
    pub fn get_value<T: Clone + 'static>(&self, id: ValueId) -> T {
        let states = self.states.borrow();
        let value = states.get(id).map(|x| x.1.downcast_ref::<T>()).flatten();
        value.expect("Bad type").clone()
    }
    pub fn set_value<T: Clone + 'static>(&self, id: ValueId, value: T) {
        let mut states = self.states.borrow_mut();
        if let Some(v) = states.get_mut(id) {
            v.1 = Box::new(value)
        }
    }
    pub fn update_value<T: Clone + 'static>(&self, id: ValueId, f: impl FnOnce(&mut T)) {
        let mut states = self.states.borrow_mut();
        let value = states
            .get_mut(id)
            .map(|x| x.1.downcast_mut::<T>())
            .flatten();
        if let Some(v) = value {
            f(v)
        }
    }
}

#[derive(Clone, Copy)]
pub struct StateRef<T>(ValueId, PhantomData<T>);

impl<T: Clone + 'static> StateRef<T> {
    pub fn get(&self) -> T {
        use_layer::<State>().get_value::<T>(self.0)
    }
    pub fn set(&self, value: T) {
        use_layer::<State>().set_value::<T>(self.0, value)
    }
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        use_layer::<State>().update_value::<T>(self.0, f)
    }
}

impl State {
    pub fn use_state<T: Any + 'static>(&self, v: T) -> StateRef<T> {
        let id = with_scope_mut(|s| {
            self.states
                .borrow_mut()
                .insert((s.get_parent(), Box::new(v)))
        });
        StateRef(id, PhantomData)
    }
}

pub fn use_state<T: Any + Clone + 'static>(value: T) -> StateRef<T> {
    use_layer_or_default::<State>().use_state(value)
}
