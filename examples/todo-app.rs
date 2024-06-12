use anyhow::Result;
use crossterm::event;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::*,
    Terminal,
};
use rizzup::{prelude::*, ratatui::widget_ref};

#[derive(Default, Clone)]
struct Todo {
    text: String,
    complete: bool,
}

impl Todo {
    pub fn new(text: String) -> Self {
        Self {
            text,
            complete: false,
        }
    }
}

fn todo_text_input(focused: ReadSignal<bool>) -> RatView {
    let value = create_signal("".to_string());
    let todos = use_context::<Signal<Vec<Todo>>>();

    let value_c = value.clone();
    on(move |key: &event::KeyCode| {
        if focused.get() == false {
            return;
        }
        match key {
            event::KeyCode::Char(ch) => value_c.update(|x| x.push(*ch)),
            event::KeyCode::Backspace => value_c.update(|x| {
                x.pop();
            }),
            event::KeyCode::Enter => {
                todos.update(|t| t.push(Todo::new(value_c.get())));
                value_c.set("".into());
            }
            _ => {}
        }
    });

    widget_ref(move || {
        let block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .border_style(match focused.get() {
                true => Color::Cyan,
                false => Color::default(),
            })
            .title("Add a todo");
        Paragraph::new(value.get()).block(block)
    })
}

fn todo_list(focused: ReadSignal<bool>) -> RatView {
    let todos = use_context::<Signal<Vec<Todo>>>();
    let state = create_signal({
        let mut s = ListState::default();
        s.select(Some(0));
        s
    });

    let todos_c = todos.clone();
    let state_c = state.clone();
    on(move |key: &event::KeyCode| {
        if focused.get() == false {
            return;
        }
        let size = todos_c.get().len();
        match key {
            event::KeyCode::Enter => todos_c.update(|t| {
                if let Some(s) = state_c.get().selected() {
                    if let Some(todo) = t.get_mut(s) {
                        todo.complete = !todo.complete
                    }
                }
            }),
            event::KeyCode::Backspace => todos_c.update(|t| {
                if let Some(selected) = state_c.get().selected() {
                    if selected < t.len() && t.len() > 0 {
                        t.remove(selected);
                        state_c.update(|s| s.select(Some(selected.saturating_sub(1))));
                    }
                }
            }),
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

    statefull_widget_ref(state, move || {
        let block = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::all())
            .border_style(match focused.get() {
                true => Color::Cyan,
                false => Color::default(),
            })
            .title("Todo");

        let list = todos.get().into_iter().map(|x| {
            ListItem::new(x.text.clone()).add_modifier(match x.complete {
                true => Modifier::CROSSED_OUT,
                false => Modifier::empty(),
            })
        });

        let list = List::new(list)
            .block(block)
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
        list
    })
}

#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Input,
    List,
}

fn todo_list_app() -> RatView {
    let focus = create_signal(Focus::Input);
    provide_context(create_signal::<Vec<Todo>>(vec![]));

    let todo_list = todo_list(create_memo(move || focus.get() == Focus::List));
    let todo_list_input = todo_text_input(create_memo(move || focus.get() == Focus::Input));

    on(move |ev: &event::KeyCode| match ev {
        event::KeyCode::Char(_) => focus.update(|v| *v = Focus::Input),
        event::KeyCode::Down => focus.update(|v| match v {
            Focus::Input => *v = Focus::List,
            Focus::List => {}
        }),
        event::KeyCode::Tab => focus.update(|v| match v {
            Focus::Input => *v = Focus::List,
            Focus::List => *v = Focus::Input,
        }),
        _ => {}
    });

    render(move |area, buf| {
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

    with_tracking_scope(|| {
        let app = todo_list_app();

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
