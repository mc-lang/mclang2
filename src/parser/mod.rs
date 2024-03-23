mod utils;
mod precompiler;
mod builtin;

use std::{collections::HashMap, path::Path};

use anyhow::{bail, Result};

use crate::{cli::CliArgs, lexer::Lexer, types::{ast::{AstNode, Block, ConstUse, Constant, FnCall, Function, If, MemSize, MemUse, Memory, Module, Program, StructDef, While}, common::Loc, token::{InstructionType, KeywordType, Token, TokenType, TypeType}}};

use self::{builtin::get_builtin_symbols, precompiler::{precompile_const, precompile_mem}, utils::{expect, peek_check, peek_check_multiple, PeekResult}};


bitflags::bitflags! {
    struct Flags: u8 {
        const EXTERN = 1 << 0;
        const EXPORT = 1 << 1;
        const INLINE = 1 << 2;
        const ALLOW_TYPES = 1 << 3;
    }
}

//TODO: Implement Module paths
pub fn parse(cli_args: &CliArgs, tokens: &mut Vec<Token>) -> Result<Program> {
    tokens.reverse();
    let module = Module {
        loc: Loc::new(&tokens[0].loc.file, 0, 0),
        ident: Path::new(&tokens[0].loc.file).file_stem().expect("Something went horribly wrong").to_string_lossy().to_string(),
        body: Vec::new(),
        path: vec![],
    };


    let mut prog = Program {
        ast: AstNode::Module(module.clone()),
        functions: HashMap::new(),
        constants: HashMap::new(),
        memories: HashMap::new(),
        struct_defs: HashMap::new()
    };

    let syms = get_builtin_symbols(&mut prog);
    match &mut prog.ast {
        AstNode::Module(module) => {
            module.body.push(syms)
        }
        _ => unreachable!()
    }
    
    while !tokens.is_empty() {
        let node = parse_next(cli_args, &mut prog, tokens, Flags::empty(), true)?;
        match &mut prog.ast {
            AstNode::Module(module) => {
                module.body.push(node);
            }
            _ => unreachable!()
        }
    }

    // prog.ast = module;

    Ok(prog)
}

fn parse_next(cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>, flags: Flags, is_module_root: bool) -> Result<AstNode> {
    let token = tokens.pop().expect("We broke reality!");
    // debug!({loc => token.loc.clone()}, "t: {:?}", token.typ);
    let ret = match &token.typ {
        TokenType::Keyword(kw) => {
            match kw {
                KeywordType::If        => parse_if(&token, cli_args, prog, tokens)?,
                KeywordType::While     => parse_while(&token, cli_args, prog, tokens)?,
                KeywordType::Include   => parse_include(&token, cli_args, prog, tokens)?,
                KeywordType::Memory    => parse_memory(&token, cli_args, prog, tokens, is_module_root)?,
                KeywordType::Constant  => parse_const(&token, cli_args, prog, tokens)?,
                KeywordType::Function  => parse_function(&token, cli_args, prog, tokens, flags)?,
                KeywordType::StructDef => parse_struct(&token, cli_args, prog, tokens)?,
                KeywordType::TypeDef   => todo!(),
                KeywordType::Inline    => parse_inline(&token, cli_args, prog, tokens, flags)?,
                KeywordType::Export    => parse_export(&token, cli_args, prog, tokens, flags)?,
                KeywordType::Extern    => parse_extern(&token, cli_args, prog, tokens, flags)?,
                kw => {
                    dbg!(&prog.constants);
                    error!({loc => token.loc}, "Unexpected token {kw:?}");
                    bail!("")
                }
            }
        },
        TokenType::Instruction(it) => {
            if is_module_root {
                error!({loc => token.loc}, "Unexpected token {it:?}, please create a main function, this is not a scripting language");
                bail!("")
            } else {
                match it {
                    InstructionType::StructPath(p) => parse_struct_path(&token, prog, p)?,
                    InstructionType::StructItem(p) => parse_struct_item(&token, prog, p)?,
                    _ => AstNode::Token(token)
                }
            }
        },
        TokenType::Unknown(ut) => {
            if is_module_root {
                error!({loc => token.loc}, "Unexpected token {ut:?}, please create a main function, this is not a scripting language");
                bail!("")
            } else {
                // AstNode::Token(token)
                parse_unknown(&token, cli_args, prog, tokens, flags)?
            }
        },
        TokenType::Type(t) => {
            if flags.contains(Flags::ALLOW_TYPES) {
                AstNode::Token(token)
            } else {
                error!({loc => token.loc}, "Unexpected type {t:?}");
                bail!("")
            }
        },
    };
    Ok(ret)
}

