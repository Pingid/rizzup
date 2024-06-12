use anyhow::Result;
use crossterm::event;
use ratatui::{widgets::*, Terminal};
use rizzup::prelude::*;

fn input() -> RatView {
    let value = create_signal("".to_string());

    let value_c = value.clone();
    on(move |key: &event::KeyCode| match key {
        event::KeyCode::Char(ch) => value_c.update(|x| x.push(*ch)),
        event::KeyCode::Backspace => value_c.update(|x| {
            x.pop();
        }),
        _ => {}
    });

    widget_ref(move || {
        let block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .title("Start typeing")
            .title("(Press esc to exit)");

        Paragraph::new(value.get()).block(block)
    })
}

fn main() -> Result<()> {
    let mut term = init_tui()?;
    init_panic_hook();

    with_tracking_scope(|| {
        let app = input();

        loop {
            term.draw(|f| f.render_widget_ref(app, f.size()))?;

            let event = event::read()?;
            if let event::Event::Key(key) = event {
                if key.kind == event::KeyEventKind::Press {
                    send(key.code)
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
