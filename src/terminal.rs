extern crate termion;

use std::boxed::Box;
use std::io::{stdin, stdout, Result, Stdin, Stdout, Write};
use std::vec::Vec;
use terminal::termion::event::Key;
use terminal::termion::input::Keys;
use terminal::termion::raw::{IntoRawMode, RawTerminal};
use terminal::termion::screen::AlternateScreen;
use terminal::termion::terminal_size;
use terminal::termion::{clear, color, cursor, style};

pub use terminal::termion::input::TermRead;

enum Control {
    SetCursor(u16, u16),
    CursorLeft(u16),
    CursorRight(u16),
    CursorUp(u16),
    CursorDown(u16),
    PutChar(char),
    SplitVertically,
    SplitHorizontally,
    Exit,
}

enum Direction {
    Vertical,
    Horizontal,
}

pub trait Pane {
    fn draw(&self, x: u16, y: u16, width: u16, height: u16) -> String;
}

struct VerticalSplit {
    top: Box<Pane>,
    bottom: Box<Pane>,
}

impl Pane for VerticalSplit {
    fn draw(&self, x: u16, y: u16, width: u16, height: u16) -> String {
        let height = height / 2;
        String::from(format!(
            "{}{}{}{}",
            cursor::Goto(x, y),
            self.top.draw(x, y, width, height),
            cursor::Goto(x, y + height),
            self.bottom.draw(x, y + height + 1, width, height)
        ))
    }
}

struct HorizontalSplit {
    left: Box<Pane>,
    right: Box<Pane>,
}

impl Pane for HorizontalSplit {
    fn draw(&self, x: u16, y: u16, width: u16, height: u16) -> String {
        let width = width / 2;
        String::from(format!(
            "{}{}{}{}",
            cursor::Goto(x, y),
            self.left.draw(x, y, width, height),
            cursor::Goto(x + width, y),
            self.right.draw(x + width, y, width, height)
        ))
    }
}

struct BufferPane {
    // TODO connect to buffer
}

impl Pane for BufferPane {
    fn draw(&self, x: u16, y: u16, width: u16, height: u16) -> String {
        let mut content = String::new();
        for i in 1..height {
            content.push_str(
                format!(
                    "{}~{}|",
                    cursor::Goto(x, y + i),
                    cursor::Goto(x + width - 1, y + i)
                ).as_str(),
            );
        }

        String::from(format!(
            "{}{}| [no name] |{}[ nep{}]",
            content,
            cursor::Goto(x, y),
            cursor::Goto(x, y + height),
            cursor::Goto(x + width - 1, y + height)
        ))
    }
}

impl BufferPane {
    fn split_horizontally(self) -> HorizontalSplit {
        HorizontalSplit {
            left: Box::new(self.clone()),
            right: Box::new(self),
        }
    }

    fn split_vertically(self) -> VerticalSplit {
        VerticalSplit {
            top: Box::new(self.clone()),
            bottom: Box::new(self),
        }
    }
}

impl Clone for BufferPane {
    fn clone(&self) -> Self {
        //*self
        BufferPane {}
    }
}

pub struct Terminal {
    output: AlternateScreen<RawTerminal<Stdout>>,
    buffer: Vec<Key>,
    root: Box<Pane>,
    width: u16,
    height: u16,
}

impl Terminal {
    pub fn new() -> Terminal {
        let (width, height) = terminal_size().unwrap();
        let mut terminal = Terminal {
            output: AlternateScreen::from(stdout().into_raw_mode().unwrap()),
            buffer: Vec::new(),
            root: Box::new(BufferPane {}),
            width: width,
            height: height,
        };

        write!(terminal.output, "{}{}", clear::All, cursor::Goto(3, 2));
        terminal.draw();
        terminal
    }

    pub fn start(&mut self) {
        let input = stdin();
        for key in input.keys() {
            let key = key.unwrap();
            // self.buffer.push(key);

            // TODO remove and reimplement with custom bindings
            let control = match key {
                // Key::Char('h') => Control::CursorLeft(1),
                // Key::Char('j') => Control::CursorDown(1),
                // Key::Char('k') => Control::CursorUp(1),
                // Key::Char('l') => Control::CursorRight(1),
                // Key::Char('q') => Control::Exit,
                Key::Char(x) => Control::PutChar(x),
                _ => Control::Exit,
            };

            if let Control::Exit = control {
                break;
            } else {
                self.apply_control(control);
                self.draw();
            }
        }
    }

    fn draw(&mut self) {
        let (width, height) = terminal_size().unwrap();
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            write!(self.output, "{}", clear::All);
        }

        write!(
            self.output,
            "{}{}{}{}{}",
            cursor::Save,
            cursor::Hide,
            self.root.draw(1, 1, width, height),
            cursor::Restore,
            cursor::Show
        );
        self.output.flush().unwrap();
    }

    fn apply_control(&mut self, control: Control) -> Result<()> {
        // TODO reimplement with buffer and tab based controls
        match control {
            Control::SetCursor(x, y) => write!(self.output, "{}", cursor::Goto(x, y)),
            Control::CursorLeft(x) => write!(self.output, "{}", cursor::Left(x)),
            Control::CursorRight(x) => write!(self.output, "{}", cursor::Right(x)),
            Control::CursorUp(x) => write!(self.output, "{}", cursor::Up(x)),
            Control::CursorDown(x) => write!(self.output, "{}", cursor::Down(x)),
            Control::PutChar(x) => write!(self.output, "{}", x),
            Control::SplitVertically => Ok(()),
            Control::SplitHorizontally => Ok(()),
            Control::Exit => Ok(()),
        }
    }
}
