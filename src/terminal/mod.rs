extern crate termion;

use std::boxed::Box;
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use std::vec::Vec;
use terminal::termion::event::Key;
use terminal::termion::input::TermRead;
use terminal::termion::raw::{IntoRawMode, RawTerminal};
use terminal::termion::screen::AlternateScreen;
use terminal::termion::{clear, color, cursor, style};

pub mod mode;

use self::mode::{Binding, BindingMember, Mode};

pub struct Terminal {
    mode: mode::Mode,
    output: AlternateScreen<RawTerminal<Stdout>>,
    input: Stdin,
    will_exit: bool,
}

impl Terminal {
    pub fn new() -> Terminal {
        let mut terminal = Terminal {
            mode: Mode::new(vec![
                Box::new(Binding::new(
                    vec![BindingMember::BindingKey(Key::Char('q'))],
                    Box::new(|terminal, keys| {
                        //terminal.exit();
                    }),
                )),
            ]),
            output: AlternateScreen::from(stdout().into_raw_mode().unwrap()),
            input: stdin(),
            will_exit: false,
        };

        write!(terminal.output, "{}{}", clear::All, cursor::Goto(1, 1));
        terminal.output.flush().unwrap();
        terminal
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn exit(&mut self) {
        self.will_exit = true;
    }

    pub fn update(&mut self) {
        let mut keys: Vec<Key> = Vec::new();
        for c in self.input.keys() {
            keys.push(c.unwrap());
            match self.mode.consume(self, &mut keys) {
                Option::Some(_) => {
                    keys = Vec::new();
                    self.output.flush().unwrap();
                }
                Option::None => continue,
            }

            if self.will_exit {
                return;
            } else {
                break;
            }
        }
    }
}
