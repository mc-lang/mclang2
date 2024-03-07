

pub struct Loc {
    pub ln: usize,
    pub col: usize,
    pub file: String
}

impl Loc {
    pub fn new(file: String, ln: usize, col: usize) -> Self {
        Self {
            ln,
            col,
            file,
        }
    }
}

impl ToString for Loc {
    fn to_string(&self) -> String {
        format!("{}:{}:{}", self.file, self.ln, self.col)
    }
}

impl Into<String> for Loc {
    fn into(self) -> String {
        self.to_string()
    }
}