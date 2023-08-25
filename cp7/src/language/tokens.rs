use std::{collections::HashMap, fmt};

use enum_variant_type::EnumVariantType;

#[derive(Clone, Debug, Eq, PartialEq, Hash, EnumVariantType)]
#[evt(module = "token")]
pub enum Token {
    CurlyBracketOpen,
    CurlyBracketClose,
    Number(i64),
    Symbol(String),
    Word(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::CurlyBracketOpen => write!(f, "{{"),
            Token::CurlyBracketClose => write!(f, "}}"),
            Token::Number(number) => write!(f, "{number}"),
            Token::Symbol(symbol) => write!(f, ":{symbol}"),
            Token::Word(word) => write!(f, "{word}"),
        }
    }
}

#[derive(Debug)]
pub struct Tokens {
    pub by_address: HashMap<TokenAddress, Token>,

    pub leftmost: Option<TokenAddress>,
    pub rightmost: Option<TokenAddress>,

    pub right_neighbors: HashMap<TokenAddress, TokenAddress>,
    pub left_neighbors: HashMap<TokenAddress, TokenAddress>,

    pub left_to_right: HashMap<RightNeighborAddress, AddressedToken>,
    pub right_to_left: HashMap<LeftNeighborAddress, AddressedToken>,
}

impl Tokens {
    pub fn left_to_right(&self) -> TokensLeftToRight {
        TokensLeftToRight {
            next: self.leftmost,
            tokens: self,
        }
    }

    pub fn right_to_left(&self) -> TokensRightToLeft {
        TokensRightToLeft {
            current: self.rightmost,
            tokens: self,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TokenAddress {
    pub as_left_neighbor: LeftNeighborAddress,
    pub as_right_neighbor: RightNeighborAddress,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AddressedToken {
    pub token: TokenAddress,
    pub left_neighbor: Option<LeftNeighborAddress>,
    pub right_neighbor: Option<RightNeighborAddress>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct RightNeighborAddress {
    pub hash: blake3::Hash,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LeftNeighborAddress {
    pub hash: blake3::Hash,
}

pub struct TokensLeftToRight<'r> {
    next: Option<TokenAddress>,
    tokens: &'r Tokens,
}

impl<'r> Iterator for TokensLeftToRight<'r> {
    type Item = TokenAddress;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next?;
        self.next = self.tokens.right_neighbors.get(&current).copied();
        Some(current)
    }
}

pub struct TokensRightToLeft<'r> {
    current: Option<TokenAddress>,
    tokens: &'r Tokens,
}

impl<'r> Iterator for TokensRightToLeft<'r> {
    type Item = TokenAddress;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = self.tokens.left_neighbors.get(&current).copied();
        Some(current)
    }
}
