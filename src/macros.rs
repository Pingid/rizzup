#[macro_export]
macro_rules! reactive {
    (signal: $body:expr) => {{
        create_signal($body)
    }};
    (memo: $body:expr) => {{
        create_memo(move || $body)
    }};
    (memo: $($m:ident($s:ident)),*: $body:expr) => {{
        let ($($s),*) = ($($s.$m()),*);
        create_memo(move || $body)
    }};
    (selector: $body:expr) => {{
        create_selector(move || $body)
    }};
    (selector: $($m:ident($s:ident)),*: $body:expr) => {{
        let ($($s),*) = ($($s.$m()),*);
        create_selector(move || $body)
    }};
    (receiver: { $($pt:pat => $exp:expr,)* }) => {{
        on(move |ev| {
            match ev {
                $($pt => { $exp; },)*
                _ => {},
            };
        });
    }};
    (receiver: $($m:ident($s:ident)),*: { $($pt:pat => $exp:expr,)* }) => {{
        let ($($s),*) = ($($s.$m()),*);
        on(move |ev| {
            match ev {
                $($pt => { $exp; },)*
                _ => {},
            };
        });
    }};
}
pub use reactive;