fn parse_struct_item(org: &Token, prog: &mut Program, p: &Vec<String>) -> Result<AstNode> {
    fn find_disp(strct: &StructDef, disp: &mut usize, path: &[String]) {
        let Some(p) = path.get(0) else {
            return
        };

        for item in &strct.body {
            if p == &item.0 {
                match &item.2 {
                    TypeType::Struct(strct) => {
                        *disp += item.1;
                        find_disp(strct, disp, &path[1..])
                    },
                    _ => {
                        *disp += item.1;
                    }
                }
            }
        }

    }
    if let Some(mem) = prog.memories.get(&p[0].to_string()) {
        match &mem.size {
            MemSize::Size(_) => {
                error!({loc => org.loc()}, "You can only access items in structs");
                bail!("")
            },
            MemSize::Type(t) => {
                match t {
                    TypeType::Struct(s) => {

                        let mut disp = 0;
                        find_disp(&s, &mut disp, &p[1..]);
                        return Ok(AstNode::MemUse(MemUse{
                            ident: p[0].clone(),
                            loc: org.loc(),
                            disp: Some(disp)
                        }));
                    },
                    _ => {
                        error!({loc => org.loc()}, "You can only access items in structs");
                        bail!("")
                    }
                }
            },
        }
    }

    error!("Failed to find memory {}", p[0]);
    bail!("")
}

fn parse_struct_path(org: &Token, prog: &mut Program, p: &Vec<String>) -> Result<AstNode> {

    fn find_disp(strct: &StructDef, disp: &mut usize, path: &[String]) {
        let Some(p) = path.get(0) else {
            return
        };

        for item in &strct.body {
            if p == &item.0 {
                match &item.2 {
                    TypeType::Struct(strct) => {
                        *disp += item.1;
                        find_disp(strct, disp, &path[1..])
                    },
                    _ => {
                        *disp += item.1;
                    }
                }
            }
        }

    }
    let mut disp = 0;
    if let Some(strct) = prog.struct_defs.get(&p[0].to_string()) {
        find_disp(strct, &mut disp, &p[1..]);
        return Ok(AstNode::StructDispPush{
            ident: org.lexem.clone(),
            loc: org.loc(),
            disp
        });
    }

    error!("Failed to find struct {}", p[0]);
    bail!("")
}

fn parse_struct(org: &Token, cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>) -> Result<AstNode> {
    let ident = expect(tokens, TokenType::Unknown(String::new()))?;
    expect(tokens, TokenType::Keyword(KeywordType::Do))?;


    let mut body: Vec<(String, usize, TypeType)> = Vec::new();
    let mut size = 0;

    loop {
        let ident = expect(tokens, TokenType::Unknown(String::new()))?;
        expect(tokens, TokenType::Keyword(KeywordType::Do))?;
        let typ = parse_next(cli_args, prog, tokens, Flags::ALLOW_TYPES, false)?;
        let (typ, disp) = match &typ {
            AstNode::Token(t) => {
                match &t.typ {
                    TokenType::Type(t) => {
                        let disp = size;
                        size += t.get_size();
                        (t, disp)
                    }
                    _ => {
                        error!({loc => t.loc()}, "Expected type, got {t:?}");
                        bail!("")
                    }
                }
            },
            t => {
                error!({loc => typ.loc()}, "Expected type, got {t:?}");
                bail!("")
            }
        };
        expect(tokens, TokenType::Keyword(KeywordType::End))?;

        body.push((ident.lexem, disp, typ.clone()));

        if peek_check(tokens, TokenType::Keyword(KeywordType::Done)).correct(){
            tokens.pop();
            break;
        }
        // if peek_check(tokens, TokenType::Keyword(KeywordType::End)).correct()
    };



    let def = StructDef{
        loc: org.loc(),
        ident: ident.lexem.clone(),
        body,
        size,
    };

    prog.struct_defs.insert(ident.lexem, def.clone());

    Ok(AstNode::StructDef(def))
}

