use std::collections::{HashMap, HashSet};

use eyre::bail;




#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {

    // stack
    PushInt,
    PushStr,
    PushCStr,
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
    Returns,
    With,

    FnCall,
    MemUse,
    ConstUse,

    Return,
    None // Used for macros and any other non built in word definitions

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
    ConstantDef,
    Function,
    FunctionDef,
    FunctionDefExported,
    FunctionThen,
    FunctionDone,
    Inline,
    Export,
    Struct,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InternalType {
    Arrow
}
#[derive(Debug, Clone, PartialEq)]
pub enum OpType {
    Keyword(KeywordType),
    Instruction(InstructionType),
    Internal(InternalType)
}

#[derive(Debug, Clone)]
pub struct Operator{
    pub typ: OpType,
    pub tok_typ: TokenType,
    pub value: usize,
    pub text: String, //? only used for OpType::PushStr
    pub addr: Option<usize>, //? only used for OpType::PushStr
    pub jmp: usize,
    pub loc: Loc,
    pub types: (usize, usize)
}

impl Operator {
    pub fn new(typ: OpType, tok_typ: TokenType, value: usize, text: String, file: String, row: usize, col: usize) -> Self {
        Self {
            typ,
            value,
            jmp: 0,
            addr: None,
            text,
            loc: (file, row, col),
            tok_typ,
            types: (0, 0)
        }
    }
    pub fn set_addr(&mut self, addr: usize) -> Self {
        self.addr = Some(addr);
        (*self).clone()
    }

    // pub fn set_types(&mut self, args: usize, rets: usize) -> Self {
    //     self.types = (args, rets);
    //     (*self).clone()
    // }
    
}

