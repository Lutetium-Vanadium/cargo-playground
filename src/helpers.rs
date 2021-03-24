use crossterm::style::Colorize;
use crossterm::{cursor, terminal};
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
