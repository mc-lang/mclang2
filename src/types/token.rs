use super::common::Loc;




struct Token {
    loc: Loc,
    typ: TokenType,
    val: TokenValue
}

pub enum TokenValue {
    Int(usize),
    Str(String)
}

pub enum TokenType {
    DbgPrint,
    Keyword(TokenKeyword),
    Syscall(u8),

    //? Literal
    PushInt,
    PushStr,

    //? Stack manipulation
    Dup,
    Rot, // a b c => b c a
    Over, // a b => a b a
    Swap, // a b => b a

    //? Math
    Plus,
    Minus,
    Mul,
    Div,
    Mod,

    //? Logical
    And,
    Or,
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
    Ne,

    //? Bitwise
    Shr,
    Shl,
    Bor,
    Band,
}

pub enum TokenKeyword {
    Function,
    If,
    Else,
    End,
    Done,
    Macro,
    While,
    Do
}