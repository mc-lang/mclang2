use std::{fs, path::PathBuf, io::{Write, BufWriter}, collections::HashMap};
use crate::{definitions::*, Args, warn, lerror};
use crate::compile::commands::linux_x86_64_compile_and_link;
use crate::definitions::InstructionType;
use super::{commands::linux_x86_64_run, Constant, Memory, Function};

use anyhow::{Result, bail};


pub fn compile(program: &Program, args: &Args) -> Result<i32>{
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

    let mut alloced_structs: Vec<(String, String)> = Vec::new();
    // println!("{}", tokens.len());
    let mut strings: Vec<String> = Vec::new();
    
    writeln!(writer, "BITS 64")?;
    writeln!(writer, "segment .text")?;

    writeln!(writer, "{}", super::MACRO_DEFINITIONS)?;
    writeln!(writer, "{}", super::DBG_PRINT)?;
    

    if !crate::config::ENABLE_EXPORTED_FUNCTIONS && !args.lib_mode {
        writeln!(writer, "global _start")?;
        writeln!(writer, "_start:")?; 
        writeln!(writer, "    lea rbp, [rel ret_stack]")?;
        writeln!(writer, "    call main")?;
        writeln!(writer, "    jmp end")?;
    }


    let mut ti = 0;
    while ti < program.ops.len() {
        let token = &program.ops[ti];
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
                if program.ops[ti-1].typ == OpType::Keyword(KeywordType::Else) ||
                program.ops[ti-1].typ == OpType::Keyword(KeywordType::End){
                    writeln!(writer, "addr_{ti}:")?;
                }
            }

            if ti + 1 < program.ops.len() && program.ops[ti+1].typ == OpType::Keyword(KeywordType::End) {
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
                        writeln!(writer, "    OP_PushInt {}", token.value)?;
                        ti += 1;
                    },
                    InstructionType::PushStr => {
                        writeln!(writer, "    OP_PushStr {}, str_{}", token.text.len(), strings.len())?;
                        strings.push(token.text.clone());
                        ti += 1;
                    }
                    InstructionType::PushCStr => {
                        writeln!(writer, "    OP_PushCStr str_{}", strings.len())?;
                        strings.push(token.text.clone());
                        ti += 1;
                    }
                    InstructionType::Drop => {
                        writeln!(writer, "    OP_Drop")?;
                        ti += 1;
                    },
                    InstructionType::Print => {
                        writeln!(writer, "    OP_Print")?;
                        ti += 1;
                    },
        
                    InstructionType::Dup => {
                        writeln!(writer, "    OP_Dup")?;
                        ti += 1;
                    },
        
                    InstructionType::Rot => {
                        writeln!(writer, "    OP_Rot")?;
                        ti += 1;
                    },
                    InstructionType::Swap => {
                        writeln!(writer, "    OP_Swap")?;
                        ti += 1;
                    },
                    InstructionType::Over => {
                        writeln!(writer, "    OP_Over")?;
                        ti += 1;
                    },
                    InstructionType::Read8 => {
                        writeln!(writer, "    OP_Load8")?;
                        ti += 1;
                    }
        
                    InstructionType::Write8 => {
                        writeln!(writer, "    OP_Store8")?;
                        ti += 1;
                    }
                    InstructionType::Read32 => {
                        writeln!(writer, "    OP_Load32")?;
                        ti += 1;
                    }
        
                    InstructionType::Write32 => {
                        writeln!(writer, "    OP_Store32")?;
                        ti += 1;
                    }
                    InstructionType::Read64 => {
                        writeln!(writer, "    OP_Load64")?;
                        ti += 1;
                    }
        
                    InstructionType::Write64 => {
                        writeln!(writer, "    OP_Store64")?;
                        ti += 1;
                    }
        
                    // math
                    InstructionType::Plus => {
                        writeln!(writer, "    OP_Plus")?;
                        ti += 1;
                    },
                    InstructionType::Minus => {
                        writeln!(writer, "    OP_Minus")?;
                        ti += 1;
                    },
                    InstructionType::Equals => {
                        writeln!(writer, "    OP_Equals")?;
                        ti += 1;
                    },
                    InstructionType::Lt => {
                        writeln!(writer, "    OP_Lt")?;
                        ti += 1;
                    },
                    InstructionType::Gt => {
                        writeln!(writer, "    OP_Gt")?;
                        ti += 1;
                    },
                    InstructionType::NotEquals => {
                        writeln!(writer, "    OP_NotEquals")?;
                        ti += 1;
                    },
                    InstructionType::Le => {
                        writeln!(writer, "    OP_Le")?;
                        ti += 1;
                    },
                    InstructionType::Ge => {
                        writeln!(writer, "    OP_Ge")?;
                        ti += 1;
                    },
                    InstructionType::Band => {
                        writeln!(writer, "    OP_Band")?;
                        ti += 1;
                    },
                    InstructionType::Bor => {
                        writeln!(writer, "    OP_Bor")?;
                        ti += 1;
                    },
                    InstructionType::Shr => {
                        writeln!(writer, "    OP_Shr")?;
                        ti += 1;
                    },
                    InstructionType::Shl => {
                        writeln!(writer, "    OP_Shl")?;
                        ti += 1;
                    },
                    InstructionType::DivMod => {
                        writeln!(writer, "    OP_DivMod")?;
                        ti += 1;
                    },
                    InstructionType::Mul => {
                        writeln!(writer, "    OP_Mul")?;
                        ti += 1;
                    },
                    InstructionType::Syscall0 => {
                        writeln!(writer, "    OP_Syscall0")?;
                        ti += 1;
                    },
                    InstructionType::Syscall1 => {
                        writeln!(writer, "    OP_Syscall1")?;
                        ti += 1;
                    },
                    InstructionType::Syscall2 => {
                        writeln!(writer, "    OP_Syscall2")?;
                        ti += 1;
                    },
                    InstructionType::Syscall3 => {
                        writeln!(writer, "    OP_Syscall3")?;
                        ti += 1;
                    },
                    InstructionType::Syscall4 => {
                        writeln!(writer, "    OP_Syscall4")?;
                        ti += 1;
                    },
                    InstructionType::Syscall5 => {
                        writeln!(writer, "    OP_Syscall5")?;
                        ti += 1;
                    },
                    InstructionType::Syscall6 => {
                        writeln!(writer, "    OP_Syscall6")?;
                        ti += 1;
                    },
                    InstructionType::MemUse => {
                        writeln!(writer, "    OP_MemUse {}", token.addr.unwrap())?;
                        ti += 1;
                    },
                    InstructionType::None => {
                        println!("{token:?}");
                        unreachable!()
                    },
                    InstructionType::FnCall => {
                        writeln!(writer, "    OP_FnCall {}", token.text)?;
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
                        writeln!(writer, "    OP_ConstUse {}", token.text)?;

                        let mut c = constants.get(&token.text).unwrap().clone();
                        c.used = true;
                        constants.remove(&token.text);
                        constants.insert(token.text.clone(), c);
                        ti += 1;
                    },
                    InstructionType::StructUse => {
                        writeln!(writer, "    OP_StructUse {}", token.text)?;
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
                    KeywordType::FunctionDefExported => {

                        if !crate::config::ENABLE_EXPORTED_FUNCTIONS {
                            lerror!(&token.loc, "Experimental feature 'exported functions' is not enabled");
                            bail!("");
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
                                bail!("");
                            } 
                        }

                        if token.types.1 == 1 {
                            should_push_ret = true;
                        } else if token.types.1 > 1 {
                            lerror!(&token.loc, "More than 1 return arguments in an external function is not supported");
                            bail!("");
                        } 
                            
                        functions.push(Function { loc: token.loc.clone(), name: token.text.clone(), exter: false});
                        ti += 1;
                    },
                    KeywordType::Function |
                    KeywordType::Include |
                    KeywordType::Inline |
                    KeywordType::Export |
                    KeywordType::Struct |
                    KeywordType::Constant => unreachable!(),
                }
            }
            OpType::Internal(t) => {
                match t {
                    InternalType::StructAlloc{name} => {
                        alloced_structs.push((name, token.text.clone()));
                        ti += 1;
                    },
                    InternalType::Arrow => panic!("{t:?}"),
                }
            },
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
    for m in memories {
        writeln!(writer, "    mem_{}: resb {}", m.id, m.size)?;
    }
    

    for s in alloced_structs {
        let Some(st) = program.struct_defs.get(&s.0) else {
            // TODO: Make better error
            panic!("Couldn find struct in struct defs");
        };

        let name = &s.1;
        let mut st_size = 0;

        writeln!(writer, "    struct_{name}:")?;
        for f in &st.fields {
            let size = f.1.get_size();
            writeln!(writer, "    struct_{name}.{}: resb {}", f.0, size)?;
            st_size += size;
        }

        writeln!(writer, "    struct_{name}.__size: db {}", st_size)?;

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

    linux_x86_64_compile_and_link(&of_a, &of_o, &of_c, args.quiet);

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
        bail!("");
    }
    
    Ok(())
}