fn parse_memory(org: &Token, cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>, is_module_root: bool) -> Result<AstNode> {
    let name = expect(tokens, TokenType::Unknown(String::new()))?;


    let mut body = Vec::new();
    loop {

        let t = peek_check(tokens, TokenType::Keyword(KeywordType::End));
        match t {
            PeekResult::Correct(_) => break,
            PeekResult::Wrong(_) => (),
            PeekResult::None => panic!("idk what to do herre"),
        }
        body.push(parse_next(cli_args, prog, tokens, Flags::ALLOW_TYPES, false)?);
    }
    expect(tokens, TokenType::Keyword(KeywordType::End))?;

    let val = precompile_mem(prog, body)?;

    let name = name.lexem.clone()
        .replace("(", "_OPRN_")
        .replace(")", "_CPRN_");
    
    let def = Memory{
        loc: org.loc(),
        ident: name.clone(),
        size: val,
        statc: is_module_root,
    };


    prog.memories.insert(name, def.clone());

    Ok(AstNode::Memory(def))

}

// TODO: Extern functions
fn parse_function(org: &Token, cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>, flags: Flags ) -> Result<AstNode> {
    

    let name = expect(tokens, TokenType::Unknown(String::new()))?;
    expect(tokens, TokenType::Keyword(KeywordType::With))?;
    let mut args = Vec::new();
    
    loop {
        if let PeekResult::Correct(t) = peek_check_multiple(tokens, vec![
            TokenType::Type(TypeType::Any),
            TokenType::Type(TypeType::U8),
            TokenType::Type(TypeType::U16),
            TokenType::Type(TypeType::U32),
            TokenType::Type(TypeType::U64),
            TokenType::Type(TypeType::Ptr),
            TokenType::Type(TypeType::Void),
            TokenType::Type(TypeType::Custom(Vec::new())),
        ]) {
            match &t.typ {
                TokenType::Type(tt) => {
                    args.push(tt.clone());
                }
                _ => unreachable!()
            }
        } else {
            break;
        }
        tokens.pop();
    }
    
    expect(tokens, TokenType::Keyword(KeywordType::Returns))?;
    
    let mut ret_args = Vec::new();
    
    loop {
        if let PeekResult::Correct(t) = peek_check_multiple(tokens, vec![
            TokenType::Type(TypeType::Any),
            TokenType::Type(TypeType::U8),
            TokenType::Type(TypeType::U16),
            TokenType::Type(TypeType::U32),
            TokenType::Type(TypeType::U64),
            TokenType::Type(TypeType::Ptr),
            TokenType::Type(TypeType::Void),
            TokenType::Type(TypeType::Custom(Vec::new())),
        ]) {
            match &t.typ {
                TokenType::Type(tt) => {
                    ret_args.push(tt.clone());
                }
                _ => unreachable!()
            }
        } else {
            break;
        }
        tokens.pop();
    }
    
    
    expect(tokens, TokenType::Keyword(KeywordType::Then))?;
    let mut body = Vec::new();
    loop {

        let fn_got = peek_check(tokens, TokenType::Keyword(KeywordType::Done));
        match fn_got {
            PeekResult::Correct(_) => break,
            PeekResult::Wrong(_) => (),
            PeekResult::None => panic!("idk what to do herre"),
        }
        body.push(parse_next(cli_args, prog, tokens, Flags::empty(), false)?);
    }
    expect(tokens, TokenType::Keyword(KeywordType::Done))?;

    let fn_def = Function {
        loc: org.loc(),
        inline: flags.contains(Flags::INLINE),
        extrn: flags.contains(Flags::EXTERN),
        export: flags.contains(Flags::EXPORT),
        ident: name.lexem.clone(),
        arg_types: args,
        ret_types: ret_args,
        body,
    };
    //TODO: Support module paths without double definitions
    // let mut mp = match &prog.ast {
    //     AstNode::Module(m) => {
    //         m.path.clone()
    //     }
    //     _ => panic!("")
    // };
    // mp.push(name.lexem.clone());
    // let mp = mp.join("::");

    // prog.function_aliases.insert(mp, name.lexem.clone());
    prog.functions.insert(name.lexem.clone(), fn_def.clone());
    Ok(AstNode::Function(fn_def))
}

