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
            _ => {}
        }
        draw(&mut terminal, &state)?;
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct FileList {
    files: Vec<FileInfo>,
    selected_file: Option<usize>,
}
impl FileList {
    fn get_selected_file(&self) -> Option<&FileInfo> {
        match self.selected_file {
            None => None,
            Some(i) => self.files.get(i),
        }
    }

    fn unselect_file(&mut self) {
        self.selected_file = None
    }

    fn select_first(&mut self) {
        if self.files.len() > 0 {
            self.selected_file = Some(0);
        }
    }

    /// Select next file in list, returning whether the file selection has changed.
    fn select_next(&mut self) -> bool {
        let prev = self.selected_file;
        if let Some(selected) = self.selected_file {
            self.selected_file = Some((selected + 1).min(self.files.len().saturating_sub(1)));
        }
        if prev != self.selected_file {
            true
        } else {
            false
        }
    }

    /// Select previous file in list, returning whether the file selection has changed.
    fn select_prev(&mut self) -> bool {
        let prev = self.selected_file;
        if let Some(selected) = self.selected_file {
            self.selected_file = Some(selected.saturating_sub(1));
        }
        if prev != self.selected_file {
            true
        } else {
            false
        }
    }
}
impl Default for FileList {
    fn default() -> Self {
        FileList {
            files: vec![],
            selected_file: None,
        }
    }
}

#[derive(Debug, Clone)]
struct FileInfo {
    name: String,
    is_dir: bool,
    // hmmm
    path: PathBuf,
}

// TODO: define `current` and `selected`
#[derive(Debug, Clone)]
struct State {
    base_path: std::path::PathBuf,
    flists: [FileList; 2],
    selected_flist: usize,
}

// FIXME: don't allow state of selecting a list with no files in it
impl State {
    fn new() -> Result<State, std::io::Error> {
        let init_path = std::env::current_dir()?;
        let current_dir_files = State::read_file_list(&init_path);
        let selected_file = current_dir_files.first().map(|_| 0);

        let flists = [
            FileList {
                files: current_dir_files,
                selected_file,
            },
            FileList::default(),
        ];

        let mut state = State {
            base_path: init_path,
            flists,
            selected_flist: 0,
        };

        state.flists[1] = FileList {
            files: state.files_in_selected_dir(),
            selected_file: None,
        };

        Ok(state)
    }

    fn files_in_selected_dir(&self) -> Vec<FileInfo> {
        match self.current_list().get_selected_file() {
            None => Vec::default(),
            Some(file) => {
                if file.is_dir {
                    State::read_file_list(&file.path)
                } else {
                    Vec::default()
                }
            }
        }
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

    fn space_to_right(&self) -> bool {
        self.selected_flist < (self.flists.len() - 1)
    }

    fn refresh_files_to_right(&mut self) {
        if self.space_to_right() {
            self.flists[self.selected_flist + 1] = FileList {
                files: self.files_in_selected_dir(),
                selected_file: None,
            };
        }
    }

    fn selected_file_is_dir(&self) -> bool {
        self.current_list()
            .get_selected_file()
            .map_or(false, |f| f.is_dir)
    }

    fn selected_file_path(&self) -> Option<&PathBuf> {
        self.current_list().get_selected_file().map(|f| &f.path)
    }

    fn current_list(&self) -> &FileList {
        &self.flists[self.selected_flist]
    }

    fn current_list_mut(&mut self) -> &mut FileList {
        &mut self.flists[self.selected_flist]
    }

    fn selected_file_in_list(&self, list: usize) -> Option<usize> {
        self.flists.get(list).and_then(|l| l.selected_file)
    }

    fn on_up(&mut self) {
        if self.current_list_mut().select_prev() {
            self.refresh_files_to_right();
        }
    }

    fn on_down(&mut self) {
        if self.current_list_mut().select_next() {
            self.refresh_files_to_right();
        }
    }

    fn on_left(&mut self) {
        if self.selected_flist != 0 {
            self.current_list_mut().unselect_file();
            self.selected_flist -= 1;
        } else if let Some(parent) = self.base_path.parent() {
            let parent = parent.to_owned();

            self.current_list_mut().unselect_file();

            let files = State::read_file_list(&parent);
            let selected_file = files.binary_search_by_key(&self.base_path, |f| f.path.clone());
            self.flists = [
                FileList {
                    files,
                    // FIXME: I bet we can somehow go into a parent directory with no visible files...
                    selected_file: selected_file.ok(),
                },
                self.flists[0].clone(),
            ];

            self.base_path = parent
        }
    }

    fn on_right(&mut self) {
        if self.selected_file_is_dir() {
            if self.space_to_right() {
                self.selected_flist += 1;
                self.current_list_mut().select_first();
            } else if let Some(selected_dir_path) = self.selected_file_path() {
                let selected_dir_path = selected_dir_path.clone();

                let files = State::read_file_list(&selected_dir_path);
                let selected_file = files.first().map(|_| 0);
                self.flists = [
                    self.flists[1].clone(),
                    FileList {
                        files,
                        selected_file,
                    },
                ];

                // FIXME: no unwrap!
                self.base_path = selected_dir_path.parent().unwrap().to_owned();
            }
        }
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
    list_state.select(state.selected_file_in_list(thing));
    let list = List::new(
        state.flists[thing]
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
