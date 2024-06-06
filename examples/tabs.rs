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

    let text_c = text.clone();
    on(move |key: &event::KeyCode| match key {
        event::KeyCode::Char(ch) => text_c.update(|x| x.input.push(*ch)),
        event::KeyCode::Backspace => text_c.update(|x| {
            x.input.pop();
        }),
        _ => {}
    });

    widget_ref(move || format!("Tab 1 {}", text.get().input))
}

fn tab2() -> RatView {
    let value = create_signal("".to_string());
    let text = use_context::<Signal<State>>();

    let value_c = value.clone();
    on(move |key: &event::KeyCode| match key {
        event::KeyCode::Char(ch) => value_c.update(|x| x.push(*ch)),
        event::KeyCode::Backspace => value_c.update(|x| {
            x.pop();
        }),
        _ => {}
    });

    widget_ref(move || format!("Tab 1 {} Tab 2 {}", text.get().input, value.get()))
}

fn input() -> RatView {
    provide_context(create_signal(State::default()));
    let state = use_context::<Signal<State>>();

    let state_c = state.clone();
    let tab = create_selector(move || state_c.get().tab);

    let state_c = state.clone();

    on(move |key: &event::KeyCode| match key {
        event::KeyCode::Left => state_c.update(|v| v.tab = 0),
        event::KeyCode::Right => state_c.update(|v| v.tab = 1),
        _ => {}
    });

    widget_ref(move || match tab.get() == 0 {
        true => tab1(),
        false => widget_ref(move || widget_ref(move || tab2())),
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
