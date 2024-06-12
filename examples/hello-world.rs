use anyhow::Result;
use crossterm::event;
use ratatui::Terminal;
use rizzup::prelude::*;

fn hello_world() -> RatView {
    widget_ref(|| "Hello World! (press 'q' to quit)")
}

fn main() -> Result<()> {
    let mut term = init_tui()?;
    init_panic_hook();

    with_tracking_scope(|| {
        let app = hello_world();

        loop {
            term.draw(|f| f.render_widget_ref(app, f.size()))?;

            let event = event::read()?;
            if let event::Event::Key(key) = event {
                if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Char('q') {
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
