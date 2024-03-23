use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::types::{ast::{AstNode, Constant, Module, Program}, common::Loc};


lazy_static!(
    static ref DEFAULT_CONSTANTS: HashMap<&'static str, AstNode> = {
        let mut h = HashMap::new();
        // No bsd cause im not about to create 3 or 4 diffrent compilation targets
        h.insert("__WINDOWS", AstNode::Int(Loc::default(), cfg!(target_os = "windows") as usize));
        h.insert("__LINUX", AstNode::Int(Loc::default(), cfg!(target_os = "linux") as usize));
        h.insert("__ENDIAN_LITTLE", AstNode::Int(Loc::default(), cfg!(target_endian="little") as usize));
        h.insert("__ENDIAN_BIG", AstNode::Int(Loc::default(), cfg!(target_endian="big") as usize));
        

        h
    };
);



pub fn get_builtin_symbols(prog: &mut Program) -> AstNode {
    let mut md = Module {
        loc: Loc::new(String::from("BUILTIN"), 0, 0),
        path: vec![String::from("builtin")],
        ident: String::from("BUILTIN"),
        body: Vec::new(),
    };



    for (k, v) in DEFAULT_CONSTANTS.iter() {
        let c = Constant {
            loc: Loc::default(),
            ident: k.to_string(),
            value: Box::from(v.clone()),
        };
        prog.constants.insert(k.to_string(), c.clone());
        md.body.push(AstNode::Constant(c));
    }


    AstNode::Module(md)
}