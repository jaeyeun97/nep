use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

pub struct Line {
    characters: Vec<char>,
}

impl Line {
    fn new() -> Line {
        Line { characters: vec![] }
    }

    fn from<T: Iterator<Item = char>>(t: T) -> Line {
        Line {
            characters: t.collect(),
        }
    }

    fn split(&mut self, index: usize) -> Line {
        let len = self.len();
        Line::from(self.characters.drain(index..len))
    }

    pub fn borrow_chars(&self) -> &Vec<char> {
        &self.characters
    }

    pub fn insert(&mut self, index: usize, c: char) {
        self.characters.insert(index, c);
    }

    pub fn delete(&mut self, index: usize) {
        self.characters.remove(index);
    }

    pub fn len(&self) -> usize {
        self.characters.len()
    }
}

pub struct Buffer {
    lines: Vec<Line>,
    file: Option<File>,
    filename: Option<String>,
    dirty: bool,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            lines: vec![Line::new()],
            file: None,
            filename: None,
            dirty: false,
        }
    }

    pub fn from(filename: String) -> Buffer {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(PathBuf::from(filename.clone()))
            .expect("Failed to open buffer");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file contents");

        let mut lines = contents
            .lines()
            .map(|x| Line::from(x.chars()))
            .collect::<Vec<Line>>();

        if lines.len() == 0 {
            lines.push(Line::new());
        }

        Buffer {
            lines: lines,
            file: Some(file),
            filename: Some(filename),
            dirty: false,
        }
    }

    pub fn write_back(&mut self) {
        match self.file {
            Some(ref mut file) => {
                file.seek(SeekFrom::Start(0)).unwrap();
                let contents = self
                    .lines
                    .iter()
                    .map(|x| x.characters.iter().collect::<String>() + "\n")
                    .collect::<String>();
                let contents = contents.as_bytes();
                let len = contents.len();
                file.write(contents).unwrap();
                file.set_len(len as u64).unwrap();
                self.dirty = false;
            }
            None => return,
        }
    }

    pub fn split_line(&mut self, line: usize, column: usize) {
        self.dirty = true;
        let new_line = self.lines[line].split(column);
        self.lines.insert(line + 1, new_line);
    }

    pub fn merge_line(&mut self, line: usize) -> usize {
        if line > 0 {
            let mut to_merge = self.lines.remove(line);
            let merging = &mut self.lines[line.saturating_sub(1)];
            let len = merging.len();
            merging.characters.append(&mut to_merge.characters);
            len
        } else {
            0
        }
    }

    pub fn borrow_line(&self, line: usize) -> &Line {
        &self.lines[line]
    }

    pub fn borrow_line_mut(&mut self, line: usize) -> &mut Line {
        self.dirty = true;
        &mut self.lines[line]
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn get_name(&self) -> String {
        (match self.filename {
            Some(ref filename) => filename.clone(),
            None => "[no name]".to_string(),
        }) + if self.dirty() { "*" } else { "" }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }
}
