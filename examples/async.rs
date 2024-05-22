use anyhow::Result;
use crossterm::event;
use futures::StreamExt;
use ratatui::{style::Stylize, text::*, widgets::*, Terminal};
use rizzup::prelude::*;

#[derive(Debug, Clone)]
enum Message {
    Blink(bool),
}

fn input(_: ReadSignal<()>) -> Child {
    let (input_r, input_w) = create_signal("".to_string());
    let (blink_r, blink_w) = create_signal(false);

    async_with_dispatch(move |send| async move {
        let mut b = false;
        loop {
            b = !b;
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            send.dispatch(Message::Blink(b));
        }
    });

    match_on!(event::KeyCode, {
        event::KeyCode::Char(ch) => input_w.update(|x| x.push(*ch)),
        event::KeyCode::Backspace => input_w.update(|x| { x.pop(); }),
    });

    match_on!(Message, {
        Message::Blink(n) => blink_w.set(*n),
    });

    widget! {
        let block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .title("Start typeing")
            .title("(Press esc to exit)");

        let lines = Line::from(vec![Span::from(format!("{}", input_r.get())),
            Span::from(format!("|")).fg(match blink_r.get() {
                true =>  ratatui::style::Color::Black,
                false =>  ratatui::style::Color::White,
            })
        ]);

        Paragraph::new(lines).block(block)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut term = init_tui()?;
    init_panic_hook();

    let events = Events::default();
    let mut sync = AsyncTasks::new();
    let app = App::new(input, ())
        .with_layer(&events)
        .with_layer(&sync.layer)
        .render();

    let mut reader = crossterm::event::EventStream::new();
    let mut reciever = sync.reciever().await;

    loop {
        term.draw(|f| f.render_widget_ref(app, f.size()))?;

        tokio::select! {
            ev = reciever.recv() => match ev {
                Some(m) => events.dispatch_boxed(m),
                _ => {}
            },
            ev = reader.next() => match ev {
                Some(Ok(event)) => {
                    if let event::Event::Key(key) = event {
                        if key.kind == event::KeyEventKind::Press {
                            events.dispatch(key.code)
                        }
                        if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Esc {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let _ = sync.shutdown().await;

    restore_tui()?;

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
