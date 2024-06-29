use anyhow::{Context, Result};
use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::widgets::{Block, Padding};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Borders, Paragraph},
    Terminal,
};
use rizzup::prelude::*;
use std::io::{stderr, Stderr};

fn input() -> Child {
    let state = use_state("".to_string());

    let setter = state.clone();
    on(move |key: event::KeyCode| match key {
        event::KeyCode::Char(ch) => setter.update(|x| x.push(ch)),
        event::KeyCode::Backspace => setter.update(|x| {
            x.pop();
        }),
        _ => {}
    });

    view(move || {
        let block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .title("Start typeing")
            .title("(Press esc to exit)");

        Box::new(Paragraph::new(state.get()).block(block))
    })
}

fn main() -> Result<()> {
    let mut term = setup_terminal()?;
    let events = Events::default();
    let app = App::new(input).with_layer(&events).render();

    loop {
        term.draw(|f| f.render_widget_ref(app, f.size()))?;

        let event = event::read()?;
        if let event::Event::Key(key) = event {
            if key.kind == event::KeyEventKind::Press {
                events.dispatch(key.code)
            }
            if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Esc {
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
