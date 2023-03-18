
use crate::constants::{Token, TokenType};
use color_eyre::Result;

fn lex_word(s: String) -> (TokenType, String) {
    match s {
        s if s.parse::<u64>().is_ok() => { // negative numbers not yet implemented
            return (TokenType::Int, s);
        },
        s => {
            return(TokenType::Word, s);
        }
    }
}

pub fn find_col<F>(text: String, mut col: u32, predicate: F) -> Result<u32> where F: Fn(char) -> bool {
    while (col as usize) < text.len() && !predicate(text.chars().nth(col as usize).unwrap()) {
        col += 1;
    }

    Ok(col)
}



fn lex_line(text: String) -> Result<Vec<(u32, String)>> {
    let mut tokens: Vec<(u32, String)> = Vec::new();

    let mut col = find_col(text.clone(), 0, |x| !x.is_whitespace())?;
    let mut col_end: u32 = 0;
    while col_end < text.clone().len() as u32 {
        col_end = find_col(text.clone(), col, |x| x.is_whitespace())?;
        let t = &text[(col as usize)..((col_end as usize))];

        if t == "//" {
            return Ok(tokens);
        }

        if !t.is_empty() {
            tokens.push((col, t.to_string()));
        }
        col = find_col(text.clone(), col_end, |x| !x.is_whitespace())?;
    }

    Ok(tokens)
}

pub fn lex(code: String, file: &String) -> Result<Vec<Token>> {
    let lines: Vec<(usize, &str)> = code
        .split(['\n', '\r'])
        .enumerate()
        .collect();
    
    let lines: Vec<(u32, String)> = lines.iter().map(|i| (i.0 as u32, i.1.to_string())).collect();

    let mut tokens: Vec<Token> = Vec::new();

    for (row, line) in lines {
        let lt = lex_line(line)?;
        for (col, tok) in lt {
            let (tok_type, tok) = lex_word(tok);
            let t = Token{
                file: file.clone(),
                line: row + 1,
                col: col,
                text: tok,
                typ: tok_type
            };
            tokens.push(t);
        }
    }
    // println!("{}", tokens.len());

    // for token in tokens.clone() {
    //     println!("tok: {:?}", token.text);
    // }
    Ok(tokens)
}