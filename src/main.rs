use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::{cursor, execute, terminal};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};

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
            }) => {
                state.on_up();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => {
                state.on_down();
            }
            _ => {}
        }
        draw(&mut terminal, &state)?;
    }

    Ok(())
}

struct State {
    base_path: std::path::PathBuf,
    dirs: [Vec<FileInfo>; 2],
    selected_dir: usize,
    selected_file: usize,
}

struct FileInfo {
    name: String,
    is_dir: bool,
    // hmmm
    path: PathBuf,
}

impl State {
    fn new() -> Result<State, std::io::Error> {
        let init_path = std::env::current_dir()?;
        let current_dir_files = State::read_file_list(&init_path);

        let selected_dir = 0;
        let selected_file = 0;

        let dirs = [current_dir_files, vec![]];

        let mut state = State {
            base_path: init_path,
            dirs,
            selected_dir,
            selected_file,
        };

        state.dirs[1] = state.files_in_selected_dir();

        Ok(state)
    }

    fn files_in_selected_dir(&self) -> Vec<FileInfo> {
        self.dirs[self.selected_dir]
            .get(self.selected_file)
            .filter(|f| f.is_dir)
            .map(|f| State::read_file_list(&f.path))
            .unwrap_or_default()
    }

    fn read_file_list<P: AsRef<Path>>(path: &P) -> Vec<FileInfo> {
        let mut files = path.as_ref().read_dir().map_or(vec![], |contents| {
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
                        path: entry.path(),
                    }
                })
                .collect()
        });
        files.sort_by(|f1, f2| f1.name.cmp(&f2.name));
        files
    }

    fn on_up(&mut self) {
        self.selected_file = self.selected_file.saturating_sub(1);
        self.dirs[1] = self.files_in_selected_dir();
    }

    fn on_down(&mut self) {
        let list_len = self
            .dirs
            .get(self.selected_dir)
            .map(|l| l.len())
            .unwrap_or(0);
        self.selected_file = (self.selected_file + 1).min(list_len.saturating_sub(1));

        self.dirs[1] = self.files_in_selected_dir();
    }
}

fn draw<B: Backend>(terminal: &mut Terminal<B>, state: &State) -> io::Result<()> {
    terminal.draw(|f| {
        let frame_chunks = Layout::default()
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

        let files_block = Block::default().title("Files").borders(Borders::ALL);
        let files_block_inner_area = files_block.inner(frame_chunks[0]);
        f.render_widget(files_block, frame_chunks[0]);
        {
            let list_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(files_block_inner_area);

            draw_file_list(f, list_chunks[0], &state, 0);
            draw_file_list(f, list_chunks[1], &state, 1);
        }

        draw_status(f, frame_chunks[1], &state);
    })
}

fn draw_file_list<B: Backend>(f: &mut Frame<B>, area: Rect, state: &State, thing: usize) {
    let list_block = Block::default().borders(Borders::NONE);

    let mut list_state = ListState::default();
    list_state.select(if state.selected_dir == thing {
        Some(state.selected_file)
    } else {
        None
    });
    let list = List::new(
        state.dirs[thing]
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
    .block(list_block)
    .highlight_style(Style::default().bg(Color::Rgb(32, 32, 32)));

    f.render_stateful_widget(list, area, &mut list_state);
}

fn draw_status<B: Backend>(f: &mut Frame<B>, area: Rect, state: &State) {
    let status_block = Block::default().borders(Borders::NONE);
    let para = Paragraph::new(state.base_path.to_string_lossy().to_string())
        .block(status_block)
        .alignment(Alignment::Left);
    f.render_widget(para, area);
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