fn parse_if(org: &Token, cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>) -> Result<AstNode> {
    let mut test: Vec<AstNode> = Vec::new();
    let mut body: Vec<AstNode> = Vec::new();
    let mut els: Vec<AstNode> = Vec::new();
    loop {
        test.push(parse_next(cli_args, prog, tokens, Flags::empty(), false)?);
        match peek_check(tokens, TokenType::Keyword(KeywordType::Do)) {
            PeekResult::Correct(_) => break,
            PeekResult::Wrong(w) => {
                match w.typ {
                    TokenType::Keyword(KeywordType::Then) => {
                        warn!({loc => w.loc()}, "If is defined as `if ... do ... done`");
                    }
                    _ => ()
                }
            },
            PeekResult::None => panic!("idk what to do herre"),
        }
    }

    expect(tokens, TokenType::Keyword(KeywordType::Do))?;



    loop {
        body.push(parse_next(cli_args, prog, tokens, Flags::empty(), false)?);
        match peek_check_multiple(tokens, vec![
            TokenType::Keyword(KeywordType::Else),
            TokenType::Keyword(KeywordType::Done),
            ]) {
            PeekResult::Correct(_) => break,
            PeekResult::Wrong(_) => (),
            PeekResult::None => panic!("idk what to do herre"),
        }
    }


    let els_t = tokens.last().expect("IMPOSSIBLEEE!!!!!!111").clone();
    let els = match els_t.typ.clone() {
        TokenType::Keyword(kw) => {
            match kw {
                KeywordType::Done => {
                    expect(tokens, TokenType::Keyword(KeywordType::Done))?;
                    AstNode::Block(Block{
                        comment: String::new(),
                        loc: els_t.loc,
                        body: Vec::new(),
                    })
                },
                KeywordType::Else => {
                    expect(tokens, TokenType::Keyword(KeywordType::Else))?;
                    if peek_check(tokens, TokenType::Keyword(KeywordType::If)).correct() {
                        let if_org =expect(tokens, TokenType::Keyword(KeywordType::If))?;
                        parse_if(&if_org, cli_args, prog, tokens)?
                    } else {
                        loop {
                            els.push(parse_next(cli_args, prog, tokens, Flags::empty(), false)?);
                            match peek_check(tokens, TokenType::Keyword(KeywordType::Done)) {
                                PeekResult::Correct(_) => break,
                                PeekResult::Wrong(w) => {
                                    match w.typ {
                                        TokenType::Keyword(KeywordType::Then) => {
                                            warn!("If is defined as `if ... do ... done`");
                                        }
                                        _ => ()
                                    }
                                },
                                PeekResult::None => panic!("idk what to do herre"),
                            }
                        }
                        expect(tokens, TokenType::Keyword(KeywordType::Done))?;

                        AstNode::Block(Block{
                            comment: String::new(),
                            loc: els_t.loc,
                            body: els,
                        })
                    }
                },
                e => {
                    error!({loc => els_t.loc.clone()}, "Expected {:?} or {:?} but got {:?}", KeywordType::Done, KeywordType::Else, e);
                    bail!("");
                }
            }
        },
        e => {
            error!({loc => els_t.loc.clone()}, "Expected {:?} or {:?} but got {:?}", KeywordType::Done, KeywordType::Else, e);
            bail!("");
        }
    };
    Ok(AstNode::If(If{
        test,
        body,
        els: Box::new(els),
        loc: org.loc(),
    }))
}


fn parse_while(org: &Token, cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>) -> Result<AstNode> {
    let mut test: Vec<AstNode> = Vec::new();
    let mut body: Vec<AstNode> = Vec::new();
    loop {
        test.push(parse_next(cli_args, prog, tokens, Flags::empty(), false)?);
        match peek_check(tokens, TokenType::Keyword(KeywordType::Do)) {
            PeekResult::Correct(_) => break,
            PeekResult::Wrong(w) => {
                match w.typ {
                    TokenType::Keyword(KeywordType::Then) => {
                        warn!("while is defined as `while ... do ... done`");
                    }
                    _ => ()
                }
            },
            PeekResult::None => panic!("idk what to do herre"),
        }
    }

    expect(tokens, TokenType::Keyword(KeywordType::Do))?;
    
    
    
    loop {
        body.push(parse_next(cli_args, prog, tokens, Flags::empty(), false)?);
        match peek_check_multiple(tokens, vec![
            TokenType::Keyword(KeywordType::Else),
            TokenType::Keyword(KeywordType::Done),
            ]) {
                PeekResult::Correct(_) => break,
            PeekResult::Wrong(_) => (),
            PeekResult::None => panic!("idk what to do herre"),
        }
    }

    
    expect(tokens, TokenType::Keyword(KeywordType::Done))?;

    Ok(AstNode::While(While{
        test,
        body,
        loc: org.loc(),
    }))
}

