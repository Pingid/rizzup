use anyhow::{Context, Result};
use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use rizzup::prelude::*;
use std::io::{stderr, Stderr};

fn hello_world() -> Child {
    view(|| Box::new("Hello World! (press 'q' to quit)"))
}

fn main() -> Result<()> {
    let mut term = setup_terminal()?;
    let app = App::new(hello_world).render();

    loop {
        term.draw(|f| f.render_widget_ref(app, f.size()))?;

        let event = event::read()?;
        if let event::Event::Key(key) = event {
            if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Char('q') {
                break;
            }
        }
    }

    restore_terminal(&mut term)?;

    Ok(())
}

/// Setup the terminal. This is where you would enable raw mode, enter the alternate screen, and
/// hide the cursor. This example does not handle errors. A more robust application would probably
/// want to handle errors and ensure that the terminal is restored to a sane state before exiting.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stderr>>> {
    let mut stdout = stderr();
    enable_raw_mode().context("failed to enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")
}

/// Restore the terminal. This is where you disable raw mode, leave the alternate screen, and show
/// the cursor.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stderr>>) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("unable to switch to main screen")?;
    terminal.show_cursor().context("unable to show cursor")
}
