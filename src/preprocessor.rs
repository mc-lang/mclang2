use std::collections::HashMap;
use std::ops::Deref;
use std::path::{PathBuf, Path};


use color_eyre::Result;
use eyre::eyre;

use crate::constants::{Loc, OpType, TokenType, KeywordType, InstructionType, Operator};
use crate::lexer::lex;
use crate::precompiler::precompile;
use crate::{lerror, Args, warn, linfo, parser};
use crate::parser::lookup_word;



#[derive(Debug, Clone)]
pub struct Function {
    pub loc: Loc,
    pub name: String,
    pub inline: bool,
    pub tokens: Option<Vec<Operator>>
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub loc: Loc,
    pub name: String
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub loc: Loc,
    pub id: usize
    
}

type Functions = HashMap<String, Function>;
type Memories = HashMap<String, Memory>;
type Constants = HashMap<String, Constant>;

#[derive(Debug, Clone)]
pub struct Preprocessor<'a> {
    pub program: Vec<Operator>,
    pub functions: Functions,
    pub memories: Memories,
    pub constants: Constants,
    pub in_function: Option<String>,
    args: &'a Args
}


impl<'a> Preprocessor<'a> {
    pub fn new(prog: Vec<Operator>, args: &'a Args) -> Self {
        Self {
            program: prog,
            args,
            functions: HashMap::new(),
            memories: HashMap::new(),
            constants: HashMap::new(),
            in_function: None
        }
    }


    pub fn preprocess(&mut self) -> Result<&mut Preprocessor<'a>>{
        // println!("pre: has do tokens: {:?}", self.program.iter().map(|t| if t.typ == OpType::Keyword(KeywordType::Do) {Some(t)} else {None} ).collect::<Vec<Option<&Operator>>>());
        
        let mut f_inline = false;
        let mut f_export = false;

        let mut program: Vec<Operator> = Vec::new();

