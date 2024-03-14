mod utils;

use std::path::PathBuf;
use std::{fs::File, io::BufWriter, path::Path};
use std::io::Write;
use crate::types::ast::{AstNode, Function, Module, Program};
use crate::types::token::{InstructionType, Token, TokenType};

use super::utils::run_cmd;
use super::Compiler;




pub struct X86_64LinuxNasmCompiler {
    strings: Vec<String>,
    func_mem_i: Vec<usize>,
    if_i: usize,
    while_i: usize,
    used_consts: Vec<String>
}

impl X86_64LinuxNasmCompiler {
    fn handle_token(&mut self, fd: &mut BufWriter<File>, _: &Program, token: &Token) -> anyhow::Result<()> {
        match &token.typ {
            TokenType::Instruction(it) => {
                match it {
                    InstructionType::PushInt(i) => {
                        writeln!(fd, "    mov rax, {i} ; PUSHINT({i})")?;
                        writeln!(fd, "    push rax")?;
                    },
                    InstructionType::PushStr(s) => {
                        writeln!(fd, "    push {}", s.len())?;
                        writeln!(fd, "    push str_{}; PUSHSTR({})", self.strings.len(), s.escape_debug())?;
                        self.strings.push(s.clone());
                    },
                    InstructionType::PushCStr(s) => {
                        writeln!(fd, "    push str_{}; PUSHCSTR({})", self.strings.len(), s.escape_debug())?;
                        self.strings.push(s.clone());
                    }, 
                    InstructionType::PushChar(c) => {
                        writeln!(fd, "    push {}; PUSHCHAR({})", *c as u8, c.escape_debug())?;
                    },
                    InstructionType::Drop => {
                        writeln!(fd, "    pop rax ; DROP")?;
                    },
                    InstructionType::Print => {
                        writeln!(fd, "    pop rdi")?;
                        writeln!(fd, "    call _dbg_print ; _DBG_PRINT")?;
                    },
                    InstructionType::Dup => {
                        writeln!(fd, "    pop rax  ; DUP")?;
                        writeln!(fd, "    push rax")?;
                        writeln!(fd, "    push rax")?;
                    },
                    InstructionType::Rot => {
                        writeln!(fd, "    pop rax ; ROT")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    pop rcx")?;
                        writeln!(fd, "    push rbx")?;
                        writeln!(fd, "    push rax")?;
                        writeln!(fd, "    push rcx")?;
                    },
                    InstructionType::Over => {
                        writeln!(fd, "    pop rax ; OVER")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    push rbx")?;
                        writeln!(fd, "    push rax")?;
                        writeln!(fd, "    push rbx")?;
                    },
                    InstructionType::Swap => {
                        writeln!(fd, "    pop rax ; SWAP")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    push rax")?;
                        writeln!(fd, "    push rbx")?;
                    }
                    InstructionType::Minus => {
                        writeln!(fd, "    pop rax ; SUB")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    sub rbx, rax")?;
                        writeln!(fd, "    push rbx")?;
                    },
                    InstructionType::Plus => {
                        writeln!(fd, "    pop rax ; ADD")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    add rax, rbx")?;
                        writeln!(fd, "    push rax")?;
                    },
                    InstructionType::Equals => {
                        writeln!(fd, "    mov rcx, 0 ; EQ")?;
                        writeln!(fd, "    mov rdx, 1")?;
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    cmp rax, rbx")?;
                        writeln!(fd, "    cmove rcx, rdx")?;
                        writeln!(fd, "    push rcx")?;
                    },
                    InstructionType::Gt => {
                        writeln!(fd, "    mov rcx, 0 ; GT")?;
                        writeln!(fd, "    mov rdx, 1")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    cmp rax, rbx")?;
                        writeln!(fd, "    cmovg rcx, rdx")?;
                        writeln!(fd, "    push rcx")?;
                    },
                    InstructionType::Lt => {
                        writeln!(fd, "    mov rcx, 0 ; LT")?;
                        writeln!(fd, "    mov rdx, 1")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    cmp rax, rbx")?;
                        writeln!(fd, "    cmovl rcx, rdx")?;
                        writeln!(fd, "    push rcx")?;
                    },
                    InstructionType::Ge => {
                        writeln!(fd, "    mov rcx, 0 ; GE")?;
                        writeln!(fd, "    mov rdx, 1")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    cmp rax, rbx")?;
                        writeln!(fd, "    cmovge rcx, rdx")?;
                        writeln!(fd, "    push rcx")?;
                    },
                    InstructionType::Le => {
                        writeln!(fd, "    mov rcx, 0 ; LE")?;
                        writeln!(fd, "    mov rdx, 1")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    cmp rax, rbx")?;
                        writeln!(fd, "    cmovle rcx, rdx")?;
                        writeln!(fd, "    push rcx")?;
                    },
                    InstructionType::NotEquals => {
                        writeln!(fd, "    mov rdx, 1 ; NEQ")?;
                        writeln!(fd, "    mov rcx, 0")?;
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    cmp rax, rbx")?;
                        writeln!(fd, "    cmove rcx, rdx")?;
                        writeln!(fd, "    push rcx")?;
                    },
                    InstructionType::Band => {
                        writeln!(fd, "    pop rax ; BAND")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    and rbx, rax")?;
                        writeln!(fd, "    push rbx")?;
                    },
                    InstructionType::Bor => {
                        writeln!(fd, "    pop rax ; BOR")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    or rbx, rax")?;
                        writeln!(fd, "    push rbx")?;
                    }
                    InstructionType::Shr => {
                        writeln!(fd, "    pop rcx")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    shr rbx, cl")?;
                        writeln!(fd, "    push rbx")?;
                    },
                    InstructionType::Shl => {
                        writeln!(fd, "    pop rcx")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    shl rbx, cl")?;
                        writeln!(fd, "    push rbx")?;
                    },
                    InstructionType::DivMod => {
                        writeln!(fd, "    xor rdx, rdx")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    div rbx")?;
                        writeln!(fd, "    push rax")?;
                        writeln!(fd, "    push rdx")?;
                    },
                    InstructionType::Mul => {
                        writeln!(fd, "    pop rax ; MUL")?;
                        writeln!(fd, "    pop rbx")?;
                        writeln!(fd, "    mul rbx")?;
                        writeln!(fd, "    push rax")?;
                    },
                    InstructionType::Read8 => {
                        writeln!(fd, "    pop rax ; READ8")?;
                        writeln!(fd, "    xor rbx, rbx")?;
                        writeln!(fd, "    mov bl, byte [rax]")?;
                        writeln!(fd, "    push rbx")?;
                    }
                    InstructionType::Write8 => {
                        writeln!(fd, "    pop rax ; WRITE 8")?;
                        writeln!(fd, "    xor rbx, rbx")?;
                        writeln!(fd, "    mov ebx, dword [rax]")?;
                        writeln!(fd, "    push rbx")?;
                    },
                    InstructionType::Read32 => {
                        writeln!(fd, "    pop rax ; READ 32")?;
                        writeln!(fd, "    xor rbx, rbx")?;
                        writeln!(fd, "    mov ebx, dword [rax]")?;
                        writeln!(fd, "    push rbx")?;
                    },
                    InstructionType::Write32 => {
                        writeln!(fd, "    pop rbx ; WRITE 32")?;
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    mov dword[rax], ebx")?;
                    },
                    InstructionType::Read64 => {
                        writeln!(fd, "    pop rax ; READ 32")?;
                        writeln!(fd, "    xor rbx, rbx")?;
                        writeln!(fd, "    mov rbx, qword [rax]")?;
                        writeln!(fd, "    push rbx")?;
                    },
                    InstructionType::Write64 => {
                        writeln!(fd, "    pop rbx ; WRITE 64")?;
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    mov qword[rax], rbx")?;
                    },
                    InstructionType::Syscall0 => {
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    syscall")?;
                        writeln!(fd, "    push rax")?;
                    },
                    InstructionType::Syscall1 => {
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    pop rdi")?;
                        writeln!(fd, "    syscall")?;
                        writeln!(fd, "    push rax")?;
                    },
                    InstructionType::Syscall2 => {
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    pop rdi")?;
                        writeln!(fd, "    pop rsi")?;
                        writeln!(fd, "    syscall")?;
                        writeln!(fd, "    push rax")?;
                    },
                    InstructionType::Syscall3 => {
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    pop rdi")?;
                        writeln!(fd, "    pop rsi")?;
                        writeln!(fd, "    pop rdx")?;
                        writeln!(fd, "    syscall")?;
                        writeln!(fd, "    push rax")?;
                    },
                    InstructionType::Syscall4 => {
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    pop rdi")?;
                        writeln!(fd, "    pop rsi")?;
                        writeln!(fd, "    pop rdx")?;
                        writeln!(fd, "    pop r10")?;
                        writeln!(fd, "    syscall")?;
                        writeln!(fd, "    push rax")?;
                    },
                    InstructionType::Syscall5 => {
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    pop rdi")?;
                        writeln!(fd, "    pop rsi")?;
                        writeln!(fd, "    pop rdx")?;
                        writeln!(fd, "    pop r10")?;
                        writeln!(fd, "    pop r8")?;
                        writeln!(fd, "    syscall")?;
                        writeln!(fd, "    push rax")?;
                    },
                    InstructionType::Syscall6 => {
                        writeln!(fd, "    pop rax")?;
                        writeln!(fd, "    pop rdi")?;
                        writeln!(fd, "    pop rsi")?;
                        writeln!(fd, "    pop rdx")?;
                        writeln!(fd, "    pop r10")?;
                        writeln!(fd, "    pop r8")?;
                        writeln!(fd, "    pop r9")?;
                        writeln!(fd, "    syscall")?;
                        writeln!(fd, "    push rax")?;
                    },
                    InstructionType::CastBool |
                    InstructionType::CastPtr |
                    InstructionType::CastInt |
                    InstructionType::CastVoid => (), //? Possibly have a use for this
                    InstructionType::TypeBool |
                    InstructionType::TypePtr |
                    InstructionType::TypeInt |
                    InstructionType::TypeVoid |
                    InstructionType::TypeAny |
                    InstructionType::FnCall |
                    InstructionType::MemUse |
                    InstructionType::ConstUse => unreachable!(),
                    InstructionType::Return => {
                        writeln!(fd, "    sub rbp, 8")?;
                        writeln!(fd, "    mov rbx, qword [rbp]")?;
                        writeln!(fd, "    push rbx")?;
                        writeln!(fd, "    ret")?;
                    },
                }
            },
            TokenType::Keyword(_) |
            TokenType::Unknown(_) => unreachable!(),
        }
        Ok(())
    }