fn parse_inline(_: &Token, cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>, flags: Flags) -> Result<AstNode>  {
    let allowed_tokens = vec!{
        TokenType::Keyword(KeywordType::Function)
    };


    let Some(t) = tokens.last() else {
        error!("Expected one of {:?} after {:?} but found nothing", allowed_tokens, TokenType::Keyword(KeywordType::Inline));
        bail!("")
    };
    
    
    let mut found = false;

    for at in &allowed_tokens {
        if utils::cmp(at, &t.typ) {
            found = true;
        }
    }

    if !found {
        error!({loc => t.loc.clone()}, "Expected one of {:?} after {:?} but found {:?}", allowed_tokens, TokenType::Keyword(KeywordType::Inline), t.typ);
        bail!("");
    }

    
    parse_next(cli_args, prog, tokens, flags | Flags::INLINE,  false)
}

fn parse_extern(_: &Token, cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>, flags: Flags) -> Result<AstNode>  {
    let allowed_tokens = vec!{
        TokenType::Keyword(KeywordType::Function),
        TokenType::Keyword(KeywordType::Constant),
        TokenType::Keyword(KeywordType::Memory),
    };


    let Some(t) = tokens.last() else {
        error!("Expected one of {:?} after {:?} but found nothing", allowed_tokens, TokenType::Keyword(KeywordType::Extern));
        bail!("")
    };
    
    
    let mut found = false;

    for at in &allowed_tokens {
        if utils::cmp(at, &t.typ) {
            found = true;
        }
    }

    if !found {
        error!({loc => t.loc.clone()}, "Expected one of {:?} after {:?} but found {:?}", allowed_tokens, TokenType::Keyword(KeywordType::Extern), t.typ);
        bail!("");
    }

    
    parse_next(cli_args, prog, tokens, flags | Flags::EXTERN,  false)
}

fn parse_export(_: &Token, cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>, flags: Flags) -> Result<AstNode>  {
    let allowed_tokens = vec!{
        TokenType::Keyword(KeywordType::Function),
        TokenType::Keyword(KeywordType::Constant),
        TokenType::Keyword(KeywordType::Memory),
    };


    let Some(t) = tokens.last() else {
        error!("Expected one of {:?} after {:?} but found nothing", allowed_tokens, TokenType::Keyword(KeywordType::Export));
        bail!("")
    };
    
    
    let mut found = false;

    for at in &allowed_tokens {
        if utils::cmp(at, &t.typ) {
            found = true;
        }
    }

    if !found {
        error!({loc => t.loc.clone()}, "Expected one of {:?} after {:?} but found {:?}", allowed_tokens, TokenType::Keyword(KeywordType::Export), t.typ);
        bail!("");
    }

    
    parse_next(cli_args, prog, tokens, flags | Flags::EXPORT,  false)
}


