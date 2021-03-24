use super::Examples;
use crossterm::style::{style, Attribute, Color, Print, PrintStyledContent, SetForegroundColor};
use crossterm::{cursor, event, execute, queue, terminal};
use std::io;
use std::path::PathBuf;
use std::{convert::TryFrom, io::Write};

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

pub fn pick_from(examples: Examples) -> crossterm::Result<Option<PathBuf>> {
    let Examples { examples, mut path } = examples;
    let _raw = RawMode::enable()?;

    let th = terminal::size()?.1;

    // FIXME: handle lists longer than screen
    let ex_len = u16::try_from(examples.len()).unwrap();
    assert!(ex_len <= th);

    let ch = cursor::position()?.1;

    let mut stdout = io::stdout();

    let mut base_row = ch + 1;

    if ch > th - ex_len - 1 {
        let dist = ch + ex_len + 1 - th;
        base_row -= dist;
        queue!(stdout, terminal::ScrollUp(dist), cursor::MoveUp(dist))?;
    }

    queue!(
        stdout,
        PrintStyledContent(style("  Pick an example: ").attribute(Attribute::Bold)),
        PrintStyledContent(style("(use arrow keys)").with(Color::DarkGrey)),
        cursor::MoveToNextLine(1),
    )?;

    let mut currently_at = 0;

    for (i, example) in examples.iter().enumerate() {
        println(example, i == currently_at, &mut stdout)?;
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
                    print(&examples[currently_at], false, &mut stdout)?;
                    currently_at -= 1;
                    queue!(stdout, cursor::MoveToPreviousLine(1))?;
                }
                event::KeyCode::Down | event::KeyCode::Char('j')
                    if currently_at < examples.len() - 1 =>
                {
                    println(&examples[currently_at], false, &mut stdout)?;
                    currently_at += 1;
                }
                event::KeyCode::Home | event::KeyCode::PageUp | event::KeyCode::Char('g')
                    if currently_at != 0 =>
                {
                    print(&examples[currently_at], false, &mut stdout)?;
                    queue!(stdout, cursor::MoveTo(0, base_row))?;
                    currently_at = 0;
                }
                event::KeyCode::End | event::KeyCode::PageDown | event::KeyCode::Char('G')
                    if currently_at != examples.len() - 1 =>
                {
                    print(&examples[currently_at], false, &mut stdout)?;
                    currently_at = examples.len() - 1;
                    queue!(stdout, cursor::MoveTo(0, base_row + ex_len - 1))?;
                }
                event::KeyCode::Enter => {
                    path.push(&examples[currently_at]);
                    break Some(path);
                }
                _ => continue,
            }

            print(&examples[currently_at], true, &mut stdout)?;
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
