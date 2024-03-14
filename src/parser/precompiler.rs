use anyhow::bail;

use crate::types::{ast::{AstNode, Program}, common::Loc, token::{InstructionType, TokenType}};


pub fn precompile_mem(prog: &Program, ast: Vec<AstNode> ) -> anyhow::Result<usize> {
    match precompile_const(prog, ast, &mut Vec::new()) {
        Ok(v) => {
            match v {
                AstNode::Int(_, i) => {
                    return Ok(i)
                }
                _ => {
                    error!("memories can only have numbers or types in their size");
                    bail!("")
                }
            }
        },
        Err(e) => bail!(e),
    }
}
pub fn precompile_const(prog: &Program, ast: Vec<AstNode>, stack: &mut Vec<usize> ) -> anyhow::Result<AstNode> {
    for node in ast.clone() {
        match &node {
            AstNode::ConstUse(c) => {
                let Some(val) = prog.constants.get(&c.ident) else {
                    error!({loc => c.loc.clone()}, "Unknown constant {:?}", c.ident) ;
                    bail!("")
                };
                match Box::leak(val.value.clone()) {
                    t @ AstNode::Int(..) => {
                        return Ok(t.clone());
                    }

                    t @ AstNode::Str(..) => {
                        return Ok(t.clone());
                    }

                    t @ AstNode::CStr(..) => {
                        return Ok(t.clone());
                    }

                    t @ AstNode::Char(..) => {
                        return Ok(t.clone());
                    }


                    // AstNode::Token(t) => {
                    //     match t.typ.clone() {
                    //         TokenType::Instruction(it) => {
                    //                 match it {
                    //                     InstructionType::PushInt(i) => stack.push(i),
                    //                     InstructionType::PushCStr(_) => {
                    //                         //TODO: Handle this better
                    //                         return Ok(AstNode::Token(t.clone()));
                    //                     },
                    //                     InstructionType::PushChar(_) => {
                    //                         //TODO: Handle this better
                    //                         return Ok(AstNode::Token(t.clone()));
                    //                     },
                    //                     _ => panic!()
                    //                 }
                    //         },
                    //         _ => panic!()
                    //     }
                    // },
                    _ => panic!()
                }

            },
            AstNode::Token(t) => {
                match t.typ.clone() {
                    TokenType::Keyword(_) => {
                        error!({loc => t.loc.clone()}, "Unsupported token {t:?}, we dont support precompilation of this") ;
                        bail!("")
                    },
                    TokenType::Instruction(it) => {
                        match it {
                            InstructionType::PushInt(i) => {
                                stack.push(i);
                            },
                            InstructionType::PushCStr(s) => {
                                //TODO: Handle this better
                                return Ok(AstNode::CStr(t.loc.clone(), s));
                            },
                            InstructionType::PushStr(s) => {
                                //TODO: Handle this better
                                return Ok(AstNode::Str(t.loc.clone(), s));
                            },
                            InstructionType::PushChar(c) => {
                                //TODO: Handle this better
                                return Ok(AstNode::Char(t.loc.clone(), c));
                            },
                            InstructionType::Minus => {
                                let a = stack_pop(stack, &t.loc)?;
                                let b = stack_pop(stack, &t.loc)?;
                                stack.push(b - a);
                            },
                            InstructionType::Plus => {
                                let a = stack_pop(stack, &t.loc)?;
                                let b = stack_pop(stack, &t.loc)?;
                                stack.push(b + a);
                            },
                            InstructionType::DivMod => {
                                let a = stack_pop(stack, &t.loc)?;
                                let b = stack_pop(stack, &t.loc)?;
                                stack.push(b / a);
                                stack.push(b % a);
                            },
                            InstructionType::Mul => {
                                let a = stack_pop(stack, &t.loc)?;
                                let b = stack_pop(stack, &t.loc)?;
                                stack.push(b * a);
                            },
                            InstructionType::Drop => {
                                stack_pop(stack, &t.loc)?;
                            },
                            //TODO: Support these later
                            // InstructionType::Dup => todo!(),
                            // InstructionType::Rot => todo!(),
                            // InstructionType::Over => todo!(),
                            // InstructionType::Swap => todo!(),
                            // InstructionType::Equals => todo!(),
                            // InstructionType::Gt => todo!(),
                            // InstructionType::Lt => todo!(),
                            // InstructionType::Ge => todo!(),
                            // InstructionType::Le => todo!(),
                            // InstructionType::NotEquals => todo!(),
                            // InstructionType::Band => todo!(),
                            // InstructionType::Bor => todo!(),
                            // InstructionType::Shr => todo!(),
                            // InstructionType::Shl => todo!(),
                            //TODO: Support this when we have types
                            // InstructionType::CastBool => todo!(),
                            // InstructionType::CastPtr => todo!(),
                            // InstructionType::CastInt => todo!(),
                            // InstructionType::CastVoid => todo!(),
                            InstructionType::ConstUse => unreachable!(),
                            _ => {
                                error!({loc => t.loc.clone()}, "Unsupported token {t:?}, we dont support precompilation of this") ;
                                bail!("")
                            }
                        }
                    },
                    TokenType::Unknown(_) => todo!(),
                }   
            },
            //TODO: Implement these
            t @ AstNode::If { .. } | 
            t @ AstNode::While { .. } |
            t => {
                error!({loc => t.loc()}, "Unsupported token {t:?}, we dont support precompilation of this") ;
                bail!("")
            }
        }

    }

    Ok(AstNode::Int(ast[0].loc(), stack[0]))
}

fn stack_pop(stack: &mut Vec<usize>, loc: &Loc) -> anyhow::Result<usize> {
    match stack.pop() {
        Some(i) => Ok(i),
        None => {
            error!({loc => loc.clone()}, "Failed to precompile tokens, failed to pop from stack");
            bail!("")
        },
    }
}