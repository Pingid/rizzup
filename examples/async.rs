use anyhow::Result;
use crossterm::event;
use futures::StreamExt;
use ratatui::{style::Stylize, text::*, widgets::*, Terminal};
use rizzup::prelude::*;

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

fn input() -> RatView {
    let input = create_signal("".to_string());
    let blink = create_signal(true);
    let active = create_signal(true);

    let ticker = create_async_task(active.clone(), move |active, send| async move {
        if !active {
            return;
        }
        loop {
            send.send(Message::Tick);
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    });

    let input_c = input.clone();
    create_memo(move || {
        let _ = input_c.get();
        active.set(true);
        blink.set(true);
    });

    let input_c = input.clone();
    on(move |ev: &event::KeyCode| match ev {
        event::KeyCode::Char(ch) => input_c.update(|x| x.push(*ch)),
        event::KeyCode::Backspace => input_c.update(|x| {
            x.pop();
        }),
        event::KeyCode::Up => active.set(false),
        event::KeyCode::Down => active.set(true),
        _ => {}
    });

    on(move |ev: &Message| match ev {
        Message::Tick => blink.update(|v| *v = !*v),
    });

    widget_ref(move || {
        let block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .title("Start typeing")
            .title("(Press esc to exit)")
            .title(format!("{:?}", ticker.get_state()));

        let lines = Line::from(vec![
            Span::from(format!("{}", input.get())),
            Span::from(format!("│")).fg(match blink.get() {
                true => ratatui::style::Color::Black,
                false => ratatui::style::Color::Blue,
            }),
        ]);

        Paragraph::new(lines).block(block)
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut term = init_tui()?;
    init_panic_hook();

    create_async_scope(move |handle| async move {
        let mut reader = crossterm::event::EventStream::new();
        let app = input();
        loop {
            term.draw(|f| f.render_widget_ref(app, f.size()))?;

            tokio::select! {
                _ = handle.listen() => {},
                ev = reader.next() => match ev {
                    Some(Ok(event)) => {
                        if let event::Event::Key(key) = event {
                            if key.kind == event::KeyEventKind::Press {
                                send(key.code)
                            }
                            if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Esc {
                                handle.shutdown().await;
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        restore_tui()?;

        Ok::<(), anyhow::Error>(())
    }).await;

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
