use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;

use super::buffer::Buffer;
use super::cursor::Cursor;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::{clear, cursor, style, terminal_size};

pub struct Interface<T: 'static + Send + Sync + std::io::Write> {
    write: Arc<Mutex<RawTerminal<AlternateScreen<T>>>>,
    offset: Arc<Mutex<usize>>,
    buffer: Arc<Mutex<Buffer>>,
    cursor: Arc<Mutex<Cursor>>,
    size: Arc<Mutex<(u16, u16)>>,
    draw_ready: Arc<(Mutex<bool>, Condvar)>,
    cursor_update: Arc<(Mutex<bool>, Condvar)>,
    will_stop: Arc<Mutex<bool>>,
    splashed: Arc<Mutex<bool>>,
}

impl<T: 'static + Send + Sync + std::io::Write> Interface<T> {
    pub fn from(t: T) -> Interface<T> {
        let buffer = Arc::new(Mutex::new(Buffer::new()));
        let interface = Interface::from_buffer(t, buffer);
        *interface.splashed.lock().unwrap() = false;
        interface
    }

    pub fn from_buffer(t: T, buffer: Arc<Mutex<Buffer>>) -> Interface<T> {
        Interface {
            size: Arc::new(Mutex::new(terminal_size().unwrap())),
            write: Arc::new(Mutex::new(
                AlternateScreen::from(t).into_raw_mode().unwrap(),
            )),
            offset: Arc::new(Mutex::new(0)),
            buffer: Arc::clone(&buffer),
            cursor: Arc::new(Mutex::new(Cursor::new(&buffer))),
            draw_ready: Arc::new((Mutex::new(true), Condvar::new())),
            cursor_update: Arc::new((Mutex::new(true), Condvar::new())),
            will_stop: Arc::new(Mutex::new(false)),
            splashed: Arc::new(Mutex::new(true)),
        }
    }

    fn start_resize(&mut self) -> JoinHandle<()> {
        let size = Arc::clone(&self.size);
        let draw_ready = Arc::clone(&self.draw_ready);
        let cursor_update = Arc::clone(&self.cursor_update);
        let will_stop = Arc::clone(&self.will_stop);
        let splashed = Arc::clone(&self.splashed);

        thread::spawn(move || loop {
            if *will_stop.lock().unwrap() {
                break;
            }

            let mut size_change = false;
            let new_size = terminal_size().unwrap();
            {
                let mut size = size.lock().unwrap();
                if *size != new_size {
                    *size = new_size;
                    size_change = true;
                }
            }

            if size_change {
                *splashed.lock().unwrap() = true;
                let &(ref lock, ref cvar) = &*cursor_update;
                *lock.lock().unwrap() = true;
                cvar.notify_one();

                let &(ref lock, ref cvar) = &*draw_ready;
                *lock.lock().unwrap() = true;
                cvar.notify_one();
            }
        })
    }

    fn start_draw(&mut self) -> JoinHandle<()> {
        let size = Arc::clone(&self.size);
        let draw_ready = Arc::clone(&self.draw_ready);
        let will_stop = Arc::clone(&self.will_stop);
        let write = Arc::clone(&self.write);
        let buffer = Arc::clone(&self.buffer);
        let offset = Arc::clone(&self.offset);
        let splashed = Arc::clone(&self.splashed);

        thread::spawn(move || loop {
            if *will_stop.lock().unwrap() {
                break;
            }

            let &(ref lock, ref cvar) = &*draw_ready;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }

            let mut write = write.lock().unwrap();
            let (width, height) = *size.lock().unwrap();
            let offset = *offset.lock().unwrap();
            let buffer = buffer.lock().unwrap();

            write!(write, "{}{}", cursor::Hide, cursor::Save);
            if *splashed.lock().unwrap() {
                write!(write, "{}", clear::All);
            }

            let line_number_width = f64::log10(buffer.len() as f64).floor() as u16 + 2;

            let mut written = 0u16;
            let mut used = 0usize;
            while written < height.saturating_sub(1) {
                if offset + used >= buffer.len() {
                    break;
                }

                let line = buffer.borrow_line(offset + used);
                write!(write, "{}", cursor::Goto(1, written as u16 + 1));
                write!(
                    write,
                    "{line: >0$} ",
                    line_number_width as usize,
                    line = offset + used + 1
                );

                let mut characters_written = line_number_width + 1;
                for character in line.borrow_chars().iter() {
                    write!(write, "{}", character);
                    characters_written += 1;

                    if characters_written >= width {
                        written += 1;
                        characters_written = line_number_width + 1;
                        if written >= height.saturating_sub(2) {
                            break;
                        } else {
                            write!(write, "{}", cursor::Goto(line_number_width, written + 1));
                            write!(write, "{line: >0$}", line_number_width as usize, line = " ");
                        }
                    }
                }
                written += 1;
                used += 1;
            }

            while written < height.saturating_sub(1) {
                write!(write, "{}", cursor::Goto(1, written + 1));
                write!(write, "{}", "~");
                written += 1;
            }

            write!(
                write,
                "{}{} {} ",
                cursor::Goto(1, height),
                style::Invert,
                buffer.get_name(),
            );

            write!(write, "{}", style::Reset);

            write!(
                write,
                "{}nep ",
                cursor::Goto(width.saturating_sub(3), height),
            );

            write!(write, "{}{}", cursor::Restore, cursor::Show);
            *ready = false;
            write.flush().unwrap();
        })
    }

    fn start_cursor_update(&mut self) -> JoinHandle<()> {
        let size = Arc::clone(&self.size);
        let draw_ready = Arc::clone(&self.draw_ready);
        let cursor_update = Arc::clone(&self.cursor_update);
        let will_stop = Arc::clone(&self.will_stop);
        let write = Arc::clone(&self.write);
        let buffer = Arc::clone(&self.buffer);
        let cursor = Arc::clone(&self.cursor);
        let offset = Arc::clone(&self.offset);

        thread::spawn(move || loop {
            if *will_stop.lock().unwrap() {
                break;
            }

            let &(ref lock, ref cvar) = &*cursor_update;
            let mut ready = lock.lock().unwrap();
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }

            let (width, height) = *size.lock().unwrap();
            let mut new_offset = false;

            {
                let mut offset = offset.lock().unwrap();
                let line = cursor.lock().unwrap().line();

                if line <= *offset {
                    *offset = line;
                    new_offset = true;
                } else if line > *offset + height.saturating_sub(2) as usize {
                    *offset = line - height.saturating_sub(2) as usize;
                    new_offset = true;
                }
            }

            if new_offset {
                let &(ref lock, ref cvar) = &*draw_ready;
                *lock.lock().unwrap() = true;
                cvar.notify_one();
            }

            let mut write = write.lock().unwrap();
            let offset = *offset.lock().unwrap();
            let cursor = cursor.lock().unwrap();

            let line = cursor.line();
            let column = cursor.column();

            let buffer = buffer.lock().unwrap();
            let line_number_width = f64::log10(buffer.len() as f64).floor() as u16 + 3;
            let max_width = width - line_number_width;

            let mut displacement_y = 0;
            for i in 0..(line - offset) {
                let len = buffer.borrow_line(offset + i as usize).len();
                if len > max_width as usize {
                    displacement_y += len / (max_width as usize);
                }
            }

            displacement_y += column / (max_width as usize);
            let displacement_x = line_number_width as usize + column % (max_width as usize);
            write!(
                write,
                "{}",
                cursor::Goto(
                    1 + displacement_x as u16,
                    1 + (line - offset + displacement_y) as u16
                )
            );

            *ready = false;
            write.flush().unwrap();
        })
    }

    fn notify_cursor(&self) {
        *self.splashed.lock().unwrap() = true;
        let &(ref lock, ref cvar) = &*self.cursor_update;
        *lock.lock().unwrap() = true;
        cvar.notify_one();
    }

    fn notify_draw(&self) {
        *self.splashed.lock().unwrap() = true;
        let &(ref lock, ref cvar) = &*self.draw_ready;
        *lock.lock().unwrap() = true;
        cvar.notify_one();
    }

    fn show_splash(&self) {
        let mut write = self.write.lock().unwrap();
        let (width, height) = *self.size.lock().unwrap();
        let (width, height) = (width as usize, height as usize);

        let splash = include_str!("splash").to_string();
        let mut splash = splash.split("#").peekable();

        let (lower, _) = splash.size_hint();

        write!(write, "{}{}", cursor::Goto(5, 5), lower);

        if width >= splash.peek().unwrap().to_string().len() + 6 && height >= lower + 8 {
            write!(write, "{}", clear::All);
            for (i, line) in splash.enumerate() {
                write!(
                    write,
                    "{}{}",
                    cursor::Goto(
                        ((width - line.len()) / 2) as u16,
                        ((height - lower) / 2 + i - 8) as u16
                    ),
                    line,
                );
            }
            write.flush().unwrap();
        }
    }

    pub fn start<U: std::io::Read>(mut self, u: U) {
        if !*self.splashed.lock().unwrap() {
            self.show_splash();
        }
        let draw_thread = self.start_draw();
        let cursor_thread = self.start_cursor_update();
        let resize_thread = self.start_resize();

        for key in u.keys() {
            match key.unwrap() {
                Key::Char('\t') => {
                    {
                        let mut cursor = self.cursor.lock().unwrap();
                        let line = cursor.line();
                        let column = cursor.column();
                        for _ in 0..4 {
                            self.buffer
                                .lock()
                                .unwrap()
                                .borrow_line_mut(line)
                                .insert(column, ' ');
                            cursor.right();
                        }
                    }
                    self.notify_cursor();
                    self.notify_draw();
                }
                Key::Char('\n') => {
                    {
                        let mut cursor = self.cursor.lock().unwrap();
                        let line = cursor.line();
                        let column = cursor.column();
                        self.buffer.lock().unwrap().split_line(line, column);
                        cursor.jump_next();
                    }
                    self.notify_cursor();
                    self.notify_draw();
                }
                Key::Char(c) => {
                    {
                        let mut cursor = self.cursor.lock().unwrap();
                        let line = cursor.line();
                        let column = cursor.column();
                        self.buffer
                            .lock()
                            .unwrap()
                            .borrow_line_mut(line)
                            .insert(column, c);
                        cursor.right();
                    }
                    self.notify_cursor();
                    self.notify_draw();
                }
                Key::Up => {
                    self.cursor.lock().unwrap().up();
                    self.notify_cursor();
                }
                Key::Down => {
                    self.cursor.lock().unwrap().down();
                    self.notify_cursor();
                }
                Key::Left => {
                    self.cursor.lock().unwrap().left();
                    self.notify_cursor();
                }
                Key::Right => {
                    self.cursor.lock().unwrap().right();
                    self.notify_cursor();
                }
                Key::Backspace => {
                    {
                        let mut cursor = self.cursor.lock().unwrap();
                        let line = cursor.line();
                        let column = cursor.column();

                        if column > 0 {
                            self.buffer
                                .lock()
                                .unwrap()
                                .borrow_line_mut(line)
                                .delete(column - 1);
                            if column < self.buffer.lock().unwrap().borrow_line(line).len() {
                                cursor.left();
                            }
                        } else {
                            let position = self.buffer.lock().unwrap().merge_line(line);
                            cursor.jump_prev(position);
                        }
                    }

                    self.notify_cursor();
                    self.notify_draw();
                }
                Key::Esc => {
                    *self.will_stop.lock().unwrap() = true;
                    resize_thread.join().unwrap();
                    self.notify_cursor();
                    cursor_thread.join().unwrap();
                    self.notify_draw();
                    draw_thread.join().unwrap();
                    break;
                }
                Key::Ctrl('s') => {
                    self.buffer.lock().unwrap().write_back();
                    self.notify_draw();
                }
                _ => continue,
            }
        }
    }
}