    fn handle_module(&mut self, fd: &mut BufWriter<File>, prog: &Program, module: &Module) -> anyhow::Result<()> {
        writeln!(fd, "; {} Module {} START", module.path.join("::"), module.ident)?;
        self.handle_ast_list(fd, prog, module.body.clone())?;
        writeln!(fd, "; {} Module {} END", module.path.join("::"), module.ident)?;
        Ok(())
    }

    fn handle_function(&mut self, fd: &mut BufWriter<File>, prog: &Program, func: &Function) -> anyhow::Result<()> {
        writeln!(fd, "{f}: ; fn {f}", f=func.ident)?;
        writeln!(fd, "    pop rbx")?;
        writeln!(fd, "    mov qword [rbp], rbx")?;
        writeln!(fd, "    add rbp, 8")?;
        
        self.handle_ast_list(fd, prog, func.body.clone())?;
        
        writeln!(fd, "    sub rbp, 8")?;
        writeln!(fd, "    mov rbx, qword [rbp]")?;
        writeln!(fd, "    push rbx")?;
        writeln!(fd, "    ret")?;
        Ok(())
    }

    fn handle_ast_list(&mut self, fd: &mut BufWriter<File>, prog: &Program, ast: Vec<AstNode>) -> anyhow::Result<()> {
        for node in ast {
            match &node {
                AstNode::Function(f) => self.handle_function(fd, prog, f)?,
                AstNode::Constant(_) => (),
                AstNode::If(i) => {
                    let id = self.if_i;
                    self.if_i += 1;

                    writeln!(fd, "; IF({id}) START")?;
                    self.handle_ast_list(fd, prog, i.test.clone())?;
                    writeln!(fd, "    pop rax")?;
                    writeln!(fd, "    test rax, rax")?;
                    writeln!(fd, "    jz if_{id}_else")?;
                    writeln!(fd, "if_{id}_start:")?;
                    self.handle_ast_list(fd, prog, i.body.clone())?;
                    writeln!(fd, "    jmp if_{id}_end")?;
                    writeln!(fd, "if_{id}_else:")?;
                    self.handle_ast_list(fd, prog, vec![Box::leak(i.els.clone()).clone()])?;
                    writeln!(fd, "if_{id}_end:")?;
                    writeln!(fd, "; IF({id}) END")?;
                },
                AstNode::While(w) => {
                    let id = self.while_i;
                    self.while_i += 1;
                    writeln!(fd, "; WHILE({id}) START")?;
                    writeln!(fd, "while_{id}_test:")?;
                    self.handle_ast_list(fd, prog, w.test.clone())?;
                    writeln!(fd, "    pop rax")?;
                    writeln!(fd, "    test rax, rax")?;
                    writeln!(fd, "    jz while_{id}_exit")?;
                    writeln!(fd, "while_{id}_start:")?;
                    self.handle_ast_list(fd, prog, w.body.clone())?;
                    writeln!(fd, "while_{id}_end:")?;
                    writeln!(fd, "    jmp while_{id}_test")?;
                    writeln!(fd, "while_{id}_exit:")?;
                    writeln!(fd, "; WHILE({id}) END")?;
                },
                AstNode::Module(m) => self.handle_module(fd, prog, m)?,
                AstNode::Memory(m) => {
                    if !m.statc {
                        todo!()
                    }
                },
                AstNode::MemUse(_) => {
                    
                },
                AstNode::ConstUse(c) => {
                    self.used_consts.push(c.ident.clone());
                    writeln!(fd, "    mov rax, qword [c_{}]", c.ident)?;
                    writeln!(fd, "    push rax")?;
                },
                AstNode::FnCall(f)=> {
                    writeln!(fd, "    call {f} ; FUNCTIONCALL({f:?})", f=f.ident)?;
                },
                AstNode::Block(b)=> {
                    writeln!(fd, "; BLOCK({}) START", b.comment)?;
                    self.handle_ast_list(fd, prog, b.body.clone())?;
                    writeln!(fd, "; BLOCK({}) END", b.comment)?;
                },
                AstNode::Token(t) => self.handle_token(fd, prog, t)?,
                AstNode::Int(_, _) |
                AstNode::Str(_, _) |
                AstNode::CStr(_, _) |
                AstNode::Char(_, _) => unreachable!(),
            }
        }
        Ok(())
    }
}


