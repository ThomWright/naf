use std::io;
use std::io::Write;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::{cursor, terminal_size};

struct State {
    path: std::path::PathBuf,
    entries: Vec<FileInfo>,
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

        let size = terminal_size().unwrap();

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

        let state = State {
            path: init_path,
            entries,
        };

        // draw
        write!(screen, "{}", termion::cursor::Goto(1, 1))?;
        for s in state.entries {
            if s.is_dir {
                write!(
                    screen,
                    "{}{}",
                    termion::style::Bold,
                    termion::color::Fg(termion::color::LightBlue)
                )?;
            }
            write!(screen, "{}\r\n", s.name)?;
            write!(screen, "{}", termion::style::Reset)?;
        }
        write!(screen, "{}", termion::cursor::Goto(1, size.1))?;
        // write!(screen, "{}", "hello")?;
        write!(screen, "{}", state.path.to_string_lossy().to_string())?;
        screen.flush()?;

        for c in stdin.keys() {
            write!(screen, "{}", termion::clear::All)?;
            match c {
                Ok(Key::Char('q')) => break,
                _ => {}
            }
            // TODO: draw
            screen.flush()?;
        }

        write!(screen, "{}", termion::style::Reset)?;
    }

    Ok(())
}
