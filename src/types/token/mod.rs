#![allow(dead_code)]

use super::common::Loc;

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {

    // stack
    PushInt(usize),
    PushStr(String),
    PushCStr(String),
    PushChar(char),
    Drop,
    Print,
    Dup,
    Rot, // a b c => b c a
    Over, // a b => a b a
    Swap, // a b => b a

    // math
    Minus,
    Plus,
    Equals,
    Gt,
    Lt,
    Ge,
    Le,
    NotEquals,
    Band, // &
    Bor, // |
    Shr, // >>
    Shl,  // <<
    DivMod, // /
    Mul,


    // mem
    Read8,
    Write8,
    Read32,
    Write32,
    Read64,
    Write64,

    // syscalls
    Syscall0,
    Syscall1,
    Syscall2,
    Syscall3,
    Syscall4,
    Syscall5,
    Syscall6,

    CastBool,
    CastPtr,
    CastInt,
    CastVoid,

    // typing
    TypeBool,
    TypePtr,
    TypeInt,
    TypeVoid,
    // TypeStr,
    TypeAny,
    
    FnCall,
    MemUse,
    ConstUse,
    
    Return,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeywordType {
    If,
    Else,
    End,
    While,
    Do,
    Include,
    Memory,
    Constant,
    Function,
    Then,
    Done,
    Struct,
    Inline,
    Export,
    Extern,
    Returns,
    With,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword(KeywordType),
    Instruction(InstructionType),
    Unknown(String)
}


#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub typ: TokenType,
    pub loc: Loc,
    pub lexem: String,
}

impl Token {
    pub fn new(typ: TokenType, loc: Loc, lexem: String) -> Self {
        Self {
            typ,
            loc,
            lexem,
        }
    }
    pub fn loc(&self) -> Loc {
        self.loc.clone()
    }
}