fn parse_include(_: &Token, cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>) -> Result<AstNode> {
    let path = expect(tokens, 
        TokenType::Instruction(
            InstructionType::PushStr(
                String::new()
            )
        )
    )?;

    for ip in &cli_args.include_path {
        let p = ip.join(&path.lexem).to_path_buf();
        if p.exists() {
            info!({loc => path.loc.clone()}, "Lexing file {}", path.lexem.clone());
            let mut lexer = Lexer::new();
            lexer.lex(p.as_std_path())?;
            
            let mut mod_tokens = lexer.tokens;
            
            mod_tokens.reverse();
            
            let mut mp = match &prog.ast {
                AstNode::Module(m) => {
                    m.path.clone()
                }
                _ => panic!("")
            };
            
            mp.push(p.file_stem().unwrap().to_string());
            
            let module = Module {
                loc: Loc::new(path.loc.file.clone(), 0, 0),
                ident: Path::new(&path.loc.file).file_stem().expect("Something went horribly wrong").to_string_lossy().to_string(),
                body: Vec::new(),
                path: mp,
            };


            let mut mod_prog = Program {
                ast: AstNode::Module(module),
                functions: prog.functions.clone(),
                constants: prog.constants.clone(),
                memories: prog.memories.clone(),
                struct_defs: prog.struct_defs.clone(),
                
            };
            
            info!({loc => path.loc.clone()}, "Parsing file {}", path.lexem.clone());
            while !mod_tokens.is_empty() {
                let node = parse_next(cli_args, &mut mod_prog, &mut mod_tokens, Flags::empty(), true)?;
                match &mut mod_prog.ast {
                    AstNode::Module(module) => {
                        module.body.push(node);
                    }
                    _ => unreachable!()
                }
            }

            prog.constants = mod_prog.constants;
            prog.functions = mod_prog.functions;
            prog.memories = mod_prog.memories;
            return Ok(mod_prog.ast)
        }
        
    };

    error!("Could not find file {:?} in these locations: {:?}", path.lexem, cli_args.include_path);
    bail!("")

}

fn parse_const(org: &Token, cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>) -> Result<AstNode> {
    let name = expect(tokens, TokenType::Unknown(String::new()))?;


    let mut body = Vec::new();
    loop {

        let t = peek_check(tokens, TokenType::Keyword(KeywordType::End));
        match t {
            PeekResult::Correct(_) => break,
            PeekResult::Wrong(_) => (),
            PeekResult::None => panic!("idk what to do herre"),
        }
        body.push(parse_next(cli_args, prog, tokens, Flags::empty(), false)?);
    }
    expect(tokens, TokenType::Keyword(KeywordType::End))?;

    let val = precompile_const(prog, body, &mut Vec::new())?;

    
    let def = Constant{
        loc: org.loc(),
        ident: name.lexem.clone(),
        value: Box::new(val),
    };


    prog.constants.insert(name.lexem, def.clone());

    Ok(AstNode::Constant(def))
}

fn parse_unknown(org: &Token, _: &CliArgs, prog: &mut Program, _: &mut Vec<Token>, _: Flags ) -> Result<AstNode> {
    //TODO: Typing?
    if let Some(func) = prog.functions.get(&org.lexem.clone()) {
        if func.inline {
            return Ok(AstNode::Block(Block{ loc: org.loc.clone(), body: func.body.clone(), comment: format!("inline fn {}", func.ident) }))
        } else {
            return Ok(AstNode::FnCall(FnCall{ loc: org.loc.clone(), ident: org.lexem.clone() }));
        }
    }

    if let Some(_) = prog.constants.get(&org.lexem.clone()) {
        return Ok(AstNode::ConstUse(ConstUse{ loc: org.loc.clone(), ident: org.lexem.clone() }));
    }

    if let Some(_) = prog.memories.get(&org.lexem.clone()) {
        return Ok(AstNode::MemUse(MemUse{ loc: org.loc.clone(), ident: org.lexem.clone(), disp: None }));
    }

    if let Some(t) = prog.struct_defs.get(&org.lexem.clone()) {
        return Ok(AstNode::Token(Token {
            typ: TokenType::Type(TypeType::Struct(t.clone())),
            loc: org.loc(),
            lexem: org.lexem.clone(),
        }));
    }


    // if org.lexem.clone().contains("::") {
    //     let pth = org.lexem.clone();
    //     let pth = pth.split("::").collect::<Vec<&str>>();
    //     dbg!(prog.struct_defs.clone());
    //     if let Some(t) = prog.struct_defs.get(&pth[0].to_string()) {
    //         if let Some(i) = t.body.iter().find(|i| i.0 == pth[1].to_string()) {
    //             return Ok(AstNode::StructDispPush{
    //                 ident: org.lexem.clone(),
    //                 loc: org.loc(),
    //                 disp: i.1
    //             });

    //         }
    //     }
    // }


    // dbg!(&prog.constants);
    debug!({loc => org.loc.clone()}, "Unknown token");
    error!({loc => org.loc.clone()}, "Unknown token {:?}", org);
    bail!("")
}