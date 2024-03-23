#![allow(dead_code)]

use super::{ast::StructDef, common::Loc};

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {

    // stack
    PushInt(usize),
    PushStr(String),
    PushCStr(String),
    PushChar(char),
    StructPath(Vec<String>), // foo::bar
    StructItem(Vec<String>), // foo.bar
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
    StructDef,
    TypeDef,
    Inline,
    Export,
    Extern,
    Returns,
    With,
}

#[derive(Clone, PartialEq)]
pub enum TypeType {
    Ptr,
    U8,
    U16,
    U32,
    U64,
    Void,
    Any,
    Custom(Vec<TypeType>),
    Struct(StructDef)
}

impl TypeType {
    pub fn get_size(&self) -> usize {
        match self {
            TypeType::Ptr => std::mem::size_of::<*const ()>(),
            TypeType::U8 => 1,
            TypeType::U16 => 2,
            TypeType::U32 => 4,
            TypeType::U64 => 8,
            TypeType::Void => 0,
            TypeType::Any => 0,
            TypeType::Custom(ts) => ts.iter().map(|f| f.get_size()).sum(),
            TypeType::Struct(s) => s.size,
        }
    }
}

impl std::fmt::Debug for TypeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ptr => write!(f, "Ptr"),
            Self::U8 => write!(f, "U8"),
            Self::U16 => write!(f, "U16"),
            Self::U32 => write!(f, "U32"),
            Self::U64 => write!(f, "U64"),
            Self::Void => write!(f, "Void"),
            Self::Any => write!(f, "Any"),
            Self::Custom(arg0) => f.debug_tuple("Custom").field(arg0).finish(),
            Self::Struct(arg0) => write!(f, "{} {}{:?}", arg0.size, arg0.ident, arg0.body),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword(KeywordType),
    Type(TypeType),
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