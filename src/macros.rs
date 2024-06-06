#[macro_export]
macro_rules! match_on {
    ($tp:ty, $pt:pat => $exp:expr) => {{
        on::<$tp>(move |ev| {
            match ev {
                $pt => $exp,
                _ => {},
            };
        });
    }};
    ($tp:ty, { $($pt:pat => $exp:expr,)* }) => {{
        on::<$tp>(move |ev| {
            match ev {
                $($pt => $exp,)*
                _ => {},
            };
        });
    }};
}

pub use match_on;

#[macro_export]
macro_rules! create_memo {
    ([$($s:ident),*], $body:expr) => {{
        let ($($s),*) = ($($s.clone()),*);
        create_memo(move || $body)
    }};
}
pub use create_memo;

#[macro_export]
macro_rules! create_selector {
    ([$($s:ident),*], $body:expr) => {{
        let ($($s),*) = ($($s.clone()),*);
        create_selector(move || $body)
    }};
}
pub use create_selector;

#[macro_export]
macro_rules! on {
    ($pt:pat => $exp:expr) => {{
        on(move |ev| {
            match ev {
                $pt => $exp,
                _ => {},
            };
        });
    }};
    ({ $($pt:pat => $exp:expr,)* }) => {{
        on(move |ev| {
            match ev {
                $($pt => $exp,)*
                _ => {},
            };
        });
    }};
    ([$($s:ident),*],{ $($pt:pat => $exp:expr,)* }) => {{
        let ($($s),*) = ($($s.clone()),*);
        on(move |ev| {
            match ev {
                $($pt => $exp,)*
                _ => {},
            };
        });
    }};
}

pub use on;

#[cfg(test)]
mod tests {
    use crate::environment::*;
    use crate::signal::*;

    #[test]
    fn test_memo_macro() {
        create_tracking_scope(|| {
            let s1 = create_signal("one".to_string());
            let s2 = create_signal("two".to_string());

            let one = create_memo!([s1, s2], format!("{} {}", s1.get(), s2.get()));
            let two = create_memo!([s1, s2], format!("{}-{}", s1.get(), s2.get()));

            assert_eq!(one.get(), "one two");
            assert_eq!(two.get(), "one-two");
        })
    }
}
