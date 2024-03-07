use crate::definitions::Loc;

pub mod linux_x86_64;
pub mod commands;

#[derive(Debug, Clone)]
pub struct Constant {
    pub loc: Loc,
    pub name: String,
    pub value_i: Option<usize>,
    pub value_s: Option<String>,
    pub used: bool
    // extern: bool
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub size: usize,
    pub loc: Loc,
    pub id: usize
}

#[derive(Debug, Clone)]
pub struct Function {
    pub loc: Loc,
    pub name: String,
    pub exter: bool,
}

const DBG_PRINT: &'static str = "
_dbg_print:
    mov     r9, -3689348814741910323
    sub     rsp, 40
    mov     BYTE [rsp+31], 10
    lea     rcx, [rsp+30]
.L2:
    mov     rax, rdi
    lea     r8, [rsp+32]
    mul     r9
    mov     rax, rdi
    sub     r8, rcx
    shr     rdx, 3
    lea     rsi, [rdx+rdx*4]
    add     rsi, rsi
    sub     rax, rsi
    add     eax, 48
    mov     BYTE [rcx], al
    mov     rax, rdi
    mov     rdi, rdx
    mov     rdx, rcx
    sub     rcx, 1
    cmp     rax, 9
    ja      .L2
    lea     rax, [rsp+32]
    mov     edi, 1
    sub     rdx, rax
    xor     eax, eax
    lea     rsi, [rsp+32+rdx]
    mov     rdx, r8
    mov     rax, 1
    syscall
    add     rsp, 40
    ret
";

const MACRO_DEFINITIONS: &'static str = "\
%macro OP_PushInt 1
    mov rax, %1
    push rax
%endmacro

; str_len, str_id
%macro OP_PushStr 2
    mov rax, %1
    push rax
    mov rax, %2
    push rax
%endmacro

; str_id
%macro OP_PushCStr 1
    push rax
    mov rax, %1
    push rax
%endmacro

%macro OP_Drop 0
    pop rax
%endmacro

%macro OP_Print 0
    pop rdi
    call _dbg_print
%endmacro

%macro OP_Dup 0
    pop rax
    push rax
    push rax
%endmacro

%macro OP_Rot 0
    pop rax
    pop rbx
    pop rcx
    push rbx
    push rax
    push rcx
%endmacro

%macro OP_Swap 0
    pop rax
    pop rbx
    push rax
    push rbx
%endmacro

%macro OP_Over 0
    pop rax
    pop rbx
    push rbx
    push rax
    push rbx
%endmacro

%macro OP_Load8 0
    pop rax
    xor rbx, rbx
    mov bl, byte [rax]
    push rbx
%endmacro

%macro OP_Store8 0
    pop rbx
    pop rax
    mov byte [rax], bl
%endmacro

%macro OP_Load32 0
    pop rax
    xor rbx, rbx
    mov ebx, dword [rax]
    push rbx
%endmacro

%macro OP_Store32 0
    pop rbx
    pop rax
    mov dword[rax], ebx
%endmacro

%macro OP_Load64 0
    pop rax
    xor rbx, rbx
    mov rbx, qword [rax]
    push rbx
%endmacro

%macro OP_Store64 0
    pop rbx
    pop rax
    mov qword [rax], rbx
%endmacro

%macro OP_Plus 0
    pop rax
    pop rbx
    add rax, rbx
    push rax
%endmacro

%macro OP_Minus 0
    pop rax
    pop rbx
    sub rbx, rax
    push rbx
%endmacro

%macro OP_Equals 0
    mov rcx, 0
    mov rdx, 1
    pop rax
    pop rbx
    cmp rax, rbx
    cmove rcx, rdx
    push rcx
%endmacro

%macro OP_Lt 0
    mov rcx, 0
    mov rdx, 1
    pop rbx
    pop rax
    cmp rax, rbx
    cmovl rcx, rdx
    push rcx
%endmacro

%macro OP_Gt 0
    mov rcx, 0
    mov rdx, 1
    pop rbx
    pop rax
    cmp rax, rbx
    cmovg rcx, rdx
    push rcx
%endmacro

%macro OP_NotEquals 0
    mov rcx, 1
    mov rdx, 0
    pop rax
    pop rbx
    cmp rax, rbx
    cmove rcx, rdx
    push rcx
%endmacro

%macro OP_Le 0
    mov rcx, 0
    mov rdx, 1
    pop rbx
    pop rax
    cmp rax, rbx
    cmovle rcx, rdx
    push rcx
%endmacro

%macro OP_Ge 0
    mov rcx, 0
    mov rdx, 1
    pop rbx
    pop rax
    cmp rax, rbx
    cmovge rcx, rdx
    push rcx
%endmacro

%macro OP_Band 0
    pop rax
    pop rbx
    and rbx, rax
    push rbx
%endmacro

%macro OP_Bor 0
    pop rax
    pop rbx
    or rbx, rax
    push rbx
%endmacro

%macro OP_Shr 0
    pop rcx
    pop rbx
    shr rbx, cl
    push rbx
%endmacro

%macro OP_Shl 0
    pop rcx
    pop rbx
    shl rbx, cl
    push rbx
%endmacro

%macro OP_DivMod 0
    xor rdx, rdx
    pop rbx
    pop rax
    div rbx
    push rax
    push rdx
%endmacro

%macro OP_Mul 0
    pop rax
    pop rbx
    mul rbx
    push rax
%endmacro

%macro OP_Syscall0 0
    pop rax
    syscall
    push rax
%endmacro

%macro OP_Syscall1 0
    pop rax
    pop rdi
    syscall
    push rax
%endmacro

%macro OP_Syscall2 0
    pop rax
    pop rdi
    pop rsi
    syscall
    push rax
%endmacro

%macro OP_Syscall3 0
    pop rax
    pop rdi
    pop rsi
    pop rdx
    syscall
    push rax
%endmacro

%macro OP_Syscall4 0
    pop rax
    pop rdi
    pop rsi
    pop rdx
    pop r10
    syscall
    push rax
%endmacro

%macro OP_Syscall5 0
    pop rax
    pop rdi
    pop rsi
    pop rdx
    pop r10
    pop r8
    syscall
    push rax
%endmacro

%macro OP_Syscall6 0
    pop rax
    pop rdi
    pop rsi
    pop rdx
    pop r10
    pop r8
    pop r9
    syscall
    push rax
%endmacro

%macro OP_MemUse 1
    push mem_%1
%endmacro

%macro OP_FnCall 1
    call %1
%endmacro

%macro OP_ConstUse 1
    mov rax, qword [const_%1]
    push rax
%endmacro

%macro OP_StructUse 1
    push struct_%1
%endmacro
";