use std::any::{type_name, Any};

use crate::{nodes::ReactiveNode, runtime::Runtime, signal::*};

thread_local! {
    static RUNTIME: Runtime = Runtime::default();
}

pub fn with_runtime<R>(f: impl FnOnce(&Runtime) -> R) -> R {
    RUNTIME.with(f)
}

pub fn create_tracking_scope<R>(f: impl FnOnce() -> R) -> R {
    with_runtime(|r| {
        let scope = r.nodes.insert(ReactiveNode::default());
        let previous = r.tracker.replace(Some(scope));
        let value = f();
        r.tracker.replace(previous);
        r.cleanup_child_scope(scope);
        r.nodes.dispose(scope);
        value
    })
}

/// Cleanup
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    with_runtime(|r| r.add_cleanup(r.get_current_scope(), f))
}

/// Context
pub fn provide_context<T: Clone + Any + 'static>(x: T) {
    with_runtime(|r| r.context.provide_context(r.get_current_scope(), x))
}

pub fn use_context_option<T: Clone + Any + 'static>() -> Option<T> {
    with_runtime(|r| r.get_context(r.get_current_scope()))
}

pub fn use_context<T: Clone + Any + 'static>() -> T {
    let p = with_runtime(|r| r.get_current_scope());
    use_context_option()
        .expect(format!("Missing {} in parent scope {:?}", type_name::<T>(), p,).as_str())
}

/// Messages
pub fn on<T: 'static>(f: impl Fn(&T) + 'static) {
    with_runtime(|r| r.recievers.create_reciever(r.get_current_scope(), f));
}

pub fn send_boxed(message: &Box<dyn Any>) {
    with_runtime(|r| r.send(r.get_current_scope(), message));
}

pub fn send<T: Any + 'static>(message: T) {
    let message = Box::new(message) as Box<dyn Any>;
    send_boxed(&message);
}

/// Signals
pub fn create_signal<T: 'static>(value: T) -> Signal<T> {
    let scope = with_runtime(|r| r.create_value_node(Box::new(value)));
    Signal(scope, std::marker::PhantomData)
}

pub fn create_memo<T: 'static>(f: impl Fn() -> T + 'static) -> ReadSignal<T> {
    let scope = with_runtime(|r| r.create_cb_node(move |_| Some(Box::new(f()))));
    ReadSignal(scope, std::marker::PhantomData)
}

