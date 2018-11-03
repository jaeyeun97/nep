pub struct Line {
    characters: Vec<char>,
}

impl Line {
    fn new() -> Line {
        Line { characters: vec![] }
    }

    #[allow(dead_code)]
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

    pub fn len(&mut self) -> usize {
        self.characters.len()
    }
}

pub struct Buffer {
    lines: Vec<Line>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            lines: vec![Line::new()],
        }
    }

    pub fn split_line(&mut self, line: usize, column: usize) {
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

    pub fn borrow_line(&mut self, line: usize) -> &mut Line {
        &mut self.lines[line]
    }

    pub fn len(&mut self) -> usize {
        self.lines.len()
    }
}
