use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::{cursor, execute, terminal};
use std::io;
use std::io::Write;
use tui::backend::CrosstermBackend;
use tui::Terminal;

mod app;
mod ui;

fn main() -> crossterm::Result<()> {
    setup_panic();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    execute!(stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    run(stdout)?;

    cleanup_terminal();

    Ok(())
}

fn run<W: Write>(stdout: W) -> crossterm::Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.hide_cursor()?;

    let mut state = app::State::new()?;
    let mut ui_state = ui::UiState::default();

    let mut ui = ui::Ui::new(terminal);
    ui.draw(&state, &mut ui_state)?;

    loop {
        match crossterm::event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => {
                state.on_up();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => {
                state.on_down();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                state.on_left();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                state.on_right();
            }
            Event::Key(KeyEvent {
                code: KeyCode::PageDown,
                ..
            }) => {
                let h: usize = ui.list_height().into();
                state.on_page_down(h - 1);
            }
            Event::Key(KeyEvent {
                code: KeyCode::PageUp,
                ..
            }) => {
                let h: usize = ui.list_height().into();
                state.on_page_up(h - 1);
            }
            _ => {}
        }
        ui.draw(&state, &mut ui_state)?;
    }

    Ok(())
}

fn setup_panic() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        cleanup_terminal();
        default_hook(info);
    }));
}

fn cleanup_terminal() {
    let mut stdout = io::stdout();

    execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
    execute!(stdout, cursor::Show).unwrap();

    terminal::disable_raw_mode().unwrap();
}
