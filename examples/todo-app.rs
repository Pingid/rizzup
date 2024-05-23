use anyhow::Result;
use crossterm::event;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::*,
    Terminal,
};
use rizzup::{
    prelude::*,
    ratatui::view_statefull_widget,
    scope::{provide_layer, use_layer},
    signal::RwSignal,
};

#[derive(Clone)]
struct Todo {
    text: String,
}

fn todo_text_input() -> Child {
    let (value_r, value_w) = create_signal("".to_string());
    let todos = use_layer::<RwSignal<Vec<Todo>>>();
    let value_r_c = value_r.clone();

    on(move |key: &event::KeyCode| match key {
        event::KeyCode::Char(ch) => value_w.update(|x| x.push(*ch)),
        event::KeyCode::Backspace => value_w.update(|x| {
            x.pop();
        }),
        event::KeyCode::Enter => {
            let todo = Todo {
                text: value_r_c.get(),
            };
            todos.update(|t| t.push(todo));
            value_w.set("".into());
        }
        _ => {}
    });

    view_widget(move || {
        let block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .title("Add a todo");
        Paragraph::new(value_r.get()).block(block)
    })
}

fn todo_list() -> Child {
    let todos = use_layer::<RwSignal<Vec<Todo>>>();
    let state = create_rw_signal({
        let mut s = ListState::default();
        s.select(Some(0));
        s
    });

    let todos_c = todos.clone();
    let state_c = state.clone();
    on(move |key: &event::KeyCode| {
        let size = todos_c.get().len();
        match key {
            event::KeyCode::Up => state_c.update(|s| {
                s.select(match s.selected() {
                    Some(0) => Some(size.max(1) - 1),
                    Some(n) => Some(n - 1),
                    n => n,
                })
            }),
            event::KeyCode::Down => state_c.update(|s| {
                let selected = s.selected().unwrap_or(0);
                match selected >= size.max(1) - 1 {
                    true => s.select(Some(0)),
                    false => s.select(Some(selected + 1)),
                };
            }),
            _ => {}
        }
    });

    view_statefull_widget(state, move || {
        let block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .title("Todo");

        let list = List::new(todos.get().iter().map(|x| x.text.clone()))
            .block(block)
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
        list
    })
}

fn todo_list_app() -> Child {
    provide_layer(create_rw_signal::<Vec<Todo>>(vec![]));

    let todo_list = todo_list();
    let todo_list_input = todo_text_input();

    view_fn(move |area, buf| {
        let areas: [Rect; 2] = Layout::new(
            ratatui::layout::Direction::Vertical,
            [Constraint::Max(3), Constraint::Fill(1)],
        )
        .areas(area);

        todo_list_input.render_ref(areas[0], buf);
        todo_list.render_ref(areas[1], buf);
    })
}

fn main() -> Result<()> {
    let mut term = init_tui()?;
    init_panic_hook();

    let events = Dispatcher::default();
    create_scope(|| {
        provide_layer(events.clone());
        let app = todo_list_app();

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
