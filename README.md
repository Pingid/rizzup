# Reactive TUI component library
This library heavily employs reactive signals akin to those found in libraries like [leptos](https://github.com/leptos-rs/leptos) or [dioxus](https://github.com/dioxuslabs/dioxus). For rendering, it currently interfaces with ratatui widgets. Check out the examples, such as the text input, to understand its usage better.

```rust
fn input(_: ReadSignal<()>) -> Child {
    let (input_r, input_w) = create_signal("".to_string());

    on(move |key: &event::KeyCode| match key {
        event::KeyCode::Char(ch) => input_w.update(|x| x.push(*ch)),
        event::KeyCode::Backspace => input_w.update(|x| {
            x.pop();
        }),
        _ => {}
    });

    view_widget(move || {
        let block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .title("Start typeing")

        Paragraph::new(input_r.get()).block(block)
    })
}
```