        let mut rtokens = self.program.clone();
        rtokens.reverse();
        while !rtokens.is_empty() {
            let mut op = rtokens.pop().unwrap();
            // println!("{token:?}");
            let op_type = op.typ.clone();
            match op_type {
                OpType::Keyword(KeywordType::Include) => {
                    if rtokens.is_empty() {
                        lerror!(&op.loc, "Include path not found, expected {} but found nothing", TokenType::String.human());
                        return Err(eyre!(""));
                    }

                    let include_path = rtokens.pop().unwrap();

                    if include_path.tok_typ != TokenType::String {
                        lerror!(&include_path.loc, "Bad include path, expected {} but found {}", TokenType::String.human(), include_path.typ.human());
                        return Err(eyre!(""));
                    }

                    let mut in_paths = self.args.include.clone();
                    in_paths.append(&mut crate::DEFAULT_INCLUDES.to_vec().clone().iter().map(|f| (*f).to_string()).collect::<Vec<String>>());
                    
                    let mut include_code = String::new();
                    let mut pth = PathBuf::new();
                    if include_path.text.chars().next().unwrap() == '.' {
                        let p = Path::new(include_path.loc.0.as_str());
                        let p = p.parent().unwrap();
                        let p = p.join(&include_path.text);
                        pth = p.clone();
                        include_code = std::fs::read_to_string(p)?;
                    } else {   
                        for path in in_paths {
                            let p = PathBuf::from(path);
                            let p = p.join(&include_path.text);
                            pth = p.clone();
                            
                            if p.exists() {
                                include_code = std::fs::read_to_string(p)?;
                                break;
                            }
                            
                        }
                    }

                    if include_code.is_empty() {
                        lerror!(&include_path.loc, "Include file in path '{}' was not found or is empty", include_path.text);
                        return Err(eyre!(""));
                    }
                    let a = pth.to_str().unwrap().to_string();
                    let code = lex(&include_code, a.as_str(), self.args);
                    let mut p = parser::Parser::new(code, self.args, Some(self.clone()));
                    let mut code = p.parse()?;

                    self.set_constants(p.preprocessor.get_constants());
                    self.set_functions(p.preprocessor.get_functions());
                    self.set_memories(p.preprocessor.get_memories());
                    code.reverse();
                    rtokens.append(&mut code);


                }

                OpType::Keyword(KeywordType::Memory) => {
                    if rtokens.is_empty() {
                        lerror!(&op.loc, "Memory name not found, expected {} but found nothing", TokenType::String.human());
                        return Err(eyre!(""));
                    }

                    let name = rtokens.pop().unwrap();

                    self.is_word_available(&name, KeywordType::Memory)?;

                    let mut code: Vec<Operator> = Vec::new();

                    let mut depth = 0;
                    while !rtokens.is_empty() {
                        let t = rtokens.pop().unwrap();
                        let typ = t.typ.clone();
                        if typ == OpType::Keyword(KeywordType::End) && depth == 0 {
                            break;
                        } else if typ == OpType::Keyword(KeywordType::End) && depth != 0 {
                            depth -= 1;
                            code.push(t);
                        } else if typ == OpType::Keyword(KeywordType::If) || typ == OpType::Keyword(KeywordType::Do) {
                            code.push(t);
                            depth += 1;
                        } else {
                            code.push(t);
                        }
                    }
                    let res = precompile(&code)?;


                    if res.len() != 1 {
                        lerror!(&op.loc, "Expected 1 number, got {:?}", res);
                        return Err(eyre!(""));
                    }
                    op.value = res[0];
                    op.addr = Some(self.memories.len());
                    program.push(op.clone());

                    self.memories.insert(name.text, Memory { loc: op.loc, id: self.memories.len() });

                }

                OpType::Keyword(KeywordType::Function) => {
                    if rtokens.is_empty() {
                        lerror!(&op.loc, "Function name not found, expected {} but found nothing", TokenType::Word.human());
                        return Err(eyre!(""));
                    }

                    let mut name = rtokens.pop().unwrap();

                    if let '0'..='9' = name.text.chars().next().unwrap() {
                        lerror!(&name.loc, "Function name starts with a number which is not allowed");
                        return Err(eyre!(""));
                    }

                    // let mut should_warn = false;
                    for c in name.text.clone().chars() {
                        match c {
                            'a'..='z' |
                            'A'..='Z' |
                            '0'..='9' |
                            '-' | '_' => (),
                            '(' | ')' => {
                                name.text = name.text.clone().replace('(', "__OP_PAREN__").replace(')', "__CL_PAREN__");
                            }
                            _ => {
                                lerror!(&name.loc, "Function name contains '{c}', which is unsupported");
                                return Err(eyre!(""));
                            }
                        }
                    }
                    // if should_warn {
                        //TODO: add -W option in cli args to enable more warnings
                        //lwarn!(&function_name.loc, "Function name contains '(' or ')', this character is not supported but will be replaced with '__OP_PAREN__' or '__CL_PAREN__' respectively ");
                    // }

                    self.is_word_available(&name, KeywordType::Function)?;
                    
                    
                    if f_inline {
                        f_inline = false;
                        let mut prog: Vec<Operator> = Vec::new();
                        let mut depth = -1;
                        while !rtokens.is_empty() {
                            let op = rtokens.pop().unwrap();

                            match op.typ.clone() {
                                OpType::Instruction(i) => {
                                    match i {
                                        InstructionType::TypeAny |
                                        InstructionType::TypeBool |
                                        InstructionType::TypeInt |
                                        InstructionType::TypePtr |
                                        InstructionType::With |
                                        InstructionType::Returns |
                                        InstructionType::TypeVoid => {
                                            if depth >= 0 {
                                                prog.push(op);
                                            }
                                        },
                                        _ => prog.push(op)
                                    }
                                }
                                OpType::Keyword(k) => {
                                    match k {
                                        KeywordType::Inline |
                                        KeywordType::Include => {
                                            todo!("make error")
                                        },
                                        KeywordType::FunctionThen => {
                                            if depth >= 0 {
                                                prog.push(op);
                                            }
                                            depth += 1;
                                        },
                                        KeywordType::FunctionDone => {
                                            if depth == 0 {
                                                break;
                                            }

                                            depth -= 1;
                                        },
                                        _ => prog.push(op)
                                    }
                                }
                            }
                        }
                        let mut pre = self.clone();
                        pre.program = prog;
                        if name.text.chars().next().unwrap() == '.' {
                            pre.in_function = Some(name.text[1..].to_string());
                        }
                        pre.preprocess()?;
                        prog = pre.get_ops();

                        self.functions.insert(name.text.clone(), Function{
                            loc: name.loc.clone(),
                            name: name.text.clone(),
                            inline: true,
                            tokens: Some(prog)
                        });
                        
                    } else if f_export {
                        f_export = false;
                        self.functions.insert(name.text.clone(), Function{
                            loc: name.loc.clone(),
                            name: name.text.clone(),
                            inline: false,
                            tokens: None
                        });
                        let mut a: Vec<Operator> = Vec::new();
                        let mut fn_def = op.clone();
                        a.push(rtokens.pop().unwrap());
                        let mut ret = false;
                        while !rtokens.is_empty() {
                            let op = rtokens.pop().unwrap();
                            // println!("{:?}",op);
                            a.push(op.clone());
                            if op.typ == OpType::Instruction(InstructionType::Returns) {
                                ret = true;
                            }

                            if op.typ == OpType::Keyword(KeywordType::FunctionThen) {
                                break;
                            }

                            if op.typ == OpType::Instruction(InstructionType::TypeBool) ||
                                op.typ == OpType::Instruction(InstructionType::TypeInt) ||
                                op.typ == OpType::Instruction(InstructionType::TypePtr) {

                                if ret {
                                    fn_def.types.1 += 1;
                                } else {
                                    fn_def.types.0 += 1;
                                }
                            }
                        }

                        fn_def.typ = OpType::Keyword(KeywordType::FunctionDefExported);
                        fn_def.text = name.text;
                        // fn_def.set_types(args, rets);
                        // println!("{:?}", fn_def.types);
                        program.push(fn_def);
                        program.append(&mut a);


                    } else {

                        self.functions.insert(name.text.clone(), Function{
                            loc: name.loc.clone(),
                            name: name.text.clone(),
                            inline: false,
                            tokens: None
                        });
                        
                        let mut fn_def = op.clone();
                        fn_def.typ = OpType::Keyword(KeywordType::FunctionDef);
                        fn_def.text = name.text;
                        // println!("{:?}", token);
                        program.push(fn_def);
                    }
                }
                
                OpType::Keyword(KeywordType::Constant) => {
                    if rtokens.is_empty() {
                        lerror!(&op.loc, "Constant name not found, expected {} but found nothing", TokenType::Word.human());
                        return Err(eyre!(""));
                    }
                    // println!("{token:?}");

                    let mut name = rtokens.pop().unwrap();
                    // let mut should_warn = false;
                    

                    if let '0'..='9' | '.' = name.text.chars().next().unwrap() {
                        lerror!(&name.loc, "Constant name starts with a number or dot which is not allowed");
                        return Err(eyre!(""));
                    }

                    for c in name.text.clone().chars() {
                        match c {
                            'a'..='z' |
                            'A'..='Z' |
                            '0'..='9' |
                            '-' | '_' => (),
                            '(' | ')' => {
                                // should_warn = true;
                                name.text = name.text.clone().replace('(', "__OP_PAREN__").replace(')', "__CL_PAREN__");
                            }
                            _ => {
                                lerror!(&name.loc, "Constant name contains '{c}', which is unsupported");
                                return Err(eyre!(""));
                            }
                        }
                    }

                    // if should_warn {
                        //TODO: add -W option in cli args to enable more warnings
                        //lwarn!(&name.loc, "Constant name contains '(' or ')', this character is not supported but will be replaced with '__OP_PAREN__' or '__CL_PAREN__' respectively ");
                    // }
                    
                    self.is_word_available(&name, KeywordType::Constant)?;
                    
                    
                    self.constants.insert(name.text.clone(), Constant{
                        loc: name.loc.clone(),
                        name: name.text.clone(),
                    });

                    // println!("{:?}", self.constants);

                    let mut const_def = op.clone();
                    const_def.typ = OpType::Keyword(KeywordType::ConstantDef);
                    const_def.text = name.text;

                    let item = rtokens.pop().unwrap();
                    if item.tok_typ == TokenType::Int {
                        const_def.value = item.value;
                    } else {
                        lerror!(&op.loc, "For now only {:?} is allowed in constants", TokenType::Int);
                        return Err(eyre!(""));
                    }

                    let posibly_end = rtokens.pop();
                    // println!("end: {posibly_end:?}");
                    if posibly_end.is_none() || posibly_end.unwrap().typ != OpType::Keyword(KeywordType::End) {
                        lerror!(&op.loc, "Constant was not closed with an 'end' instruction, expected 'end' but found nothing");
                        return Err(eyre!(""));
                    }
                    // token.value = 

                    program.push(const_def);
                }  

                OpType::Keyword(KeywordType::Inline) => {
                    if f_export {
                        lerror!(&op.loc, "Function is already marked as exported, function cannot be inline and exported at the same time");
                        return Err(eyre!(""));
                    } else if f_inline {
                        lerror!(&op.loc, "Function is already marked as inline, remove this inline Keyword");
                        return Err(eyre!(""));
                    } else {
                        f_inline = true;
                    }
                }

                OpType::Keyword(KeywordType::Export) => {
                    if !crate::config::ENABLE_EXPORTED_FUNCTIONS {
                        lerror!(&op.loc, "Experimental feature Exported functions not enabled");
                        return Err(eyre!(""));
                    }
                    if f_inline {
                        lerror!(&op.loc, "Function is already marked as inline, function cannot be inline and exported at the same time");
                        return Err(eyre!(""));
                    } else if f_export {
                        lerror!(&op.loc, "Function is already marked as extern, remove this extern Keyword");
                        return Err(eyre!(""));
                    } else {
                        f_export = true;
                    }
                }

                _ => {
                    program.push(op);
                }
            }
        }
        self.program = program;
        // println!("has do tokens: {:?}", self.program.iter().map(|t| if t.typ == OpType::Keyword(KeywordType::Do) {Some(t)} else {None} ).collect::<Vec<Option<&Operator>>>());
        //* Feel free to fix this horrifying shit
        //* i wanna kms
        let mut times = 0;
        // dbg!(program.clone());
        while self.program.iter().map(|f| {
            if f.tok_typ == TokenType::Word && 
                f.typ != OpType::Instruction(InstructionType::FnCall) && 
                f.typ != OpType::Instruction(InstructionType::MemUse) &&
                f.typ != OpType::Keyword(KeywordType::FunctionDef) &&
                f.typ != OpType::Keyword(KeywordType::FunctionDefExported) &&
                f.typ != OpType::Keyword(KeywordType::ConstantDef) &&
                f.typ != OpType::Instruction(InstructionType::ConstUse) {
                lookup_word(&f.text, &f.loc)
            } else {
                OpType::Instruction(InstructionType::PushInt) // i hate myself, this is a randomly picked optype so its happy and works
            }

        }).collect::<Vec<OpType>>().contains(&OpType::Instruction(InstructionType::None)){

            if times >= 50 {
                warn!("File import depth maxed out, if the program crashes try reducing the import depth, good luck youll need it");
                break
            }
            self.expand()?;
            times += 1;
        }
        Ok(self)
    }

    pub fn expand(&mut self) -> Result<()> {
        let mut program: Vec<Operator> = Vec::new();
        // println!("{:?}", self.functions);
        let mut rtokens = self.program.clone();
        rtokens.reverse();

        while !rtokens.is_empty() {
            let op = rtokens.pop().unwrap();
            let op_type = op.typ.clone();
            if op.tok_typ == TokenType::Word {
                match op_type {
                    OpType::Instruction(InstructionType::None) => {
                        let m = self.functions.get(&op.text.clone().replace('(', "__OP_PAREN__").replace(')', "__CL_PAREN__"));
                        let mem = self.memories.get(&op.text);
                        let cons = self.constants.get(&op.text.clone().replace('(', "__OP_PAREN__").replace(')', "__CL_PAREN__"));
                        if let Some(m) = m {
                            if m.inline {
                                program.append(&mut m.tokens.clone().unwrap());
                            } else {                                
                                let mut t = op.clone();
                                t.typ = OpType::Instruction(InstructionType::FnCall);
                                t.text = m.name.clone();
                                program.push(t.clone());
                            }

                            // println!("##### {:?}", t);
                        } else if let Some(mem) = mem {
                            let mut t = op.clone();
                            t.addr = Some(mem.deref().id);
                            t.typ = OpType::Instruction(InstructionType::MemUse);
                            program.push(t);
                        } else if let Some(cons) = cons {
                            let mut t = op.clone();
                            t.text = cons.deref().name.clone();
                            t.typ = OpType::Instruction(InstructionType::ConstUse);
                            program.push(t);
                            
                        } else {
                            lerror!(&op.loc, "Preprocess: Unknown word '{}'", op.text.clone());
                            return Err(eyre!(""));
                        }
                    }
                    _ => {
                        program.push(op.clone());
                    }
                }
            } else {
                program.push(op.clone());
            }
            
            // if op.typ == OpType::Keyword(KeywordType::Do) {
            //     println!("expand: {:?}", op);
            //     program.push(op.clone());
            // }
            
        }
        // println!("expand: has do tokens: {:?}", program.iter().map(|t| if t.typ == OpType::Keyword(KeywordType::Do) {Some(t)} else {None} ).collect::<Vec<Option<&Operator>>>());

        self.program = program;
        // println!("{:#?}", self.program);
        // println!("{:?}", self.program.last().unwrap());
        Ok(())
    }

    

    pub fn get_ops(&mut self) -> Vec<Operator> {
        self.program.clone()
    }
    pub fn is_word_available(&self, word: &Operator, typ: KeywordType) -> Result<bool> {

        match typ {
            KeywordType::Memory |
            KeywordType::Constant |
            KeywordType::Function => (),
            _ => panic!()
        }
        
        if word.tok_typ != TokenType::Word {
            lerror!(&word.loc, "Bad {typ:?}, expected {} but found {}", TokenType::Word.human(), word.typ.human());
            if crate::DEV_MODE {println!("{word:?}")}
            return Err(eyre!(""));
        }

        let w = lookup_word(&word.text, &word.loc);
        if w != OpType::Instruction(InstructionType::None) {
            lerror!(&word.loc, "Bad {typ:?}, {typ:?} definition cannot be builtin word, got {:?}", word.text);
            if crate::DEV_MODE {println!("{word:?}")}
            return Err(eyre!(""));
        }

        let m = self.memories.get(&word.text);
        if let Some(m) = m {
            if typ == KeywordType::Memory {
                lerror!(&word.loc, "Memories cannot be redefined, got {}", word.text);
                linfo!(&m.loc, "first definition here"); 
                if crate::DEV_MODE {println!("{word:?}")}
                return Err(eyre!(""));
            }
            lerror!(&word.loc, "{typ:?} cannot replace memory, got {}", word.text);
            linfo!(&m.loc, "first definition here"); 
            if crate::DEV_MODE {println!("{word:?}")}
            return Err(eyre!(""));
        }
        let f = self.functions.get(&word.text);
        if let Some(f) = f {
            if typ == KeywordType::Function {
                lerror!(&word.loc, "Functions cannot be redefined, got {}", word.text);
                linfo!(&f.loc, "first definition here"); 
                if crate::DEV_MODE {println!("{word:?}")}
                return Err(eyre!(""));
            }
            lerror!(&word.loc, "{typ:?} cannot replace function, got {}", word.text);
            linfo!(&f.loc, "first definition here"); 
            if crate::DEV_MODE {println!("{word:?}")}
            return Err(eyre!(""));
        }
        let c = self.constants.get(&word.text);
        if let Some(c) = c {
            if typ == KeywordType::Constant {
                lerror!(&word.loc, "Constants cannot be redefined, got {}", word.text);
                linfo!(&c.loc, "first definition here"); 
                if crate::DEV_MODE {println!("{word:?}")}
                return Err(eyre!(""));
            }
            lerror!(&word.loc, "{typ:?} cannot replace constant, got {}", word.text);
            linfo!(&c.loc, "first definition here"); 
            if crate::DEV_MODE {println!("{word:?}")}
            return Err(eyre!(""));
        }

        Ok(true)
    }

    pub fn set_functions(&mut self, f: Functions) {
        self.functions = f;
    }
    pub fn set_constants(&mut self, f: Constants) {
        self.constants = f;
    }
    pub fn set_memories(&mut self, f: Memories) {
        self.memories = f;
    }

    pub fn get_functions(&mut self) -> Functions {
        self.functions.clone()
    }
    pub fn get_constants(&mut self) -> Constants {
        self.constants.clone()
    }
    pub fn get_memories(&mut self) -> Memories{
        self.memories.clone()
    }
}