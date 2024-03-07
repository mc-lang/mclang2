
use anyhow::{Result, bail};

use crate::{definitions::{ OpType, InstructionType, Loc, Operator}, lerror};

fn stack_pop(stack: &mut Vec<usize>, loc: &Loc) -> Result<usize> {
    if let Some(i) = stack.pop() { Ok(i) } else {
        lerror!(&loc.clone(), "Stack underflow");
        bail!("Stack underflow")
    }
}

pub fn precompile(tokens: &Vec<Operator>) -> Result<Vec<usize>>{

    let mut stack: Vec<usize> = Vec::new();
    for token in tokens.iter() {
        match token.typ.clone() {
            OpType::Instruction(i) => {
                let loc = token.loc.clone();
                match i {
                    InstructionType::PushInt => {
                        stack.push(token.value);
                    },
                    InstructionType::Plus => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(b + a);
                    },
                    InstructionType::Minus => {
                    let a = stack_pop(&mut stack, &loc)?;
                    let b = stack_pop(&mut stack, &loc)?;
                    stack.push(b - a);
                    },
                    InstructionType::Equals => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(usize::from(b == a));
                    },
                    InstructionType::Gt => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(usize::from(b > a));
                    },
                    InstructionType::Lt => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(usize::from(b < a));
                    },
                    InstructionType::NotEquals => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(usize::from(b != a));
                    },
                    InstructionType::Ge => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(usize::from(b >= a));
                    },
                    InstructionType::Le => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(usize::from(b <= a));
                    },
                    
                    InstructionType::Band => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(a & b);
                    }
                    
                    InstructionType::Bor => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(a | b);
                    }
                    
                    InstructionType::Shr => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(b >> a);
                    }
                    
                    InstructionType::Shl => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(b << a);
                    }
                    
                    InstructionType::DivMod => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(b / a);
                        stack.push(b % a);
                    }
                    InstructionType::Mul => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(b * a);
                    }
                    InstructionType::Drop => {
                        stack.pop();
                    },
                    InstructionType::Dup => {
                        let a = stack_pop(&mut stack, &loc)?;
                        stack.push(a);
                        stack.push(a);
                    },
        
                    InstructionType::Rot => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        let c = stack_pop(&mut stack, &loc)?;
                        stack.push(b);
                        stack.push(a);
                        stack.push(c);
                    }
                    InstructionType::Swap => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(a);
                        stack.push(b);
                    }
                    InstructionType::Over => {
                        let a = stack_pop(&mut stack, &loc)?;
                        let b = stack_pop(&mut stack, &loc)?;
                        stack.push(b);
                        stack.push(a);
                        stack.push(b);
                    }
                    _ => {
                        lerror!(&token.loc, "Unsupported precompiler instruction {:?}", i);
                        dbg!(tokens);
                        bail!("");
                    }
                }
            }
            OpType::Keyword(_) => {
                lerror!(&token.loc, "Unsupported precompiler keyword {:?}", token.typ);
                dbg!(tokens);
                bail!("");
            }
            OpType::Internal(t) => panic!("{t:?}"),
        }
    }
    
    Ok(stack)
}