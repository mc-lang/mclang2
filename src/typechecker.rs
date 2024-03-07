use std::collections::HashMap;

use crate::{definitions::{Operator, Types, OpType, KeywordType, InstructionType, Loc}, Args, lerror, warn};
use anyhow::{Result, bail};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Function {
    loc: Loc,
    args: Vec<Types>,
    returns: Vec<Types>,
}

#[derive(Debug, Clone)]
pub struct Constant {
    #[allow(dead_code)]
    loc: Loc,
    types: Vec<Types>,
}

impl Function {
    #[allow(dead_code)]
    pub fn default() -> Self {
        Self {
            args: Vec::new(),
            returns: Vec::new(),
            loc: (String::new(), 0, 0)
        }
    }
}

type Functions = HashMap<String, Function>;
type Constants = HashMap<String, Constant>;

pub fn typecheck(ops: Vec<Operator>, args: &Args, init_types: Option<Vec<Types>>,  funcs: HashMap<String, Function>, consts:  HashMap<String, Constant>) -> Result<(Vec<Types>, Functions, Constants)>{
    if args.unsaf {
        if !args.quiet {
            warn!("Unsafe mode enabled, disabling typechecker, goodluck");
        }
        return Ok((Vec::new(), HashMap::new(), HashMap::new()));
    }
    
    let mut functions: HashMap<String, Function> = funcs;
    let mut constants: HashMap<String, Constant> = consts;
    // let mut in_function: (String, Function, Loc) = (String::new(), Function::default(), (String::new(), 0, 0));
    let mut stack: Vec<Types> = if let Some(i) = init_types {i} else {Vec::new()};
    let mut stack_snapshots: Vec<Vec<Types>> = Vec::new();
    let mut rtokens = ops;
    rtokens.reverse();
    // println!("{:#?}", ops);
    while !rtokens.is_empty() {
        let op = rtokens.pop().unwrap();
        // println!("{:?}", stack.clone());
        // println!("{:?}", op);
        // println!("{}", ops.len());
        match op.typ.clone() {
            OpType::Keyword(keyword) => {
                match keyword {
                    KeywordType::If |
                    KeywordType::Do => {
                        stack_pop(&mut stack, &op, &[Types::Bool])?;
                    },

                    KeywordType::FunctionDefExported |
                    KeywordType::FunctionDef => {
                        let name = op.text.clone();
                        // println!("{:?}", name);
                        if let Some(p) = rtokens.pop() {
                            if p.typ != OpType::Instruction(InstructionType::With){
                                lerror!(&op.loc, "Expected {:?}, got {:?}", OpType::Instruction(InstructionType::With), p.typ);
                                bail!("");
                            }

                        } else {
                            lerror!(&op.loc, "Expected {:?}, got nothing", OpType::Instruction(InstructionType::With));
                            bail!("");
                        }
                        
                        let mut p = rtokens.pop();
                        let mut func = Function {
                            args: Vec::new(),
                            returns: Vec::new(),
                            loc: op.loc
                        };
                        let mut return_args = false;
                        while p.as_ref().is_some() {
                            let op = p.as_ref().unwrap();
                            if op.typ == OpType::Instruction(InstructionType::TypeBool) ||
                                op.typ == OpType::Instruction(InstructionType::TypeInt) ||
                                op.typ == OpType::Instruction(InstructionType::TypePtr) ||
                                op.typ == OpType::Instruction(InstructionType::TypeAny) ||
                                op.typ == OpType::Instruction(InstructionType::TypeVoid) {
                                    let t = if op.typ == OpType::Instruction(InstructionType::TypeInt) {
                                        Types::U64
                                    } else if op.typ == OpType::Instruction(InstructionType::TypeBool) {
                                        Types::Bool
                                    } else if op.typ == OpType::Instruction(InstructionType::TypePtr) {
                                        Types::Ptr
                                    } else if op.typ == OpType::Instruction(InstructionType::TypeVoid) {
                                        if return_args {
                                            func.returns = vec![Types::Void];
                                        } else {
                                            func.args = vec![Types::Void];
                                            return_args = true;
                                            continue;
                                        }
                                        Types::Void
                                    } else if op.typ == OpType::Instruction(InstructionType::TypeAny) {
                                        Types::Any
                                    } else {
                                        panic!()
                                    };

                                    if return_args {
                                        func.returns.push(t);
                                    } else {
                                        func.args.push(t);
                                    }
                            }
                                
                            if op.typ == OpType::Instruction(InstructionType::Returns) {
                                return_args = true;
                            }

                            if op.typ == OpType::Keyword(KeywordType::FunctionThen) {
                                break;
                            }
                            p = rtokens.pop();
                        };


                        let mut code: Vec<Operator> = Vec::new();

                        while !rtokens.is_empty() {
                            let op = rtokens.pop().unwrap();

                            if op.typ == OpType::Keyword(KeywordType::FunctionDone) {
                                break;
                            }
                            code.push(op);
                        }
                        let ts = if func.args.clone() == vec![Types::Void] {
                            Vec::new()
                        } else {
                            func.args.clone()
                        };

                        if ts.contains(&Types::Void) {
                            continue;
                        }
                        functions.insert(name.clone(), func.clone());
                        let (ret_typs, _, _) = typecheck(code, args, Some(ts.clone()), functions.clone(), constants.clone())?;
                        if ret_typs != func.returns && !func.returns.contains(&Types::Void){
                            lerror!(&func.loc, "Expected {:?}, but got {:?}", func.returns, ret_typs);
                            bail!("");
                        }

                        if !func.args.contains(&Types::Void) {
                            stack.append(&mut func.args);
                        }
                        stack_snapshots.push(stack.clone());
                    }

                    KeywordType::Else |
                    KeywordType::End |
                    KeywordType::While |
                    KeywordType::Include |
                    KeywordType::Constant |
                    KeywordType::Memory => (),
                    KeywordType::ConstantDef => {
                        // println!("defined constant");
                        constants.insert(op.text, Constant { loc: op.loc.clone(), types: vec![Types::U64] });
                        
                    },
                    KeywordType::FunctionThen |
                    KeywordType::FunctionDone |
                    KeywordType::Inline |
                    KeywordType::Export |
                    KeywordType::Function => {
                        println!("{:?}", op);
                        unreachable!()
                    },
                    KeywordType::Struct => todo!(),
                }
            },
            OpType::Instruction(instruction) => {
                match instruction {
                    InstructionType::PushInt => {
                        stack.push(Types::U64);
                    },
                    InstructionType::PushStr => {
                        stack.push(Types::U64);
                        stack.push(Types::Ptr);
                    },
                    InstructionType::PushCStr => {
                        stack.push(Types::U64);
                        stack.push(Types::Ptr);
                    },
                    InstructionType::Drop => {
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                    },
                    InstructionType::Print => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                    },
                    InstructionType::Dup => {
                        let a = stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(a);
                    },
                    InstructionType::Rot => {
                        let a = stack_pop(&mut stack, &op, &[Types::Any])?;
                        let b = stack_pop(&mut stack, &op, &[Types::Any])?;
                        let c = stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(b);
                        stack.push(a);
                        stack.push(c);
                    },
                    InstructionType::Over => {
                        let a = stack_pop(&mut stack, &op, &[Types::Any])?;
                        let b = stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(b.clone());
                        stack.push(a);
                        stack.push(b);
                    },
                    InstructionType::Swap => {
                        let a = stack_pop(&mut stack, &op, &[Types::Any])?;
                        let b = stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(a);
                        stack.push(b);
                    },
                    InstructionType::Minus |
                    InstructionType::Plus |
                    InstructionType::Band |
                    InstructionType::Bor |
                    InstructionType::Shr |
                    InstructionType::Shl |
                    InstructionType::Mul => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack.push(Types::U64);
                    },
                    InstructionType::Equals |
                    InstructionType::Gt |
                    InstructionType::Lt |
                    InstructionType::Ge |
                    InstructionType::Le |
                    InstructionType::NotEquals => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack.push(Types::Bool);
                    },
                    InstructionType::DivMod => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack.push(Types::U64);
                        stack.push(Types::U64);
                    },
                    InstructionType::Read8 |
                    InstructionType::Read32 |
                    InstructionType::Read64 => {
                        stack_pop(&mut stack, &op, &[Types::Ptr])?;
                        stack.push(Types::U64);
                    },
                    InstructionType::Write8 |
                    InstructionType::Write32 |
                    InstructionType::Write64 => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack_pop(&mut stack, &op, &[Types::Ptr])?;
                    },
                    InstructionType::Syscall0 => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack.push(Types::U64);
                    },
                    InstructionType::Syscall1 => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::U64);
                    },
                    InstructionType::Syscall2 => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::U64);
                    },
                    InstructionType::Syscall3 => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::U64);
                    },
                    InstructionType::Syscall4 => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::U64);
                    },
                    InstructionType::Syscall5 => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::U64);
                    },
                    InstructionType::Syscall6 => {
                        stack_pop(&mut stack, &op, &[Types::U64])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::U64);
                    },
                    InstructionType::CastBool => {
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::Bool);
                    },
                    InstructionType::CastPtr => {
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::Ptr);
                    },
                    InstructionType::CastInt => {
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::U64);
                    },
                    InstructionType::CastVoid => {
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::Any);
                    },
                    InstructionType::MemUse => {
                        stack.push(Types::Ptr);
                    },
                    InstructionType::FnCall  => {
                        stack_snapshots.push(stack.clone());
                        
                        let f = if let Some(f) = functions.get(&op.text) {f} else {
                            lerror!(&op.loc, "Could not find function {}", op.text);
                            bail!("");
                        };

                        // in_function = (op.text.clone(), f.clone(), op.loc.clone());

                        let mut s = stack.clone();
                        let mut a = f.args.clone();
                        // s.reverse();
                        a.reverse();

                        for t in a{
                            if let Some(s2) = s.pop(){
                                if t != s2 {
                                    lerror!(&op.loc, "Expected {:?}, but got {:?}", t, s2);
                                    bail!("");
                                }
                            } else {
                                lerror!(&op.loc, "Expected {:?}, but got nothing", t);
                                bail!("");
                            }
                        }

                        
                    }
                    InstructionType::Return |
                    InstructionType::None |
                    InstructionType::TypeBool |
                    InstructionType::TypePtr |
                    InstructionType::TypeInt |
                    InstructionType::TypeVoid |
                    InstructionType::TypeAny |
                    InstructionType::Returns |
                    InstructionType::With => (),
                    InstructionType::ConstUse => {
                        // println!("{constants:?}");
                        let mut c = constants.get(&op.text).unwrap().clone();
                        stack.append(&mut c.types);
                    },
                    InstructionType::StructUse => todo!(),
                }
            },
            OpType::Internal(t) => panic!("{t:?}"),
            
        }

        
    }
    
    Ok((stack, functions, constants))
}



fn stack_pop(v: &mut Vec<Types>, op: &Operator, t: &[Types]) -> Result<Types> {
    if v.is_empty() {
        lerror!(&op.loc, "Expected {:?}, but got nothing", t);
        bail!("");
    }
    let r = v.pop().unwrap();

    if !t.contains(&r) && t[0] != Types::Any {
        lerror!(&op.loc, "Expected {:?}, but got {:?}", t, r);
        bail!("");
    }

    Ok(r)
}