impl Compiler for X86_64LinuxNasmCompiler {
    fn new() -> Self {
        Self {
            strings: Vec::new(),
            used_consts: Vec::new(),
            if_i: 0,
            while_i: 0,
        }
    }

    fn generate_asm(&mut self, prog: &Program, fd: &mut BufWriter<File>) -> anyhow::Result<()> {

        writeln!(fd, "BITS 64")?;
        writeln!(fd, "segment .text")?;
        writeln!(fd, "{}", utils::DBG_PRINT)?;
        writeln!(fd, "global _start")?;
        writeln!(fd, "_start:")?; 
        writeln!(fd, "    lea rbp, [rel ret_stack]")?;
        writeln!(fd, "    call main")?;
        writeln!(fd, "    jmp __MCL_END__")?;

        match &prog.ast {
            AstNode::Module(m) => {
                self.handle_module(fd, prog, m)?;
            },
            _ => panic!()
        }


        writeln!(fd, "__MCL_END__:")?;
        writeln!(fd, "    mov rax, 60")?;
        writeln!(fd, "    mov rdi, 0")?;
        writeln!(fd, "    syscall")?;

        writeln!(fd, "segment .data")?;
        for (_, v) in prog.constants.iter() {

            if !self.used_consts.contains(&v.ident) {
                continue;
            }

            match Box::leak(v.value.clone()) {
                AstNode::Int(_, val) => {
                    writeln!(fd, "c_{}: dq {}", v.ident, val)?;
                }
                AstNode::Str(_, val) |
                AstNode::CStr(_, val) => {
                    let s_chars = val.chars().map(|c| (c as u32).to_string()).collect::<Vec<String>>();
                    let s_list = s_chars.join(",");
                    writeln!(fd, "c_{}: db {} ; {}", v.ident, s_list, val.escape_debug())?;
                }
                AstNode::Char(_, val) => {
                    writeln!(fd, "c_{}: db {} ; '{}'", v.ident, *val as u8, val)?;
                }
                c => panic!("{c:?}")
            };
        }

        for (i, s) in self.strings.iter().enumerate() {
            let s_chars = s.chars().map(|c| (c as u32).to_string()).collect::<Vec<String>>();
            let s_list = s_chars.join(",");
            writeln!(fd, "str_{i}: db {} ; STRDEF({})", s_list, s.escape_debug())?;
        }
        writeln!(fd, "segment .bss")?;
        writeln!(fd, "ret_stack: resq 256")?;

        //TODO: Memories
        

        Ok(())
    }
    

    fn compile(&mut self, asm_fp: &Path, obj_fp: &Path) -> anyhow::Result<()> {
        run_cmd("nasm", vec![
            String::from("-felf64"),
            String::from("-o"),
            obj_fp.to_string_lossy().to_string(),
            asm_fp.to_string_lossy().to_string()
        ])
    }

    fn link(&mut self, obj_files: Vec<PathBuf>, bin_fp: &Path) -> anyhow::Result<()> {
        let mut args = vec![
            String::from("-o"),
            bin_fp.to_string_lossy().to_string(),
        ];

        for f in obj_files {
            args.push(f.to_string_lossy().to_string())
        }

        run_cmd("ld", args)
    }

    fn needed_dependencies(&mut self) -> Vec<&str> {
        vec![
            "nasm",
            "ld"
        ]
    }
}