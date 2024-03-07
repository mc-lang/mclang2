use std::collections::HashMap;

use crate::{constants::{OpType, Loc, InstructionType, KeywordType, Operator}, lerror, error};
// use crate::util::logger;
use color_eyre::Result;
use eyre::eyre;

use super::{Memory, Function, Constant};
mod syscalls;

fn stack_pop(stack: &mut Vec<usize>, pos: &Loc) -> Result<usize> {
    if let Some(i) = stack.pop() { Ok(i) } else {
        lerror!(&pos.clone(), "Stack underflow");
        Err(eyre!("Stack underflow"))
    }
}

pub fn run(ops: &[crate::constants::Operator]) -> Result<i32>{
    let mut stack: Vec<usize> = Vec::new();
    let mut mem: Vec<u64> = vec![0; crate::MEM_SZ + crate::STRING_SZ];
    let mut string_idx = 0;
    
    let prerunned = pre_run(ops);
    let functions = prerunned.functions;
    let constants = prerunned.constants;
    let memories = prerunned.memories;

    let mut ret_stack: Vec<usize> = Vec::new();
    
    // for token in &tokens {
    //     println!("{{typ: \"{:?}\", val: {}, jmp: {}}}", token.typ, token.value, token.jmp);
    // }
        
    // jump to main func    
    let mut ip = if let Some(i) = functions.get("main") {i.id} else {
        crate::errors::missing_main_fn();
        return Err(eyre!(""));
    };
    
    while ip < ops.len() {
        let op = &ops[ip];
        let pos = op.loc.clone();
        match op.typ.clone() {
            OpType::Instruction(instruction) => {
                match instruction {
                    InstructionType::PushInt => {
                        stack.push(op.value);
                        ip += 1;
                    },
                    InstructionType::PushStr => {
                        if op.addr.is_none() {
                            stack.push(op.text.len()); // string len
                            stack.push(string_idx + crate::MEM_SZ);
                            
                            for c in op.text.bytes() {
                                mem[crate::MEM_SZ + string_idx] = u64::from(c);
                                string_idx += 1;
                            }
                        } else {
                            stack.push(op.text.len()); 
                            if let Some(addr) = op.addr {
                                stack.push(addr);
                            }
                        }
        
        
                        ip += 1;
                    },
                    InstructionType::PushCStr => {
                        if op.addr.is_none() {
                            stack.push(string_idx + crate::MEM_SZ);
                            
                            for c in op.text.bytes() {
                                mem[crate::MEM_SZ + string_idx] = u64::from(c);
                                string_idx += 1;
                            }
                        } else {
                            if let Some(addr) = op.addr {
                                stack.push(addr);
                            }
                        }
        
        
                        ip += 1;
                    },
                    InstructionType::Drop => {
                        stack.pop();
                        ip += 1;
                    },
                    InstructionType::Dup => {
                        let a = stack_pop(&mut stack, &pos)?;
                        stack.push(a);
                        stack.push(a);
                        ip += 1;
                    },
        
                    InstructionType::Rot => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        let c = stack_pop(&mut stack, &pos)?;
                        stack.push(b);
                        stack.push(a);
                        stack.push(c);
                        ip += 1;
                    }
                    InstructionType::Swap => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(a);
                        stack.push(b);
                        ip += 1;
                    }
                    InstructionType::Over => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(b);
                        stack.push(a);
                        stack.push(b);
                        ip += 1;
                    }
        
                    InstructionType::Print => {
                        let a = stack_pop(&mut stack, &pos)?;
                        println!("{a}");
                        // let _ = io::stdout().flush();
                        ip += 1;
                    },
                    #[allow(clippy::cast_possible_truncation)]
                    InstructionType::Read8 |
                    InstructionType::Read32 |
                    InstructionType::Read64 => {
                        let a = stack_pop(&mut stack, &pos)?;
                        if a > crate::MEM_SZ + crate::STRING_SZ {
                            lerror!(&op.loc, "Invalid memory address {a}");
                            return Ok(1);
                        }
                        let byte = mem[a];
                        stack.push(byte as usize);
                        ip += 1;
                    }
                    #[allow(clippy::cast_possible_truncation)]
                    InstructionType::Write8 => {
                        let val = stack_pop(&mut stack, &pos)?;
                        let addr = stack_pop(&mut stack, &pos)?;
                        
                        if addr > crate::MEM_SZ {
                            lerror!(&op.loc, "Invalid memory address {addr}");
                            return Ok(1);
                        }

                        mem[addr] = u64::from(val as u8);
                        ip += 1;
                    }
                    #[allow(clippy::cast_possible_truncation)]
                    InstructionType::Write32 => {
                        let val = stack_pop(&mut stack, &pos)?;
                        let addr = stack_pop(&mut stack, &pos)?;
                        
                        if addr > crate::MEM_SZ {
                            lerror!(&op.loc, "Invalid memory address {addr}");
                            return Ok(1);
                        }

                        mem[addr] = u64::from(val as u32);
                        ip += 1;
                    }

                    #[allow(clippy::cast_possible_truncation)]
                    InstructionType::Write64 => {
                        let val = stack_pop(&mut stack, &pos)?;
                        let addr = stack_pop(&mut stack, &pos)?;
                        
                        if addr > crate::MEM_SZ {
                            lerror!(&op.loc, "Invalid memory address {addr}");
                            return Ok(1);
                        }

                        mem[addr] = val as u64;
                        ip += 1;
                    }
        
                    // math
                    InstructionType::Plus => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(b + a);
                        ip += 1;
                    },
                    InstructionType::Minus => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(b - a);
                        ip += 1;
                    },
                    InstructionType::Equals => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(usize::from(b == a));
                        ip += 1;
                    },
                    InstructionType::Gt => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(usize::from(b > a));
                        ip += 1;
                    },
                    InstructionType::Lt => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(usize::from(b < a));
                        ip += 1;
                    },
                    InstructionType::NotEquals => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(usize::from(b != a));
                        ip += 1;
                    },
                    InstructionType::Ge => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(usize::from(b >= a));
                        ip += 1;
                    },
                    InstructionType::Le => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(usize::from(b <= a));
                        ip += 1;
                    },
        
                    InstructionType::Band => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(a & b);
                        ip += 1;
                    }
        
                    InstructionType::Bor => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(a | b);
                        ip += 1;
                    }
        
                    InstructionType::Shr => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(b >> a);
                        ip += 1;
                    }
        
                    InstructionType::Shl => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(b << a);
                        ip += 1;
                    }
                    
                    InstructionType::DivMod => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(b / a);
                        stack.push(b % a);
                        ip += 1;
                    }
                    InstructionType::Mul => {
                        let a = stack_pop(&mut stack, &pos)?;
                        let b = stack_pop(&mut stack, &pos)?;
                        stack.push(b * a);
                        ip += 1;
                    }
                    InstructionType::Syscall0 => {
                        todo!();
                        // ti += 1;
                    },
                    InstructionType::Syscall1 => {
                        todo!();
                        // ti += 1;
                    },
                    InstructionType::Syscall2 => {
                        todo!();
                        // ti += 1;
                    },
                    InstructionType::Syscall3 => {
                        let rax = stack_pop(&mut stack, &pos)?;
                        let rdi = stack_pop(&mut stack, &pos)?;
                        let rsi = stack_pop(&mut stack, &pos)?;
                        let rdx = stack_pop(&mut stack, &pos)?;
                        // println!("yes");
                        let ret = match rax {
                            1 => syscalls::sys_write(rax, rdi, rsi, rdx, &mem),
                            0 => 0, //? temp, so clippy doesnt complain
                            _ => {
                                error!("Syscall(3) #{} is not implemented", rax);
                                return Err(eyre!("Syscall not implemented"));
                            }
                        };
                        stack.push(ret);
                        // println!("{}", stack.len());
                        ip += 1;
                    },
                    InstructionType::Syscall4 => {
                        todo!();
                        // ti += 1;
                    },
                    InstructionType::Syscall5 => {
                        todo!();
                        // ti += 1;
                    },
                    InstructionType::Syscall6 => {
                        todo!();
                        // ti += 1;
                    },
                    InstructionType::MemUse => {

                        let m = memories.get(&op.addr.unwrap()).unwrap();
                        stack.push(m.id);
                        ip += 1;
                    },
                    InstructionType::FnCall => {
                        ret_stack.push(ip);
                        let f = functions.get(&op.text).unwrap();
                        ip = f.id;
                    }
                    InstructionType::Return => {
                        ip = ret_stack.pop().unwrap();
                        ip += 1;
                    }
                    InstructionType::ConstUse => {
                        let a = constants.get(&op.text).unwrap();
                        
                        if let Some(i) = a.value_i {
                            stack.push(i);
                        } else if let Some(_s) = a.value_s.clone() {
                            unimplemented!();
                        }
                        ip += 1;
                    },
                    InstructionType::CastBool |
                    InstructionType::CastPtr |
                    InstructionType::CastInt |
                    InstructionType::CastVoid |
                    InstructionType::TypeBool |
                    InstructionType::TypePtr |
                    InstructionType::TypeInt |
                    InstructionType::TypeVoid |
                    InstructionType::TypeAny |
                    InstructionType::Returns |
                    InstructionType::With => ip += 1,
                    InstructionType::None => unreachable!(),
                }

            }
            OpType::Keyword(k) => {
                match k {
                    // blocks
                    KeywordType::If => {
                        let a = stack_pop(&mut stack, &pos)?;
                        if a == 0 {
                            // println!("If({ti}) => t: {:?} j: {}", tokens[token.jmp as usize].typ, token.jmp);
                            ip = op.jmp;
                        } else {
                            ip += 1;
                        }
                    },
                    KeywordType::Else | KeywordType::End => {
                        ip = op.jmp;
                    }
                    KeywordType::Do => {
                        let a = stack.pop().unwrap();
                        if a == 0 {
                            ip = op.jmp;
                        } else {
                            ip += 1;
                        }
                    }
                    KeywordType::While | //* exept this one, this one should just skip over
                    KeywordType::Memory |
                    KeywordType::FunctionDef |
                    KeywordType::FunctionDefExported |
                    KeywordType::ConstantDef => {
                        //? Disabled since we now pre run the whole program
                        // constants.insert(op.text.clone(), Constant { loc: op.loc.clone(), name: op.text.clone(), value_i: Some(op.value), value_s: None, used: false });
                        ip += 1;
                    },
                    KeywordType::FunctionDone => {
                        if let Some(i) = ret_stack.pop() {
                            ip = i + 1;
                        } else {
                            break;
                        }
                    },
                    
                    KeywordType::FunctionThen  => ip += 1,
                    KeywordType::Constant |
                    KeywordType::Function |
                    KeywordType::Inline |
                    KeywordType::Export |
                    KeywordType::Include => unreachable!(),
                }
            }
            
        }
    }
    

    Ok(0)
}

pub struct Defineds {
    pub memories: HashMap<usize, Memory>,
    pub functions: HashMap<String, Function>,
    pub constants: HashMap<String, Constant>
}

pub fn pre_run(ops: &[Operator]) -> Defineds {
    let mut defineds = Defineds{
        memories: HashMap::new(),
        functions: HashMap::new(),
        constants: HashMap::new(),
    };
    for (ip, op) in ops.iter().enumerate() {

        match op.typ {
            OpType::Keyword(KeywordType::Memory) => {
                defineds.memories.insert(op.addr.unwrap(), Memory { size: op.value, loc: op.loc.clone(), id: op.addr.unwrap() });
            },
            OpType::Keyword(KeywordType::FunctionDefExported) => {

            }
            OpType::Keyword(KeywordType::FunctionDef) => {
                defineds.functions.insert(op.text.clone(), Function { loc: op.loc.clone(), name: op.text.clone(), id: ip });
            },
            OpType::Keyword(KeywordType::ConstantDef) => {
                defineds.constants.insert(op.text.clone(), Constant { loc: op.loc.clone(), name: op.text.clone(), value_i: Some(op.value), value_s: None, used: false });
            },
            _ => ()   
        }
    }
    defineds
}