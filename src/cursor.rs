use super::buffer::Buffer;
use std::sync::{Arc, Mutex};

pub struct Cursor {
    buffer: Arc<Mutex<Buffer>>,
    line: usize,
    column: usize,
}

impl Cursor {
    pub fn new(buffer: &Arc<Mutex<Buffer>>) -> Cursor {
        Cursor {
            buffer: Arc::clone(buffer),
            line: 0,
            column: 0,
        }
    }

    pub fn column(&self) -> usize {
        std::cmp::min(
            self.column,
            self.buffer.lock().unwrap().borrow_line(self.line).len(),
        )
    }

    pub fn line(&self) -> usize {
        std::cmp::min(
            self.line,
            self.buffer.lock().unwrap().len().saturating_sub(1),
        )
    }

    pub fn left(&mut self) {
        self.column = self.column();
        self.column = self.column.saturating_sub(1)
    }

    pub fn right(&mut self) {
        self.column = self.column();
        self.column = self.column.saturating_add(1);
        self.column = self.column();
    }

    pub fn up(&mut self) {
        self.line = self.line.saturating_sub(1);
    }

    pub fn down(&mut self) {
        self.line = self.line.saturating_add(1);
        self.line = self.line()
    }

    pub fn jump_next(&mut self) {
        self.down();
        self.column = 0;
    }

    pub fn jump_prev(&mut self, position: usize) {
        self.up();
        self.column = position;
        self.column = self.column();
    }
}
