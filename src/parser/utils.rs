use anyhow::{bail, Result};

use crate::types::token::{Token, TokenType};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PeekResult<T> {
    Correct(T),
    Wrong(T),
    None
}

impl<T> PeekResult<T> {
    pub fn correct(&self) -> bool{
        match self {
            PeekResult::Correct(_) => true,
            _ => false
        }
    }
    #[allow(dead_code)]
    pub fn wrong(&self) -> bool{
        match self {
            PeekResult::Wrong(_) => true,
            _ => false
        }
    }
    
    #[allow(dead_code)]
    pub fn none(&self) -> bool{
        match self {
            PeekResult::None => true,
            _ => false
        }
    }
}

pub fn cmp(lhs: &TokenType, rhs: &TokenType) -> bool {
    match (lhs, rhs) {
        (TokenType::Keyword(lhs), TokenType::Keyword(rhs)) => {
            std::mem::discriminant(lhs) == std::mem::discriminant(rhs)
        },
        (TokenType::Instruction(lhs), TokenType::Instruction(rhs)) => {
            std::mem::discriminant(lhs) == std::mem::discriminant(rhs)
        },
        (TokenType::Unknown(_), TokenType::Unknown(_)) => true,
        _ => false
    }
}

pub fn peek_check_multiple(tokens: &Vec<Token>, typs: Vec<TokenType>) -> PeekResult<&Token>{
    let t = tokens.last();
    
    if let Some(t) = t {
        for tt in typs {
            if cmp(&t.typ, &tt) {
                return PeekResult::Correct(t);
            }
        }
        PeekResult::Wrong(t)
    } else {
        PeekResult::None
    }
}

pub fn peek_check(tokens: &Vec<Token>, typ: TokenType) -> PeekResult<&Token> {
    let t = tokens.last();

    match t {
        Some(t) => {
            //? Source: https://doc.rust-lang.org/std/mem/fn.discriminant.html
            if cmp(&t.typ, &typ) {
                PeekResult::Correct(t)
            } else {
                PeekResult::Wrong(t)
            }
        },
        None => {
            PeekResult::None
        }
    }
}

pub fn expect(tokens: &mut Vec<Token>, typ: TokenType) -> Result<Token> {
    let t = tokens.pop();

    match t {
        Some(t) => {
            //? Source: https://doc.rust-lang.org/std/mem/fn.discriminant.html
            if std::mem::discriminant(&t.typ) != std::mem::discriminant(&typ) {
                error!("Expected {:?}, but got {:?}", typ, t.typ);
                bail!("")
            }
            Ok(t)
        },
        None => {
            error!("Expected {:?}, but found nothing", typ);
            bail!("")
        }
    }

}