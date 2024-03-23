use std::fmt::Display;


#[derive(Debug, Clone, Default, PartialEq)]
pub struct Loc {
    pub file: String,
    pub line: usize,
    pub col: usize
}


impl Loc {
    pub fn new<T: Into<String>>(f: T, line: usize, col: usize) -> Self {
        Self {
            file: f.into(),
            line,
            col,
        }
    }
    pub fn inc_line(&mut self) {
        self.line += 1;
        self.col = 0;
    }

    pub fn inc_col(&mut self) {
        self.col += 1;
    }
}

impl Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.col)?;
        Ok(())
    }
}


