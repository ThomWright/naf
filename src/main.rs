use std::io;
use std::io::Read;
use std::io::Write;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui::Terminal;

struct State {
    base_path: std::path::PathBuf,
    files: Vec<FileInfo>,
    selected_file: usize,
}

struct FileInfo {
    name: String,
    is_dir: bool,
}

fn main() -> Result<(), std::io::Error> {
    // Open issues:
    // - https://gitlab.redox-os.org/redox-os/termion/-/issues/176
    // - https://github.com/fdehau/tui-rs/issues/177
    // Turning this on lets us see some panic output, but prevents restoring from raw mode when exiting normally.
    if let Ok(_) = std::env::var("HANDLE_PANIC") {
        setup_panic();
    }

    // Get and lock the stdios so we don't have to get the lock all the time
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stdout = io::stdout();
    let stdout = stdout.lock();

    run(stdin, stdout)?;

    Ok(())
}

fn run<R: Read, W: Write>(stdin: R, stdout: W) -> Result<(), std::io::Error> {
    let screen = AlternateScreen::from(stdout.into_raw_mode()?);
    let mut terminal = Terminal::new(TermionBackend::new(screen))?;
    terminal.hide_cursor()?;

    let mut state = init_state()?;

    draw(&mut terminal, &state)?;

    for c in stdin.keys() {
        match c {
            Ok(Key::Char('q')) => break,
            Ok(Key::Up) => state.selected_file = state.selected_file.saturating_sub(1),
            Ok(Key::Down) => {
                state.selected_file = (state.selected_file + 1).min(state.files.len() - 1)
            }
            _ => {}
        }
        draw(&mut terminal, &state)?;
    }

    Ok(())
}

fn init_state() -> Result<State, std::io::Error> {
    let init_path = std::env::current_dir()?;
    let mut entries = init_path.as_path().read_dir().map_or(vec![], |contents| {
        contents
            .filter_map(Result::ok)
            .map(|entry| {
                let mut name = entry.file_name().to_string_lossy().to_string();
                if entry.path().is_dir() {
                    name.push('/');
                }
                FileInfo {
                    name,
                    is_dir: entry.path().is_dir(),
                }
            })
            .collect()
    });
    entries.sort_by(|f1, f2| f1.name.cmp(&f2.name));

    Ok(State {
        base_path: init_path,
        files: entries,
        selected_file: 0,
    })
}

fn draw<B: Backend>(terminal: &mut Terminal<B>, state: &State) -> io::Result<()> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Min(5),
                    // Should be 1, but: https://github.com/fdehau/tui-rs/issues/366
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(f.size());

        {
            let files_block = Block::default().title("Files").borders(Borders::ALL);
            let mut list_state = ListState::default();
            list_state.select(Some(state.selected_file));
            let list = List::new(
                state
                    .files
                    .iter()
                    .map(|f| {
                        ListItem::new(vec![Spans::from(vec![if f.is_dir {
                            Span::styled(
                                format!("{:<9}", &f.name),
                                Style::default().fg(Color::LightBlue),
                            )
                        } else {
                            Span::raw(&f.name)
                        }])])
                    })
                    .collect::<Vec<ListItem>>(),
            )
            .block(files_block)
            .highlight_style(Style::default().bg(Color::Rgb(32, 32, 32)));

            f.render_stateful_widget(list, chunks[0], &mut list_state);
        }
        {
            let path_block = Block::default().borders(Borders::NONE);
            let para = Paragraph::new(state.base_path.to_string_lossy().to_string())
                .block(path_block)
                .alignment(Alignment::Left);
            f.render_widget(para, chunks[1]);
        }
    })
}

fn setup_panic() {
    let raw_handle = io::stdout().into_raw_mode().unwrap();
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        raw_handle
            .suspend_raw_mode()
            .unwrap_or_else(|e| eprintln!("Could not suspend raw mode: {}", e));
        default_hook(info);
    }));
}
