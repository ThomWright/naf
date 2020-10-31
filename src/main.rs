use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::{cursor, execute, terminal};
use std::io;
use std::io::Write;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui::Terminal;

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

    let mut state = State::new()?;

    draw(&mut terminal, &state)?;

    loop {
        match crossterm::event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => state.selected_file = state.selected_file.saturating_sub(1),
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => state.selected_file = (state.selected_file + 1).min(state.files.len() - 1),
            _ => {}
        }
        draw(&mut terminal, &state)?;
    }

    Ok(())
}

struct State {
    base_path: std::path::PathBuf,
    files: Vec<FileInfo>,
    selected_file: usize,
}

struct FileInfo {
    name: String,
    is_dir: bool,
}

impl State {
    fn new() -> Result<State, std::io::Error> {
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
                                Style::default()
                                    .fg(Color::LightBlue)
                                    .add_modifier(Modifier::BOLD),
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
