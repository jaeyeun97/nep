extern crate termion;

use std::boxed::Box;
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use std::vec::Vec;
use terminal::termion::event::Key;
use terminal::termion::input::Keys;
use terminal::termion::raw::{IntoRawMode, RawTerminal};
use terminal::termion::screen::AlternateScreen;
use terminal::termion::{clear, color, cursor, style};

pub use terminal::termion::input::TermRead;

enum Control {
    SetCursor(i32, i32),
    PutChar(char),
    CursorLeft,
    CursorRight,
    CursorUp,
    CursorDown,
    Exit,
}

pub struct Terminal {
    output: AlternateScreen<RawTerminal<Stdout>>,
    buffer: Vec<Key>,
}

impl Terminal {
    pub fn new() -> Terminal {
        let mut terminal = Terminal {
            output: AlternateScreen::from(stdout().into_raw_mode().unwrap()),
            buffer: Vec::new(),
        };

        write!(terminal.output, "{}{}", clear::All, cursor::Goto(1, 1));
        terminal.output.flush().unwrap();
        terminal
    }

    pub fn start(&mut self) {
        let input = stdin();
        for key in input.keys() {
            self.buffer.push(key.unwrap());
            self.output.flush().unwrap();
        }
    }

    fn apply_control(&mut self, control: Control) {}
}
