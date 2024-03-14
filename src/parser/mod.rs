mod utils;
mod precompiler;
mod builtin;

use std::{collections::HashMap, path::Path};

use anyhow::{bail, Result};

use crate::{cli::CliArgs, lexer::Lexer, types::{ast::{AstNode, Block, ConstUse, Constant, FnCall, Function, If, MemUse, Module, Program, While}, common::Loc, token::{InstructionType, KeywordType, Token, TokenType}}};

use self::{builtin::get_builtin_symbols, precompiler::precompile, utils::{expect, peek_check, peek_check_multiple, PeekResult}};


bitflags::bitflags! {
    struct Flags: u8 {
        const EXTERN = 1 << 0;
        const EXPORT = 1 << 1;
        const INLINE = 1 << 2;
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
                KeywordType::If       => parse_if(&token, cli_args, prog, tokens)?,
                KeywordType::While    => parse_while(&token, cli_args, prog, tokens)?,
                KeywordType::Include  => parse_include(&token, cli_args, prog, tokens)?, //TODO: implement include
                KeywordType::Memory   => todo!(),
                KeywordType::Constant => parse_const(&token, cli_args, prog, tokens)?,
                KeywordType::Function => parse_function(&token, cli_args, prog, tokens, flags)?,
                KeywordType::Struct   => todo!(),
                KeywordType::Inline   => parse_inline(&token, cli_args, prog, tokens, flags)?,
                KeywordType::Export   => parse_export(&token, cli_args, prog, tokens, flags)?,
                KeywordType::Extern   => parse_extern(&token, cli_args, prog, tokens, flags)?,
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
                AstNode::Token(token)
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
    };
    Ok(ret)
}

// TODO: Extern functions
fn parse_function(org: &Token, cli_args: &CliArgs, prog: &mut Program, tokens: &mut Vec<Token>, flags: Flags ) -> Result<AstNode> {
    

    let name = expect(tokens, TokenType::Unknown(String::new()))?;
    expect(tokens, TokenType::Keyword(KeywordType::With))?;
    let mut args = Vec::new();
    
    loop {
        if let PeekResult::Correct(t) = peek_check_multiple(tokens, vec![
            TokenType::Instruction(InstructionType::TypeAny),
            TokenType::Instruction(InstructionType::TypeBool),
            TokenType::Instruction(InstructionType::TypeInt),
            TokenType::Instruction(InstructionType::TypePtr),
            TokenType::Instruction(InstructionType::TypeVoid),
        ]) {
            args.push(t.typ.clone());
        } else {
            break;
        }
        tokens.pop();
    }
    
    expect(tokens, TokenType::Keyword(KeywordType::Returns))?;
    
    let mut ret_args = Vec::new();
    
    loop {
        if let PeekResult::Correct(t) = peek_check_multiple(tokens, vec![
            TokenType::Instruction(InstructionType::TypeAny),
            TokenType::Instruction(InstructionType::TypeBool),
            TokenType::Instruction(InstructionType::TypeInt),
            TokenType::Instruction(InstructionType::TypePtr),
            TokenType::Instruction(InstructionType::TypeVoid),
        ]) {
            ret_args.push(t.typ.clone());
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
                        warn!("If is defined as `if ... do ... done`");
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

    let val = precompile(prog, body, &mut Vec::new())?;

    let name = name.lexem.clone()
        .replace("(", "_OPRN_")
        .replace(")", "_CPRN_");
    
    let def = Constant{
        loc: org.loc(),
        ident: name.clone(),
        value: Box::new(val),
    };


    prog.constants.insert(name, def.clone());

    Ok(AstNode::Constant(def))
}

fn parse_unknown(org: &Token, _: &CliArgs, prog: &mut Program, _: &mut Vec<Token>, _: Flags ) -> Result<AstNode> {
    //TODO: Typing?
    if let Some(func) = prog.functions.get(&org.lexem) {
        if func.inline {
            return Ok(AstNode::Block(Block{ loc: org.loc.clone(), body: func.body.clone(), comment: format!("inline fn {}", func.ident) }))
        } else {
            return Ok(AstNode::FnCall(FnCall{ loc: org.loc.clone(), ident: org.lexem.clone() }));
        }
    }

    if let Some(_) = prog.constants.get(&org.lexem) {
        return Ok(AstNode::ConstUse(ConstUse{ loc: org.loc.clone(), ident: org.lexem.clone() }));
    }

    if let Some(_) = prog.memories.get(&org.lexem) {
        return Ok(AstNode::MemUse(MemUse{ loc: org.loc.clone(), ident: org.lexem.clone() }));
    }

    dbg!(&prog.constants);
    error!({loc => org.loc.clone()}, "Unknown token {:?}", org);
    bail!("")
}