use std::collections::HashMap;

use crate::{constants::{Operator, Types, OpType, KeywordType, InstructionType, Loc}, Args, lerror, warn};
use color_eyre::Result;
use eyre::eyre;

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
                                return Err(eyre!(""));
                            }

                        } else {
                            lerror!(&op.loc, "Expected {:?}, got nothing", OpType::Instruction(InstructionType::With));
                            return Err(eyre!(""));
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
                                        Types::Int
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
                            return Err(eyre!(""))
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
                        constants.insert(op.text, Constant { loc: op.loc.clone(), types: vec![Types::Int] });
                        
                    },
                    KeywordType::FunctionThen |
                    KeywordType::FunctionDone |
                    KeywordType::Inline |
                    KeywordType::Export |
                    KeywordType::Function => {
                        println!("{:?}", op);
                        unreachable!()
                    },
                }
            },
            OpType::Instruction(instruction) => {
                match instruction {
                    InstructionType::PushInt => {
                        stack.push(Types::Int);
                    },
                    InstructionType::PushStr => {
                        stack.push(Types::Int);
                        stack.push(Types::Ptr);
                    },
                    InstructionType::PushCStr => {
                        stack.push(Types::Int);
                        stack.push(Types::Ptr);
                    },
                    InstructionType::Drop => {
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                    },
                    InstructionType::Print => {
                        stack_pop(&mut stack, &op, &[Types::Int])?;
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
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack.push(Types::Int);
                    },
                    InstructionType::Equals |
                    InstructionType::Gt |
                    InstructionType::Lt |
                    InstructionType::Ge |
                    InstructionType::Le |
                    InstructionType::NotEquals => {
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack.push(Types::Bool);
                    },
                    InstructionType::DivMod => {
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack.push(Types::Int);
                        stack.push(Types::Int);
                    },
                    InstructionType::Load8 |
                    InstructionType::Load32 |
                    InstructionType::Load64 => {
                        stack_pop(&mut stack, &op, &[Types::Ptr])?;
                        stack.push(Types::Int);
                    },
                    InstructionType::Store8 |
                    InstructionType::Store32 |
                    InstructionType::Store64 => {
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack_pop(&mut stack, &op, &[Types::Ptr])?;
                    },
                    InstructionType::Syscall0 => {
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack.push(Types::Int);
                    },
                    InstructionType::Syscall1 => {
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::Int);
                    },
                    InstructionType::Syscall2 => {
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::Int);
                    },
                    InstructionType::Syscall3 => {
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::Int);
                    },
                    InstructionType::Syscall4 => {
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::Int);
                    },
                    InstructionType::Syscall5 => {
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::Int);
                    },
                    InstructionType::Syscall6 => {
                        stack_pop(&mut stack, &op, &[Types::Int])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack_pop(&mut stack, &op, &[Types::Any])?;
                        stack.push(Types::Int);
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
                        stack.push(Types::Int);
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
                            return Err(eyre!(""));
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
                                    return Err(eyre!(""));
                                }
                            } else {
                                lerror!(&op.loc, "Expected {:?}, but got nothing", t);
                                return Err(eyre!(""));
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
                }
            },
            
        }

        
    }
    
    Ok((stack, functions, constants))
}



fn stack_pop(v: &mut Vec<Types>, op: &Operator, t: &[Types]) -> Result<Types> {
    if v.is_empty() {
        lerror!(&op.loc, "Expected {:?}, but got nothing", t);
        return Err(eyre!(""));
    }
    let r = v.pop().unwrap();

    if !t.contains(&r) && t[0] != Types::Any {
        lerror!(&op.loc, "Expected {:?}, but got {:?}", t, r);
        return Err(eyre!(""));
    }

    Ok(r)
}