pub fn create_selector<T: std::fmt::Debug + PartialEq + 'static>(
    f: impl Fn() -> T + 'static,
) -> ReadSignal<T> {
    let scope = with_runtime(|r| {
        r.create_cb_node(move |previous| {
            let previous = previous.map(|v| v.downcast_ref::<T>()).flatten();
            let next = f();
            match Some(&next) != previous {
                true => Some(Box::new(next)),
                false => None,
            }
        })
    });
    ReadSignal(scope, std::marker::PhantomData)
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use crate::environment::{create_tracking_scope, with_runtime};

    use super::*;

    fn test_signal_memo_dependancy() {
        let sig = create_signal("Foo");
        let m1 = create_memo(move || sig.get().to_uppercase());
        let m1_c = m1.clone();
        let m2 = create_memo(move || m1_c.get().to_lowercase());
        assert_eq!(sig.get(), "Foo");
        assert_eq!(m1.get(), "FOO");
        assert_eq!(m2.get(), "foo");
        sig.set("Bar");
        assert_eq!(sig.get(), "Bar");
        assert_eq!(m1.get(), "BAR");
        assert_eq!(m2.get(), "bar");
    }

    fn test_runtime_cleanup_up() {
        with_runtime(|r| {
            assert_eq!(r.nodes.0.borrow().len(), 0);
            assert_eq!(r.cleanup.borrow().len(), 0);
        });
    }

    #[test]
    fn test_signal_dependancy_tracking() {
        create_tracking_scope(|| test_signal_memo_dependancy());
        test_runtime_cleanup_up()
    }

    #[test]
    fn test_signal_dependancy_nested() {
        create_tracking_scope(|| {
            create_memo(|| test_signal_memo_dependancy());
        });
        test_runtime_cleanup_up()
    }

    #[test]
    fn test_signal_dependancy_tracks_latest() {
        create_tracking_scope(|| {
            let count = Rc::new(Cell::new(0));
            let trigger = create_signal(0);
            let s1 = create_signal("foo");
            let s2 = create_signal("bar");

            let count_c = count.clone();
            let m = create_memo(move || {
                count_c.set(count_c.get() + 1);
                return match trigger.get() % 2 == 0 {
                    true => s1.get(),
                    false => s2.get(),
                };
            });

            assert_eq!(m.get(), "foo");
            trigger.set(1);
            assert_eq!(m.get(), "bar");
            assert_eq!(count.get(), 2);
            s1.set("FOO");
            assert_eq!(
                count.get(),
                2,
                "Doesn't rerender due to no longer depending on s1"
            );
        });
        test_runtime_cleanup_up()
    }

    #[test]
    fn test_signal_selector() {
        create_tracking_scope(|| {
            let count = Rc::new(Cell::new(0));
            let source = create_signal((0, 0));
            let s1 = create_selector(move || Signal::<(i32, i32)>::get(&source).0);

            let count_c = count.clone();
            let m = create_memo(move || {
                count_c.set(count_c.get() + 1);
                s1.get()
            });

            assert_eq!(count.get(), 1);
            source.set((0, 10));

            // Doesn't update because selector equals previous
            assert_eq!(m.get(), 0);
            assert_eq!(count.get(), 1,);

            source.set((10, 10));
            assert_eq!(m.get(), 10);
            assert_eq!(count.get(), 2);
        })
    }

    #[test]
    fn test_cleanup() {
        create_tracking_scope(|| {
            let trig = create_signal(0);
            let count = create_signal(0);

            create_memo(move || match trig.get() {
                0 => create_memo(move || on_cleanup(move || count.update(|v| *v += 1))),
                _ => create_memo(move || {}),
            });
            trig.set(1);
            assert_eq!(count.get(), 1, "executed cleanup");
        })
    }

    #[test]
    fn test_recievers_cleaned_up() {
        create_tracking_scope(|| {
            let recieved = create_signal(0);
            let trig = create_signal(0);

            create_memo(move || match trig.get() {
                0 => {
                    on(move |ev: &usize| recieved.set(*ev));
                    create_memo(move || on(move |ev: &usize| recieved.set(*ev)));
                }
                _ => {}
            });

            send(10 as usize);
            assert_eq!(recieved.get(), 10);
            trig.set(1);
            send(20 as usize);
            assert_eq!(recieved.get(), 10, "");
            trig.set(0);
            send(20 as usize);
            assert_eq!(recieved.get(), 20, "");
        })
    }

    #[test]
    fn test_recievers_send_deep() {
        create_tracking_scope(|| {
            let recieved = create_signal(0);

            create_memo(move || {
                create_memo(move || {
                    create_memo(move || on(move |ev: &usize| recieved.update(|v| *v += ev)))
                })
            });

            send(10 as usize);
            assert_eq!(recieved.get(), 10);
            send(10 as usize);
            assert_eq!(recieved.get(), 20);
        })
    }

    #[test]
    fn test_recievers_cleanup_on_send() {
        fn even(f: impl Fn((String, usize)) + 'static) {
            on(move |ev: &usize| f(("even".to_string(), *ev)))
        }
        fn odd(f: impl Fn((String, usize)) + 'static) {
            on(move |ev: &usize| f(("odd".to_string(), *ev)))
        }
        create_tracking_scope(|| {
            let sig = create_signal(0);
            let result = create_signal(("init".to_string(), 0));

            let result_c = result.clone();
            on(move |ev: &usize| sig.set(*ev));

            create_memo(move || {
                let (result_a, result_b) = (result_c.clone(), result_c.clone());
                match sig.get() % 2 == 0 {
                    true => even(move |a| result_a.set(a)),
                    false => odd(move |b| result_b.set(b)),
                }
            });

            send(10 as usize);
            assert_eq!(result.get(), ("even".to_string(), 10));
            send(11 as usize);
            assert_eq!(result.get(), ("odd".to_string(), 11));
            send(10 as usize);
            assert_eq!(result.get(), ("even".to_string(), 10));
        })
    }
}

// fn debug_runtime(r: &Runtime) {
//     let nodes = r.nodes.0.borrow();
//     println!("\n");
//     for n in nodes.iter() {
//         println!("{:?}", n);
//     }
//     println!("\n");
// }
