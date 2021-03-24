use crossterm::style::{style, Attribute, Color, Print, PrintStyledContent, SetForegroundColor};
use crossterm::style::{Colorize, Styler};
use crossterm::{cursor, event, execute, queue, terminal};
use std::convert::TryFrom;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{atomic, Arc};
use std::time::Duration;
use std::{env, io, thread};

/// Gets the path to directory in which playgrounds will be created.
pub fn get_dir() -> PathBuf {
    env::var_os("CARGO_PLAYGROUND_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("cargo-playground"))
}

/// Starts a loader on a new thread.
pub fn loader(prompt: &'static str, stop: Arc<atomic::AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        const STATES: [char; 6] = ['⠷', '⠯', '⠟', '⠻', '⠽', '⠾'];
        let mut state = 0;
        let mut stdout = io::stdout();
        // it is ok if the old value is gotten as it will get new value next time
        while !stop.load(atomic::Ordering::Relaxed) {
            write!(stdout, "\r{} {}", STATES[state].cyan(), prompt).unwrap();
            stdout.flush().unwrap();
            state = (state + 1) % STATES.len();
            thread::sleep(Duration::from_millis(200));
        }

        crossterm::execute!(
            stdout,
            terminal::Clear(terminal::ClearType::CurrentLine),
            cursor::MoveToColumn(0),
        )
        .unwrap();
    })
}

const STATUS_NAME_WIDTH: usize = 12;

pub fn print_status(status_name: &str, status: &str) {
    println!(
        "{:>status_name_width$} {}",
        status_name.dark_green().bold(),
        status,
        status_name_width = STATUS_NAME_WIDTH
    );
}

/// Simple helper to make sure if the code panics in between, raw mode is disabled
struct RawMode;

impl RawMode {
    fn enable() -> crossterm::Result<Self> {
        queue!(io::stdout(), cursor::Hide)?;
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = queue!(io::stdout(), cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

pub fn pick_from<T: AsRef<str>>(prompt: &str, from: &[T]) -> crossterm::Result<Option<usize>> {
    let _raw = RawMode::enable()?;

    let th = terminal::size()?.1;

    // FIXME: handle lists longer than screen
    let from_len = u16::try_from(from.len()).unwrap();
    assert!(from_len <= th);

    let ch = cursor::position()?.1;

    let mut stdout = io::stdout();

    let mut base_row = ch + 1;

    if ch > th - from_len - 1 {
        let dist = ch + from_len + 1 - th;
        base_row -= dist;
        queue!(stdout, terminal::ScrollUp(dist), cursor::MoveUp(dist))?;
    }

    queue!(
        stdout,
        Print("  "),
        PrintStyledContent(style(prompt).attribute(Attribute::Bold)),
        PrintStyledContent(style(" (use arrow keys or j/k)").with(Color::DarkGrey)),
        cursor::MoveToNextLine(1),
    )?;

    let mut currently_at = 0;

    for (i, from) in from.iter().enumerate() {
        println(from.as_ref(), i == currently_at, &mut stdout)?;
    }

    execute!(stdout, cursor::MoveTo(1, base_row))?;

    let res = loop {
        if let event::Event::Key(e) = event::read()? {
            match e.code {
                event::KeyCode::Char('c') if e.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    break None
                }
                event::KeyCode::Esc | event::KeyCode::Null => break None,
                event::KeyCode::Up | event::KeyCode::Char('k') if currently_at != 0 => {
                    print(from[currently_at].as_ref(), false, &mut stdout)?;
                    currently_at -= 1;
                    queue!(stdout, cursor::MoveToPreviousLine(1))?;
                }
                event::KeyCode::Down | event::KeyCode::Char('j')
                    if currently_at < from.len() - 1 =>
                {
                    println(from[currently_at].as_ref(), false, &mut stdout)?;
                    currently_at += 1;
                }
                event::KeyCode::Home | event::KeyCode::PageUp | event::KeyCode::Char('g')
                    if currently_at != 0 =>
                {
                    print(from[currently_at].as_ref(), false, &mut stdout)?;
                    queue!(stdout, cursor::MoveTo(0, base_row))?;
                    currently_at = 0;
                }
                event::KeyCode::End | event::KeyCode::PageDown | event::KeyCode::Char('G')
                    if currently_at != from.len() - 1 =>
                {
                    print(from[currently_at].as_ref(), false, &mut stdout)?;
                    currently_at = from.len() - 1;
                    queue!(stdout, cursor::MoveTo(0, base_row + from_len - 1))?;
                }
                event::KeyCode::Enter => {
                    break Some(currently_at);
                }
                _ => continue,
            }

            print(from[currently_at].as_ref(), true, &mut stdout)?;
            stdout.flush()?;
        }
    };

    execute!(
        stdout,
        cursor::MoveTo(0, base_row - 1),
        terminal::Clear(terminal::ClearType::FromCursorDown)
    )?;

    Ok(res)
}

fn print(name: &str, is_selected: bool, stdout: &mut io::Stdout) -> crossterm::Result<()> {
    queue!(
        stdout,
        terminal::Clear(terminal::ClearType::CurrentLine),
        cursor::MoveToColumn(0)
    )?;

    if is_selected {
        queue!(stdout, SetForegroundColor(Color::Cyan), Print("❯ "))?;
    } else {
        queue!(stdout, Print("  "))?;
    }

    queue!(stdout, Print(name))?;

    if is_selected {
        queue!(stdout, SetForegroundColor(Color::Reset))?;
    }

    Ok(())
}

fn println(name: &str, is_selected: bool, stdout: &mut io::Stdout) -> crossterm::Result<()> {
    print(name, is_selected, stdout)?;
    queue!(stdout, cursor::MoveToNextLine(1))
}
