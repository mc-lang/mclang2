use std::{fs, path::PathBuf, io::{Write, BufWriter}, collections::HashMap};
use crate::{constants::{Operator, OpType, KeywordType}, Args, warn, lerror};
use color_eyre::Result;
use crate::compile::commands::linux_x86_64_compile_and_link;
use crate::constants::InstructionType;
use super::{commands::linux_x86_64_run, Constant, Memory, Function};
use eyre::eyre;


pub fn compile(tokens: &[Operator], args: &Args) -> Result<i32>{
    let debug = args.get_opt_level()? < 1;

    let mut of_c = PathBuf::from(&args.out_file);
    let (mut of_o, mut of_a) = if args.out_file == *crate::DEFAULT_OUT_FILE {
        let of_o = PathBuf::from("/tmp/mclang_comp.o");
        let of_a = PathBuf::from("/tmp/mclang_comp.nasm");
        (of_o, of_a)
    } else {
        let of_o = PathBuf::from(&args.out_file);
        let of_a = PathBuf::from(&args.out_file);
        (of_o, of_a)
    };

    of_c.set_extension("");
    of_o.set_extension("o");
    of_a.set_extension("nasm");

    let mut should_push_ret = false;

    let file = fs::File::create(&of_a)?;
    let mut writer = BufWriter::new(&file);
    let mut memories:  Vec<Memory> = Vec::new();
    let mut constants:  HashMap<String, Constant> = HashMap::new();
    let mut functions: Vec<Function> = Vec::new();
    // println!("{}", tokens.len());
    let mut strings: Vec<String> = Vec::new();
    
    writeln!(writer, "BITS 64")?;
    writeln!(writer, "segment .text")?;

    writeln!(writer, "_dbg_print:")?;
    writeln!(writer, "    mov     r9, -3689348814741910323")?;
    writeln!(writer, "    sub     rsp, 40")?;
    writeln!(writer, "    mov     BYTE [rsp+31], 10")?;
    writeln!(writer, "    lea     rcx, [rsp+30]")?;
    writeln!(writer, ".L2:")?;
    writeln!(writer, "    mov     rax, rdi")?;
    writeln!(writer, "    lea     r8, [rsp+32]")?;
    writeln!(writer, "    mul     r9")?;
    writeln!(writer, "    mov     rax, rdi")?;
    writeln!(writer, "    sub     r8, rcx")?;
    writeln!(writer, "    shr     rdx, 3")?;
    writeln!(writer, "    lea     rsi, [rdx+rdx*4]")?;
    writeln!(writer, "    add     rsi, rsi")?;
    writeln!(writer, "    sub     rax, rsi")?;
    writeln!(writer, "    add     eax, 48")?;
    writeln!(writer, "    mov     BYTE [rcx], al")?;
    writeln!(writer, "    mov     rax, rdi")?;
    writeln!(writer, "    mov     rdi, rdx")?;
    writeln!(writer, "    mov     rdx, rcx")?;
    writeln!(writer, "    sub     rcx, 1")?;
    writeln!(writer, "    cmp     rax, 9")?;
    writeln!(writer, "    ja      .L2")?;
    writeln!(writer, "    lea     rax, [rsp+32]")?;
    writeln!(writer, "    mov     edi, 1")?;
    writeln!(writer, "    sub     rdx, rax")?;
    writeln!(writer, "    xor     eax, eax")?;
    writeln!(writer, "    lea     rsi, [rsp+32+rdx]")?;
    writeln!(writer, "    mov     rdx, r8")?;
    writeln!(writer, "    mov     rax, 1")?;
    writeln!(writer, "    syscall")?;
    writeln!(writer, "    add     rsp, 40")?;
    writeln!(writer, "    ret")?;

    if !crate::config::ENABLE_EXPORTED_FUNCTIONS && !args.lib_mode {
        writeln!(writer, "global _start")?;
        writeln!(writer, "_start:")?; 
        writeln!(writer, "    lea rbp, [rel ret_stack]")?;
        writeln!(writer, "    call main")?;
        writeln!(writer, "    jmp end")?;
    }


    let mut ti = 0;
    while ti < tokens.len() {
        let token = &tokens[ti];
        if debug {
            writeln!(writer, "addr_{ti}:")?;
            if token.typ == OpType::Instruction(InstructionType::PushInt) {
                writeln!(writer, "    ;; -- {:?} {}", token.typ, token.value)?;
            } else if token.typ == OpType::Instruction(InstructionType::PushStr) {
                writeln!(writer, "    ;; -- {:?} {}", token.typ, token.text.escape_debug())?;
            } else {
                writeln!(writer, "    ;; -- {:?}", token.typ)?;
            }
        } else {
            if ti > 0 {
                if tokens[ti-1].typ == OpType::Keyword(KeywordType::Else) ||
                tokens[ti-1].typ == OpType::Keyword(KeywordType::End){
                    writeln!(writer, "addr_{ti}:")?;
                }
            }

            if ti + 1 < tokens.len() && tokens[ti+1].typ == OpType::Keyword(KeywordType::End) {
                writeln!(writer, "addr_{ti}:")?;
            }
            
            if let OpType::Keyword(keyword) = &token.typ {
                match keyword {
                    &KeywordType::End |
                    &KeywordType::While => {
                        writeln!(writer, "addr_{ti}:")?;
                    }
                    _ => ()
                }
            }
        }

        match token.typ.clone() {
            // stack

            OpType::Instruction(instruction) => {
                match instruction {
                    InstructionType::PushInt => {
                        writeln!(writer, "    mov rax, {}", token.value)?;
                        writeln!(writer, "    push rax")?;
                        ti += 1;
                    },
                    InstructionType::PushStr => {
                        writeln!(writer, "    mov rax, {}", token.text.len())?;
                        writeln!(writer, "    push rax")?;
                        writeln!(writer, "    mov rax, str_{}", strings.len())?;
                        writeln!(writer, "    push rax")?;
                        strings.push(token.text.clone());
                        ti += 1;
                    }
                    InstructionType::PushCStr => {
                        writeln!(writer, "    push rax")?;
                        writeln!(writer, "    mov rax, str_{}", strings.len())?;
                        writeln!(writer, "    push rax")?;
                        strings.push(token.text.clone());
                        ti += 1;
                    }
                    InstructionType::Drop => {
                        writeln!(writer, "    pop rax")?;
                        ti += 1;
                    },
                    InstructionType::Print => {
                        writeln!(writer, "    pop rdi")?;
                        writeln!(writer, "    call _dbg_print")?;
                        ti += 1;
                    },
        
                    InstructionType::Dup => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    push rax")?;
                        writeln!(writer, "    push rax")?;
        
                        ti += 1;
                    },
        
                    InstructionType::Rot => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    pop rcx")?;
                        writeln!(writer, "    push rbx")?;
                        writeln!(writer, "    push rax")?;
                        writeln!(writer, "    push rcx")?;
        
                        ti += 1;
                    },
                    InstructionType::Swap => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    push rax")?;
                        writeln!(writer, "    push rbx")?;
        
                        ti += 1;
                    },
                    InstructionType::Over => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    push rbx")?;
                        writeln!(writer, "    push rax")?;
                        writeln!(writer, "    push rbx")?;
        
                        ti += 1;
                    },
                    InstructionType::Load8 => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    xor rbx, rbx")?;
                        writeln!(writer, "    mov bl, byte [rax]")?;
                        writeln!(writer, "    push rbx")?;
                        ti += 1;
                    }
        
                    InstructionType::Store8 => {
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    mov byte [rax], bl")?;
                        ti += 1;
                    }
                    InstructionType::Load32 => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    xor rbx, rbx")?;
                        writeln!(writer, "    mov ebx, dword [rax]")?;
                        writeln!(writer, "    push rbx")?;
                        ti += 1;
                    }
        
                    InstructionType::Store32 => {
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    mov dword[rax], ebx")?;
                        ti += 1;
                    }
                    InstructionType::Load64 => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    xor rbx, rbx")?;
                        writeln!(writer, "    mov rbx, qword [rax]")?;
                        writeln!(writer, "    push rbx")?;
                        ti += 1;
                    }
        
                    InstructionType::Store64 => {
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    mov qword [rax], rbx")?;
                        ti += 1;
                    }
        
                    // math
                    InstructionType::Plus => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    add rax, rbx")?;
                        writeln!(writer, "    push rax")?;
                        ti += 1;
                    },
                    InstructionType::Minus => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    sub rbx, rax")?;
                        writeln!(writer, "    push rbx")?;
                        ti += 1;
                    },
                    InstructionType::Equals => {
                        writeln!(writer, "    mov rcx, 0")?;
                        writeln!(writer, "    mov rdx, 1")?;
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    cmp rax, rbx")?;
                        writeln!(writer, "    cmove rcx, rdx")?;
                        writeln!(writer, "    push rcx")?;
                        ti += 1;
                    },
                    InstructionType::Lt => {
                        writeln!(writer, "    mov rcx, 0")?;
                        writeln!(writer, "    mov rdx, 1")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    cmp rax, rbx")?;
                        writeln!(writer, "    cmovl rcx, rdx")?;
                        writeln!(writer, "    push rcx")?;
                        ti += 1;
                    },
                    InstructionType::Gt => {
                        writeln!(writer, "    mov rcx, 0")?;
                        writeln!(writer, "    mov rdx, 1")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    cmp rax, rbx")?;
                        writeln!(writer, "    cmovg rcx, rdx")?;
                        writeln!(writer, "    push rcx")?;
                        ti += 1;
                    },
                    InstructionType::NotEquals => {
                        writeln!(writer, "    mov rcx, 1")?;
                        writeln!(writer, "    mov rdx, 0")?;
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    cmp rax, rbx")?;
                        writeln!(writer, "    cmove rcx, rdx")?;
                        writeln!(writer, "    push rcx")?;
                        ti += 1;
                    },
                    InstructionType::Le => {
                        writeln!(writer, "    mov rcx, 0")?;
                        writeln!(writer, "    mov rdx, 1")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    cmp rax, rbx")?;
                        writeln!(writer, "    cmovle rcx, rdx")?;
                        writeln!(writer, "    push rcx")?;
                        ti += 1;
                    },
                    InstructionType::Ge => {
                        writeln!(writer, "    mov rcx, 0")?;
                        writeln!(writer, "    mov rdx, 1")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    cmp rax, rbx")?;
                        writeln!(writer, "    cmovge rcx, rdx")?;
                        writeln!(writer, "    push rcx")?;
                        ti += 1;
                    },
                    InstructionType::Band => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    and rbx, rax")?;
                        writeln!(writer, "    push rbx")?;
                        ti += 1;
                    },
                    InstructionType::Bor => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    or rbx, rax")?;
                        writeln!(writer, "    push rbx")?;
                        ti += 1;
                    },
                    InstructionType::Shr => {
                        writeln!(writer, "    pop rcx")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    shr rbx, cl")?;
                        writeln!(writer, "    push rbx")?;
                        ti += 1;
                    },
                    InstructionType::Shl => {
                        writeln!(writer, "    pop rcx")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    shl rbx, cl")?;
                        writeln!(writer, "    push rbx")?;
                        ti += 1;
                    },
                    InstructionType::DivMod => {
                        writeln!(writer, "    xor rdx, rdx")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    div rbx")?;
                        writeln!(writer, "    push rax")?;
                        writeln!(writer, "    push rdx")?;
                        ti += 1;
                    },
                    InstructionType::Mul => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    mul rbx")?;
                        writeln!(writer, "    push rax")?;
                        ti += 1;
                    },
                    InstructionType::Syscall0 => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    syscall")?;
                        writeln!(writer, "    push rax")?;
                        ti += 1;
                    },
                    InstructionType::Syscall1 => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rdi")?;
                        writeln!(writer, "    syscall")?;
                        writeln!(writer, "    push rax")?;
                        ti += 1;
                    },
                    InstructionType::Syscall2 => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rdi")?;
                        writeln!(writer, "    pop rsi")?;
                        writeln!(writer, "    syscall")?;
                        writeln!(writer, "    push rax")?;
                        ti += 1;
                    },
                    InstructionType::Syscall3 => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rdi")?;
                        writeln!(writer, "    pop rsi")?;
                        writeln!(writer, "    pop rdx")?;
                        writeln!(writer, "    syscall")?;
                        writeln!(writer, "    push rax")?;
        
                        ti += 1;
                    },
                    InstructionType::Syscall4 => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rdi")?;
                        writeln!(writer, "    pop rsi")?;
                        writeln!(writer, "    pop rdx")?;
                        writeln!(writer, "    pop r10")?;
                        writeln!(writer, "    syscall")?;
                        writeln!(writer, "    push rax")?;
                        ti += 1;
                    },
                    InstructionType::Syscall5 => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rdi")?;
                        writeln!(writer, "    pop rsi")?;
                        writeln!(writer, "    pop rdx")?;
                        writeln!(writer, "    pop r10")?;
                        writeln!(writer, "    pop r8")?;
                        writeln!(writer, "    syscall")?;
                        writeln!(writer, "    push rax")?;
                        ti += 1;
                    },
                    InstructionType::Syscall6 => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    pop rdi")?;
                        writeln!(writer, "    pop rsi")?;
                        writeln!(writer, "    pop rdx")?;
                        writeln!(writer, "    pop r10")?;
                        writeln!(writer, "    pop r8")?;
                        writeln!(writer, "    pop r9")?;
                        writeln!(writer, "    syscall")?;
                        writeln!(writer, "    push rax")?;
                        ti += 1;
                    },
                    InstructionType::MemUse => {
                        writeln!(writer, "    push mem_{}", token.addr.unwrap())?;
                        ti += 1;
                    },
                    InstructionType::None => {
                        println!("{token:?}");
                        unreachable!()
                    },
                    InstructionType::FnCall => {
                        writeln!(writer, "    call {}", token.text)?;
                        ti += 1;
                    },
                    InstructionType::Return => {

                        // Experimental feature exported functions
                        if crate::config::ENABLE_EXPORTED_FUNCTIONS && should_push_ret {
                            writeln!(writer, "    pop rdx")?;
                            should_push_ret = false;
                        }

                        writeln!(writer, "    sub rbp, 8")?;
                        writeln!(writer, "    mov rbx, qword [rbp]")?;
                        writeln!(writer, "    push rbx")?;
                        writeln!(writer, "    ret")?;
                        ti += 1;
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
                    InstructionType::With => {
                        ti += 1;
                    }
                    InstructionType::ConstUse => {
                        writeln!(writer, "    mov rax, qword [const_{}]", token.text)?;
                        writeln!(writer, "    push rax")?;

                        let mut c = constants.get(&token.text).unwrap().clone();
                        c.used = true;
                        constants.remove(&token.text);
                        constants.insert(token.text.clone(), c);
                        ti += 1;
                    },
                }
            }


            OpType::Keyword(keyword) => {
                match keyword {

                    // block
                    KeywordType::If |
                    KeywordType::Do => {
                        writeln!(writer, "    pop rax")?;
                        writeln!(writer, "    test rax, rax")?;
                        writeln!(writer, "    jz addr_{}", token.jmp)?;
                        ti += 1;
                    }
                    KeywordType::Else => {
                        writeln!(writer, "    jmp addr_{}", token.jmp)?;
                        ti += 1;
                    },
                    KeywordType::While => {
                        ti += 1;
                    }
                    KeywordType::End => {
                        if ti + 1 != token.jmp {
                            writeln!(writer, "    jmp addr_{}", token.jmp)?;
                        }
                        ti += 1;
                    },
                    KeywordType::Memory => {
                        memories.push(Memory { size: token.value, loc: token.loc.clone(), id: token.addr.unwrap() });
                        ti += 1;
                    }
                    KeywordType::ConstantDef => {
                        // TODO: after we add c style strings add supoort for them in constants
                        let a = args.get_opt_level()? < 1;
                        let c = Constant{
                            loc: token.loc.clone(),
                            name: token.text.clone(),
                            value_i: Some(token.value),
                            value_s: None,
                            used: a,
                        };
                        
                        constants.insert(token.text.clone(), c);
                        ti += 1;
                    },
                    KeywordType::FunctionDef => {
                        writeln!(writer, "{}:", token.text)?;
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    mov qword [rbp], rbx")?;
                        writeln!(writer, "    add rbp, 8")?;
                        functions.push(Function { loc: token.loc.clone(), name: token.text.clone(), exter: false});
                        ti += 1;
                    },
                    KeywordType::FunctionDone => {
                        
                        if crate::config::ENABLE_EXPORTED_FUNCTIONS && should_push_ret {
                            writeln!(writer, "    pop rdx")?;
                            should_push_ret = false;
                        }

                        writeln!(writer, "    sub rbp, 8")?;
                        writeln!(writer, "    mov rbx, qword [rbp]")?;
                        writeln!(writer, "    push rbx")?;
                        writeln!(writer, "    ret")?;
                        ti += 1;
                    }
                    KeywordType::FunctionThen => ti += 1,
                    KeywordType::Function |
                    KeywordType::Include |
                    KeywordType::Inline |
                    KeywordType::Export |
                    KeywordType::Constant => unreachable!(),
                    KeywordType::FunctionDefExported => {

                        if !crate::config::ENABLE_EXPORTED_FUNCTIONS {
                            lerror!(&token.loc, "Experimental feature 'exported functions' is not enabled");
                            return Err(eyre!(""));
                        }

                        writeln!(writer, "global {}", token.text)?;
                        writeln!(writer, "{}:", token.text)?;
                        
                        writeln!(writer, "    pop rbx")?;
                        writeln!(writer, "    mov qword [rbp], rbx")?;
                        writeln!(writer, "    add rbp, 8")?;
                        warn!("External functions are highly experimental and should be treated as such");
                        if token.types.0 == 0 {
                            writeln!(writer, "    ; no arguments")?;
                        } else {
                            if token.types.0 >= 1 {
                                writeln!(writer, "    push rdi")?;
                            }
                            if token.types.0 >= 2 {
                                writeln!(writer, "    push rsi")?;
                            } 
                            if token.types.0 >= 3 {
                                writeln!(writer, "    push rdx")?;
                            } 
                            if token.types.0 >= 4 {
                                writeln!(writer, "    push rcx")?;
                            }
                            if token.types.0 >= 5 {
                                writeln!(writer, "    push r8")?;
                            }
                            if token.types.0 >= 6 {
                                writeln!(writer, "    push r9")?;
                            }
                            if token.types.0 >= 7 {
                                lerror!(&token.loc, "More than 6 arguments in an external function is not supported");
                                return Err(eyre!(""));
                            } 
                        }

                        if token.types.1 == 1 {
                            should_push_ret = true;
                        } else if token.types.1 > 1 {
                            lerror!(&token.loc, "More than 1 return arguments in an external function is not supported");
                            return Err(eyre!(""));
                        } 
                            
                        functions.push(Function { loc: token.loc.clone(), name: token.text.clone(), exter: false});
                        ti += 1;
                    },
                }
            }
        }
    }
    writeln!(writer, "addr_{ti}:")?;
    if !crate::config::ENABLE_EXPORTED_FUNCTIONS && !args.lib_mode {
        writeln!(writer, "end:")?;
        writeln!(writer, "    mov rax, 60")?;
        writeln!(writer, "    mov rdi, 0")?;
        writeln!(writer, "    syscall")?;
    }
    writeln!(writer, "segment .data")?;
    for (i, s) in strings.iter().enumerate() {
        let s_chars = s.chars().map(|c| (c as u32).to_string()).collect::<Vec<String>>();
        let s_list = s_chars.join(",");
        writeln!(writer, "    str_{}: db {} ; {}", i, s_list, s.escape_default())?;
    }
    
    for (_, c) in constants {
        if !c.used {
            continue;
        }

        if let Some(v) = &c.value_i {
            writeln!(writer, "    const_{}: dq {}", c.name, v)?;
        } else if let Some(_v) = &c.value_s {
            todo!();
        } else {
            unreachable!();
        }

    }
    
    
    writeln!(writer, "segment .bss")?;
    for s in memories {
        writeln!(writer, "    mem_{}: resb {}", s.id, s.size)?;
    }
    writeln!(writer, "    ret_stack: resq 256")?;
    
    // for t in tokens {
    //     println!("{t:?}");
    // }

    writer.flush()?;

    
    pre_compile_steps(
        String::from_utf8_lossy(writer.buffer()).to_string().as_str(),
        functions
    )?;

    linux_x86_64_compile_and_link(&of_a, &of_o, &of_c, args.quiet)?;
    if args.run {
        let c = linux_x86_64_run(&of_c, &[], args.quiet)?;
        return Ok(c);
    }


    Ok(0)
}


fn pre_compile_steps(_code: &str, functions: Vec<Function>) -> Result<()> {
    let mut has_main = false;

    for func in functions {
        if func.name == "main" {
            has_main = true;
        }
    }


    if !has_main {
        crate::errors::missing_main_fn();
        return Err(eyre!(""));
    }
    
    Ok(())
}