impl OpType {
    pub fn human(&self) -> String {
        match (*self).clone() {
            OpType::Instruction(instruction) => {
                match instruction {
                    
                    InstructionType::PushInt => "Number",
                    InstructionType::PushStr => "String",
                    InstructionType::PushCStr => "CString",
                    InstructionType::Print => "_dbg_print",
                    InstructionType::Dup => "dup",
                    InstructionType::Drop => "drop",
                    InstructionType::Rot => "rot",
                    InstructionType::Over => "over",
                    InstructionType::Swap => "swap",
                    InstructionType::Plus => "+",
                    InstructionType::Minus => "-",
                    InstructionType::Equals => "=",
                    InstructionType::Gt => ">",
                    InstructionType::Lt => "<",
                    InstructionType::NotEquals => "!=",
                    InstructionType::Le => "<=",
                    InstructionType::Ge => ">=",
                    InstructionType::Band => "band",
                    InstructionType::Bor => "bor",
                    InstructionType::Shr => "shr",
                    InstructionType::Shl => "shl",
                    InstructionType::DivMod => "divmod",
                    InstructionType::Mul => "*",
                    InstructionType::Read8 => "read8",
                    InstructionType::Write8 => "write8",
                    InstructionType::Read32 => "read32",
                    InstructionType::Write32 => "write32",
                    InstructionType::Read64 => "read64",
                    InstructionType::Write64 => "write64",
                    InstructionType::Syscall0 => "syscall0",
                    InstructionType::Syscall1 => "syscall1",
                    InstructionType::Syscall2 => "syscall2",
                    InstructionType::Syscall3 => "syscall3",
                    InstructionType::Syscall4 => "syscall4",
                    InstructionType::Syscall5 => "syscall5",
                    InstructionType::Syscall6 => "syscall6",
                    InstructionType::CastBool => "cast(bool",
                    InstructionType::CastPtr => "cast(ptr)",
                    InstructionType::CastInt => "cast(int)",
                    InstructionType::CastVoid => "cast(void)",
                    InstructionType::None => "None",
                    InstructionType::MemUse => "Memory use (internal)",
                    InstructionType::FnCall => "Function Call (Internal)",
                    InstructionType::ConstUse => "Constant Use (Internal)",
                    InstructionType::Return => "return",
                    InstructionType::TypeBool => "bool",
                    InstructionType::TypePtr => "ptr",
                    InstructionType::TypeInt => "int",
                    InstructionType::TypeVoid => "void",
                    InstructionType::Returns => "returns",
                    InstructionType::With => "with",
                    InstructionType::TypeAny => "any",
                }
            }
            OpType::Keyword(keyword) => {
                match keyword {
                    KeywordType::If => "if",
                    KeywordType::Else => "else",
                    KeywordType::End => "end",
                    KeywordType::While => "while",
                    KeywordType::Do => "do",
                    KeywordType::Include => "include",
                    KeywordType::Memory => "memory",
                    KeywordType::Function => "fn",
                    KeywordType::Constant => "const",
                    KeywordType::FunctionThen => "then",
                    KeywordType::FunctionDone => "done",
                    KeywordType::ConstantDef => "constant Definition (internal)",
                    KeywordType::FunctionDef => "function definition (internal)",
                    KeywordType::FunctionDefExported => "extern function definition (internal)",
                    KeywordType::Inline => "inline",
                    KeywordType::Export => "export",
                    KeywordType::Struct => "struct",
                }
            }
            OpType::Internal(t) => panic!("{t:?}"),
            
        }.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub text: String,
    pub typ: TokenType,
    pub value: Option<usize>, //* only used for Memories
    pub addr: Option<usize>, //* only used for Memories
    pub op_typ: OpType //* only used for Memories
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TokenType {
    Word,
    Int,
    String,
    CString,
    Char
}

impl Token {
    pub fn loc(&self) -> Loc {
        (
            self.file.clone(),
            self.line,
            self.col
        )
    }
}

impl TokenType {
    pub fn human(self) -> String {
        match self {
            TokenType::Word => "Word",
            TokenType::Int => "Int",
            TokenType::String => "String",
            TokenType::CString => "CString",
            TokenType::Char => "Char"
        }.to_string()
    }
}

pub type Loc = (String, usize, usize);

#[derive(Debug, PartialEq, Clone)]
pub enum Types {
    Any,
    Bool,
    Ptr,
    Void,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Custom{
        size: u64 // in bytes
    },
    // todo: add signed numbers since we dont have them yet lol
}

impl Types {
    pub fn get_size(&self) -> u64 {
        match *self {
            Types::Any => 0, // any cant be a known size
            Types::Void => 0,
            Types::Bool => 1,
            Types::U8 |
            Types::I8 => 1,
            Types::U16 |
            Types::I16 => 2,
            Types::U32 |
            Types::I32 => 4,
            Types::Ptr |
            Types::U64 |
            Types::I64 => 8,
            Types::Custom { size } => size,
        }
    }
}

impl TryInto<Types> for &str {
    type Error = color_eyre::eyre::Error;

    fn try_into(self) -> Result<Types, Self::Error> {
        match self {
            "Any" => Ok(Types::Any),
            "Void" => Ok(Types::Void),
            "Bool" => Ok(Types::Bool),
            "U8" => Ok(Types::U8),
            "I8" => Ok(Types::I8),
            "U16" => Ok(Types::U16),
            "I16" => Ok(Types::I16),
            "U32" => Ok(Types::U32),
            "I32" => Ok(Types::I32),
            "Ptr" => Ok(Types::Ptr),
            "U64" => Ok(Types::U64),
            "I64" => Ok(Types::I64),
            _ => bail!("Unknown type {self}")
        }
    }
}

impl TryInto<Types> for String {
    type Error = color_eyre::eyre::Error;

    fn try_into(self) -> Result<Types, Self::Error> {
        self.into()
    }
}



#[derive(Debug, Clone)]
pub struct Function {
    pub loc: Loc,
    pub name: String,
    pub inline: bool,
    pub tokens: Option<Vec<Operator>>
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub loc: Loc,
    pub name: String
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub loc: Loc,
    pub id: usize
    
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub loc: Loc,
    pub name: String,
    pub fields: HashSet<(String, Types)>
}

pub type Functions = HashMap<String, Function>;
pub type Memories = HashMap<String, Memory>;
pub type Constants = HashMap<String, Constant>;
pub type StructDefs = HashMap<String, StructDef>;

#[derive(Debug, Clone)]
pub struct Program {
    pub ops: Vec<Operator>,
    pub functions: Functions,
    pub memories: Memories,
    pub constants: Constants,
    pub struct_defs: StructDefs
}
