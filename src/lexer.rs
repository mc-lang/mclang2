
use crate::{definitions::{Token, TokenType}, Args};

fn lex_word(s: String, tok_type: TokenType) -> (TokenType, String) {
    match s {
        s if s.parse::<u64>().is_ok() && tok_type == TokenType::Word => { // negative numbers not yet implemented
            (TokenType::Int, s)
        },
        s if tok_type == TokenType::Word => {
            (TokenType::Word, s)
        },
        s if tok_type == TokenType::String => {
            (TokenType::String, s)
        }
        s if tok_type == TokenType::CString => {
            (TokenType::CString, s)
        }
        s if tok_type == TokenType::Char => {
            (TokenType::Char, s)
        }
        _ => unreachable!()
    }
}

pub fn find_col<F>(text: &str, mut col: usize, predicate: F) -> usize where F: Fn(char, char) -> bool {
    let mut last = '\0';
    while col < text.len() && !predicate(text.chars().nth(col).unwrap(), last) {
        last = text.chars().nth(col).unwrap();
        col += 1;
    }

    col
}


// TODO: Implement multiline strings
fn lex_line(text: &str) -> Vec<(usize, String, TokenType)> {
    let mut tokens: Vec<(usize, String, TokenType)> = Vec::new();

    let mut col = find_col(text, 0, |x, _| !x.is_whitespace());
    let mut col_end: usize = 0;
    while col_end < text.to_string().len() {
        if (text.len() - col) < 1 {
            return tokens;
        }
        if &text[col..=col] == "\"" {
            col_end = find_col(text, col + 1, |x, x2| x == '"' && x2 != '\\');
            let t = &text[(col + 1)..col_end];
            let t = t.replace("\\n", "\n")
                                .replace("\\t", "\t")
                                .replace("\\r", "\r")
                                .replace("\\\'", "\'")
                                .replace("\\\"", "\"")
                                .replace("\\0", "\0");
            if !t.is_empty() {
                tokens.push((col, t.to_string(), TokenType::String));
            }
            col = find_col(text, col_end + 1, |x, _| !x.is_whitespace());

        } else if &text[col..=col] == "'"{
            col_end = find_col(text, col + 1, |x, x2| x == '\'' && x2 != '\\');
            let t = &text[(col + 1)..col_end];
            let t = t.replace("\\n", "\n")
                                .replace("\\t", "\t")
                                .replace("\\r", "\r")
                                .replace("\\\'", "\'")
                                .replace("\\\"", "\"")
                                .replace("\\0", "\0");

            
            if !t.is_empty() {
                tokens.push((col, t.to_string(), TokenType::Char));
            }
            col = find_col(text, col_end + 1, |x, _| !x.is_whitespace());

        } else {

            if &text[col..=col] == "c" && text.len() - 1 + col > 0 && &text[col+1..=col+1] == "\"" {
                col_end = find_col(text, col + 2, |x, x2| x == '"' && x2 != '\\');
                let t = &text[(col + 2)..col_end];
                let mut t = t.replace("\\n", "\n")
                                    .replace("\\t", "\t")
                                    .replace("\\r", "\r")
                                    .replace("\\\'", "\'")
                                    .replace("\\\"", "\"")
                                    .replace("\\0", "\0");
                
                if !t.is_empty() {
                    t.push('\0');
                    tokens.push((col, t.to_string(), TokenType::CString));
                }
                col = find_col(text, col_end + 1, |x, _| !x.is_whitespace());
                
            } else {
                col_end = find_col(text, col, |x, _| x.is_whitespace());
                let t = &text[col..col_end];
                
                if t == "//" {
                    return tokens;
                }
                
                if !t.is_empty() {
                    tokens.push((col, t.to_string(), TokenType::Word));
                }
                col = find_col(text, col_end, |x, _| !x.is_whitespace());
            }
        }
    }
    tokens
}

pub fn lex(code: &str, file: &str, _args: &Args) -> Vec<Token> {
    let lines: Vec<(usize, &str)> = code
        .split(['\n', '\r'])
        .enumerate()
        .collect();
    
    let lines: Vec<(usize, String)> = lines.iter().map(|i| (i.0, i.1.to_string())).collect();

    let mut tokens: Vec<Token> = Vec::new();

    for (row, line) in lines {
        let lt = lex_line(&line);
        for (col, tok, tok_type) in lt {
            let (tok_type, tok) = lex_word(tok, tok_type);
            let t = Token{
                file: file.to_string(),
                line: row + 1,
                col,
                text: tok,
                typ: tok_type,
                value: None,
                addr: None,
                op_typ: crate::definitions::OpType::Instruction(crate::definitions::InstructionType::None)
            };
            tokens.push(t);
        }
    }
    // println!("{}", tokens.len());

    // for token in tokens.clone() {
    //     println!("tok: {:?}", token.text);
    // }
    

    tokens
}