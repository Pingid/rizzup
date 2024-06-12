use anyhow::Result;
use crossterm::event;
use ratatui::Terminal;
use rizzup::prelude::*;

#[derive(Debug, Clone, Default)]
struct State {
    input: String,
    tab: usize,
}

fn tab1() -> RatView {
    let text = use_context::<Signal<State>>();

    reactive!(receiver: clone(text): {
        event::KeyCode::Char(ch) => text.update(|x| x.input.push(*ch)),
        event::KeyCode::Backspace => text.update(|x| {
            x.input.pop();
        }),
    });

    widget_ref(move || format!("Tab 1 {}", text.get().input))
}

fn tab2() -> RatView {
    let value = create_signal("".to_string());
    let text = use_context::<Signal<State>>();

    reactive!(receiver: clone(value): {
        event::KeyCode::Char(ch) => value.update(|x| x.push(*ch)),
        event::KeyCode::Backspace => value.update(|x| { x.pop(); }),
    });

    widget_ref(move || format!("Tab 1 {} Tab 2 {}", text.get().input, value.get()))
}

fn input() -> RatView {
    provide_context(create_signal(State::default()));
    let state = use_context::<Signal<State>>();

    let tab = reactive!(selector: clone(state): state.get().tab);

    reactive!(receiver: clone(state): {
        event::KeyCode::Left => state.update(|v| v.tab = 0),
        event::KeyCode::Right => state.update(|v| v.tab = 1),
    });

    widget_ref(move || match tab.get() == 0 {
        true => tab1(),
        false => tab2(),
    })
}

fn main() -> Result<()> {
    let mut term = init_tui()?;
    init_panic_hook();

    create_tracking_scope(|| {
        let app = create_memo(|| input());

        loop {
            term.draw(|f| f.render_widget_ref(app.get(), f.size()))?;

            let event = event::read()?;
            if let event::Event::Key(key) = event {
                if key.kind == event::KeyEventKind::Press {
                    send(key.code);
                }
                if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Esc {
                    break;
                }
            }
        }

        restore_tui()?;

        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}

pub fn init_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_tui();
        original_hook(panic_info);
    }));
}

pub fn init_tui() -> std::io::Result<Terminal<impl ratatui::backend::Backend>> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;
    Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stderr()))
}

pub fn restore_tui() -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}
