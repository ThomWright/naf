use std::io;
use std::io::Write;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{cursor, terminal_size};

struct State {
    term_size: (u16, u16),
    base_path: std::path::PathBuf,
    files: Vec<FileInfo>,
    selected_file: usize,
}

struct FileInfo {
    name: String,
    is_dir: bool,
}

fn main() -> Result<(), std::io::Error> {
    // Get and lock the stdios so we don't have to get the lock all the time
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let stdout = stdout.into_raw_mode().unwrap();
    {
        let screen = AlternateScreen::from(stdout);
        let mut screen = cursor::HideCursor::from(screen);

        let term_size = terminal_size().unwrap();

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

        let mut state = State {
            term_size,
            base_path: init_path,
            files: entries,
            selected_file: 0,
        };

        draw(&mut screen, &state)?;
        screen.flush()?;

        for c in stdin.keys() {
            write!(screen, "{}", termion::clear::All)?;
            match c {
                Ok(Key::Char('q')) => break,
                Ok(Key::Up) => state.selected_file = state.selected_file.saturating_sub(1),
                Ok(Key::Down) => {
                    state.selected_file = (state.selected_file + 1).min(state.files.len() - 1)
                }
                _ => {}
            }
            draw(&mut screen, &state)?;
            screen.flush()?;
        }

        write!(screen, "{}", termion::style::Reset)?;
    }

    Ok(())
}

fn draw<W: Write>(w: &mut W, state: &State) -> std::result::Result<(), std::io::Error> {
    write!(w, "{}", termion::cursor::Goto(1, 1))?;
    for (i, s) in (&state.files).iter().enumerate() {
        if s.is_dir {
            write!(
                w,
                "{}{}",
                termion::style::Bold,
                termion::color::Fg(termion::color::LightBlue)
            )?;
        }
        if i == state.selected_file {
            write!(
                w,
                "{}",
                termion::color::Bg(termion::color::AnsiValue::grayscale(5))
            )?;
        }
        write!(w, "{}\r\n", s.name)?;
        write!(w, "{}", termion::style::Reset)?;
    }
    write!(w, "{}", termion::cursor::Goto(1, state.term_size.1))?;
    write!(w, "{}", state.base_path.to_string_lossy().to_string())?;

    Ok(())
}
