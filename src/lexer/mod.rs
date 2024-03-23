use std::path::Path;
use anyhow::bail;

use crate::{error, types::{common::Loc, token::{InstructionType, KeywordType, Token, TokenType, TypeType}}};



pub struct Lexer {
    pub loc: Loc,
    pub tokens: Vec<Token>
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            loc: Default::default(),
            tokens: Default::default(),
        }
    }


    pub fn lex(&mut self, file: &Path) -> anyhow::Result<&mut Self> {
        self.reset(file);

        let chars = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to open file {file:?} : {e}");
                bail!("");
            }
        }.chars().collect::<Vec<char>>();


        let mut idx = 0;
        let mut buf = String::new();
        let mut is_searching = false;
        
        if let Err(_) = self.go_to_first_char(&chars, &mut idx) {
            return Ok(self);
        }

        let mut start_loc = self.loc.clone();
        while idx < chars.len() {

            match chars[idx] {

                'c' if chars.get(idx + 1) == Some(&'"') => {
                    start_loc = self.loc.clone();
                    is_searching = true;
                    idx += 2; // skip c and "
                    self.loc.col += 2;

                    if !buf.is_empty() {
                        debug!({loc => self.loc() }, "buffer was not empty, intresting");
                    }
                    
                    loop {
                        if chars[idx] == '"' && chars[idx-1] != '\\' {
                            break;
                        }
                        buf.push(chars[idx]);
                        if chars[idx] == '\n' {
                            self.loc.inc_line()
                        }
                        self.loc.inc_col();
                        idx += 1;
                    }

                    buf.push('\0');
                    let str = self.unescape(&&buf);
                    self.loc.inc_col();
                    self.tokens.push(Token::new(TokenType::Instruction(InstructionType::PushCStr(str)), self.loc(), buf.clone()));
                    buf.clear();
                }

                '"' => {
                    start_loc = self.loc.clone();
                    is_searching = true;
                    idx += 1; // skip "
                    self.loc.col += 1;

                    if !buf.is_empty() {
                        debug!({loc => self.loc() }, "buffer was not empty, intresting ({buf:?})");
                    }
                    
                    // while chars.get(idx+1) != Some(&'"') && chars[idx] != '\\' && chars.get(idx+1).is_some() {
                    loop {
                        if chars[idx] == '"' && chars[idx-1] != '\\' {
                            break;
                        }
                        buf.push(chars[idx]);
                        if chars[idx] == '\n' {
                            self.loc.inc_line()
                        }
                        self.loc.inc_col();
                        idx += 1;
                    }


                    let str = self.unescape(&buf);
                    self.loc.inc_col();
                    self.tokens.push(Token::new(TokenType::Instruction(InstructionType::PushStr(str)), start_loc.clone(), buf.clone()));
                    buf.clear();
                }

                '\'' => {
                    start_loc = self.loc.clone();
                    is_searching = true;
                    idx += 1; // skip '
                    self.loc.col += 1;

                    if !buf.is_empty() {
                        debug!({loc => self.loc() }, "buffer was not empty, intresting ({buf})");
                    }
                    
                    loop {
                        if chars[idx] == '"' && chars[idx-1] != '\\' {
                            break;
                        }
                        buf.push(chars[idx]);
                        if chars[idx] == '\n' {
                            self.loc.inc_line()
                        }
                        self.loc.inc_col();
                        idx += 1;
                    }

                    let str = self.unescape(&&&buf);
                    if str.len() > 1 {
                        error!({loc => self.loc()}, "Chars can only have 1 char");
                        bail!("")
                    }
                    
                    self.loc.inc_col();
                    self.tokens.push(Token::new(TokenType::Instruction(InstructionType::PushStr(str)), self.loc(), buf.clone()));
                    buf.clear();
                }
                ':' if chars.get(idx + 1) == Some(&':') => {
                    let mut p_buf = vec![buf.clone()];
                    buf.clear();
                    idx += 2; // skip ::
                    self.loc.col += 2;

                    while idx < chars.len() {
                        match chars[idx] {
                            ' ' | '\n' | '\r' => {
                                if !p_buf.is_empty() {
                                    p_buf.push(buf.clone());
                                }

                                self.tokens.push(Token::new(TokenType::Instruction(InstructionType::StructPath(p_buf.clone())), start_loc.clone(), p_buf.clone().join("::")));
                                buf.clear();
                                break;
                            }
                            c @ ('\'' | '"') => {
                                error!({loc => self.loc()}, "Invalid char in struct path token, expected /a-z|A-Z|0-9|_|-/ got {c}");
                                bail!("")
                            }

                            ':' if chars.get(idx + 1) == Some(&':') => {
                                if buf.is_empty() {
                                    error!({loc => self.loc()}, "Invalid char in struct path token, expected /a-z|A-Z|0-9|_|-/ got '.'");
                                    bail!("")
                                }
                                idx += 2; // skip ::
                                self.loc.col += 2;
                                p_buf.push(buf.clone());
                                buf.clear();
                            }

                            c => {
                                buf.push(c);
                                idx += 1;
                                self.loc.inc_col();
                            }
                        }
                    }
                }

                '.' if !buf.is_empty() => {
                    let mut p_buf = vec![buf.clone()];
                    buf.clear();
                    idx += 1; // skip .
                    self.loc.inc_col();

                    while idx < chars.len() {
                        match chars[idx] {
                            ' ' | '\n' | '\r' => {
                                if !p_buf.is_empty() {
                                    p_buf.push(buf.clone());
                                }
                                self.tokens.push(Token::new(TokenType::Instruction(InstructionType::StructItem(p_buf.clone())), start_loc.clone(), p_buf.clone().join(".")));
                                buf.clear();
                                break;
                            }
                            c @ ('\'' | '"') => {
                                error!({loc => self.loc()}, "Invalid char in struct access token, expected /a-z|A-Z|0-9|_|-/ got {c}");
                                bail!("")
                            }

                            '.' => {
                                if buf.is_empty() {
                                    error!({loc => self.loc()}, "Invalid char in struct access token, expected /a-z|A-Z|0-9|_|-/ got '.'");
                                    bail!("")
                                }
                                idx += 1; // skip .
                                self.loc.col += 1;
                                p_buf.push(buf.clone());
                                buf.clear();
                            }

                            c => {
                                buf.push(c);
                                idx += 1;
                                self.loc.inc_col();
                            }
                        }
                    }
                }

                ch @ (' ' | '\n' | '\r') => {
                    if ch == '\n' {
                        self.loc.inc_line();
                    } else {
                        self.loc.inc_col();
                    }
                    if !buf.is_empty() {
                        //TODO: Implement signed ints
                        if let Ok(int) = parse_int::parse::<usize>(&buf) {
                            self.tokens.push(Token::new(TokenType::Instruction(InstructionType::PushInt(int)), start_loc.clone(), buf.clone()));    
                        } else {
                            let token_type = self.match_token_type(&buf);
                            self.tokens.push(Token::new(token_type, start_loc.clone(), buf.clone()));
                        }

                        buf.clear();
                        is_searching = true;
                    }
                }

                '/' if chars.get(idx + 1) == Some(&'/') => {
                    let mut c = chars.get(idx);
                    while c.is_some() && c != Some(&'\n') {
                        self.loc.inc_col();
                        idx += 1;
                        c = chars.get(idx);
                    }
                    self.loc.inc_line();
                }


                ch => {
                    if is_searching {
                        is_searching = false;
                        start_loc = self.loc.clone();
                    }

                    buf.push(ch);
                    self.loc.inc_col();
                }

            }
            idx += 1;
        }
        //? Add last token
        //TODO: Implement signed ints
        if !buf.is_empty() {
            if let Ok(int) = parse_int::parse::<usize>(&buf) {
                self.tokens.push(Token::new(TokenType::Instruction(InstructionType::PushInt(int)), start_loc.clone(), buf.clone()));    
            } else {
                let token_type = self.match_token_type(&buf);
                self.tokens.push(Token::new(token_type, start_loc.clone(), buf.clone()));
            }
        }

        // for t in &self.tokens {
        //     debug!({loc => t.loc.clone()}, "token: {:?}", t.typ);
        // }

        Ok(self)
    }

    fn go_to_first_char(&mut self, chars: &Vec<char>, idx: &mut usize) -> anyhow::Result<()> {
        loop {
            if let Some(c) = chars.get(*idx) {
                match c {
                    ' ' | '\r' => self.loc.inc_col(),
                    '\n' => self.loc.inc_line(),
                    _ => break,
                }
                *idx += 1;
            } else {
                warn!("Empty program");
                bail!("")
            }
        }


        Ok(())
    }

    fn match_token_type(&self, s: &str) -> TokenType {
        match s {
            "if"         => TokenType::Keyword(KeywordType::If),
            "else"       => TokenType::Keyword(KeywordType::Else),
            "end"        => TokenType::Keyword(KeywordType::End),
            "while"      => TokenType::Keyword(KeywordType::While),
            "do"         => TokenType::Keyword(KeywordType::Do),
            "include"    => TokenType::Keyword(KeywordType::Include),
            "memory"     => TokenType::Keyword(KeywordType::Memory),
            "const"      => TokenType::Keyword(KeywordType::Constant),
            "fn"         => TokenType::Keyword(KeywordType::Function),
            "then"       => TokenType::Keyword(KeywordType::Then),
            "done"       => TokenType::Keyword(KeywordType::Done),
            "typedef"    => TokenType::Keyword(KeywordType::TypeDef),
            "structdef"  => TokenType::Keyword(KeywordType::StructDef),
            "inline"     => TokenType::Keyword(KeywordType::Inline),
            "export"     => TokenType::Keyword(KeywordType::Export),
            "extern"     => TokenType::Keyword(KeywordType::Extern),
            "returns"    => TokenType::Keyword(KeywordType::Returns),
            "with"       => TokenType::Keyword(KeywordType::With),
            "drop"       => TokenType::Instruction(InstructionType::Drop),
            "_dbg_print" => TokenType::Instruction(InstructionType::Print),
            "dup"        => TokenType::Instruction(InstructionType::Dup),
            "rot"        => TokenType::Instruction(InstructionType::Rot),
            "over"       => TokenType::Instruction(InstructionType::Over),
            "swap"       => TokenType::Instruction(InstructionType::Swap),
            "sub"        => TokenType::Instruction(InstructionType::Minus),
            "add"        => TokenType::Instruction(InstructionType::Plus),
            "eq"         => TokenType::Instruction(InstructionType::Equals),
            "gt"         => TokenType::Instruction(InstructionType::Gt),
            "lt"         => TokenType::Instruction(InstructionType::Lt),
            "ge"         => TokenType::Instruction(InstructionType::Ge),
            "le"         => TokenType::Instruction(InstructionType::Le),
            "neq"        => TokenType::Instruction(InstructionType::NotEquals),
            "band"       => TokenType::Instruction(InstructionType::Band),
            "bor"        => TokenType::Instruction(InstructionType::Bor),
            "shr"        => TokenType::Instruction(InstructionType::Shr),
            "shl"        => TokenType::Instruction(InstructionType::Shl),
            "divmod"     => TokenType::Instruction(InstructionType::DivMod),
            "mul"        => TokenType::Instruction(InstructionType::Mul),
            "read8"      => TokenType::Instruction(InstructionType::Read8),
            "write8"     => TokenType::Instruction(InstructionType::Write8),
            "read32"     => TokenType::Instruction(InstructionType::Read32),
            "write32"    => TokenType::Instruction(InstructionType::Write32),
            "read64"     => TokenType::Instruction(InstructionType::Read64),
            "write64"    => TokenType::Instruction(InstructionType::Write64),
            "syscall0"   => TokenType::Instruction(InstructionType::Syscall0),
            "syscall1"   => TokenType::Instruction(InstructionType::Syscall1),
            "syscall2"   => TokenType::Instruction(InstructionType::Syscall2),
            "syscall3"   => TokenType::Instruction(InstructionType::Syscall3),
            "syscall4"   => TokenType::Instruction(InstructionType::Syscall4),
            "syscall5"   => TokenType::Instruction(InstructionType::Syscall5),
            "syscall6"   => TokenType::Instruction(InstructionType::Syscall6),
            "(bool)"     => TokenType::Instruction(InstructionType::CastBool),
            "(ptr)"      => TokenType::Instruction(InstructionType::CastPtr),
            "(int)"      => TokenType::Instruction(InstructionType::CastInt),
            "(void)"     => TokenType::Instruction(InstructionType::CastVoid),
            "return"     => TokenType::Instruction(InstructionType::Return),
            "ptr"        => TokenType::Type(TypeType::Ptr),
            "u8"         => TokenType::Type(TypeType::U8),
            "u16"        => TokenType::Type(TypeType::U16),
            "u32"        => TokenType::Type(TypeType::U32),
            "u64"        => TokenType::Type(TypeType::U64),
            "void"       => TokenType::Type(TypeType::Void),
            "any"        => TokenType::Type(TypeType::Any),
            t => TokenType::Unknown(t.to_string())
        }
    }

    pub fn reset(&mut self, file: &Path) -> &mut Self {
        self.loc.file = file.to_string_lossy().to_string();
        self.loc.line = 1;
        self.loc.col = 0;
        self.tokens = Vec::new();
        self
    }

    fn loc(&self) -> Loc {
        self.loc.clone()
    }
    fn unescape(&self, s: &String) -> String {
        //TODO: add more escapes
        s
            .replace("\\n", "\n")
            .replace("\\0", "\0")
    }

}