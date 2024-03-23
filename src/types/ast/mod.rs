use std::collections::HashMap;

use super::{common::Loc, token::{Token, TypeType}};


//TODO: Implement missing stuff
#[derive(Debug, Clone)]
pub enum AstNode {
    Int(Loc, usize),
    Str(Loc, String),
    CStr(Loc, String),
    Char(Loc, char),
    // ExternFnDef {
    //     loc: Loc,
    //     ident: String,
    //     arg_types: Vec<TokenType>,
    //     ret_type: TokenType,
    // },
    Function(Function),
    Constant(Constant),
    // ExternConstantDef{
    //     loc: Loc,
    //     ident: String,
    //     value: InstructionType
    // },
    // StructDef{
    //     loc: Loc,
    //     extrn: bool,
    //     ident: String,
    //     body: Vec<(String, usize)> // (field ident, size in bytes)
    // },
    StructDef(StructDef),
    StructDispPush{
        loc: Loc,
        disp: usize,
        ident: String,
    },
    // StructItemPush{
    //     loc: Loc,
    //     disp: usize,
    //     ident: String,
    // },
    If(If),
    While(While),
    Module(Module),
    Memory(Memory),
    MemUse(MemUse),
    ConstUse(ConstUse),
    FnCall(FnCall),
    Block(Block),
    Token(Token),
}

impl AstNode {
    pub fn loc(&self) -> Loc {
        match self {
            AstNode::Function(f) => f.loc.clone(),
            AstNode::Constant(c) => c.loc.clone(),
            AstNode::If(t)=> t.loc.clone(),
            AstNode::While(t)=> t.loc.clone(),
            AstNode::Module(m) => m.loc.clone(),
            AstNode::Memory(m) => m.loc.clone(),
            AstNode::MemUse(t)=> t.loc.clone(),
            AstNode::ConstUse(t)=> t.loc.clone(),
            AstNode::FnCall(t)=> t.loc.clone(),
            AstNode::Block(t)=> t.loc.clone(),
            AstNode::Token(tok) => tok.loc.clone(),
            AstNode::Int(loc, _) => loc.clone(),
            AstNode::Str(loc, _) => loc.clone(),
            AstNode::CStr(loc, _) => loc.clone(),
            AstNode::Char(loc, _) => loc.clone(),
            AstNode::StructDef(s) => s.loc.clone(),
            AstNode::StructDispPush { loc, ..} => loc.clone(),
            // AstNode::StructItemPush { loc, .. } => loc.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub loc: Loc,
    pub ident: String,
    pub body: Vec<(String, usize, TypeType)>, // (field ident, size in bytes)
    pub size: usize
}
#[derive(Debug, Clone)]
pub struct MemUse {
    pub loc: Loc,
    pub ident: String,
    pub disp: Option<usize>
}
#[derive(Debug, Clone)]
pub struct ConstUse {
    pub loc: Loc,
    pub ident: String,
}
#[derive(Debug, Clone)]
pub struct FnCall  {
    pub loc: Loc,
    pub ident: String,
}
#[derive(Debug, Clone)]
pub struct Block {
    pub comment: String,
    pub loc: Loc,
    pub body: Vec<AstNode>
}

#[derive(Debug, Clone)]
pub struct While {
    pub loc: Loc,
    pub test: Vec<AstNode>,
    pub body: Vec<AstNode>,
}

#[derive(Debug, Clone)]
pub struct If {
    pub loc: Loc,
    pub test: Vec<AstNode>,
    pub body: Vec<AstNode>,
    pub els: Box<AstNode>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub loc: Loc,
    pub path: Vec<String>,
    pub ident: String,
    pub body: Vec<AstNode>
}

#[derive(Debug, Clone)]
pub struct Function {
    pub loc: Loc,
    pub ident: String,
    pub inline: bool,
    pub extrn: bool,
    pub export: bool,
    pub arg_types: Vec<TypeType>,
    pub ret_types: Vec<TypeType>,
    pub body: Vec<AstNode>
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub loc: Loc,
    pub ident: String,
    pub value: Box<AstNode>
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub loc: Loc,
    pub ident: String,
    pub statc: bool,
    pub size: MemSize // bytes
}


#[derive(Debug, Clone)]
pub struct Program {
    pub ast: AstNode,
    pub functions: HashMap<String, Function>,
    pub constants: HashMap<String, Constant>,
    pub memories: HashMap<String, Memory>,
    pub struct_defs: HashMap<String, StructDef>,
}

#[derive(Debug, Clone)]
pub enum MemSize {
    Size(usize),
    Type(TypeType)
}

impl EscIdent for FnCall {
    fn ident(&self) -> String {
        self.ident.clone()
    }
}

impl EscIdent for ConstUse {
    fn ident(&self) -> String {
        self.ident.clone()
    }
}

impl EscIdent for MemUse {
    fn ident(&self) -> String {
        self.ident.clone()
    }
}

impl EscIdent for Constant {
    fn ident(&self) -> String {
        self.ident.clone()
    }
}
impl EscIdent for Memory {
    fn ident(&self) -> String {
        self.ident.clone()
    }
}
impl EscIdent for Function {
    fn ident(&self) -> String {
        self.ident.clone()
    }
}

pub trait EscIdent {
    fn ident(&self) -> String;
    fn get_ident_escaped(&self) -> String {
        self.ident().replace("(", "_OPRN_")
            .replace(")", "_CPRN_")
    }
}