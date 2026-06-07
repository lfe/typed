use crate::error::Position;

#[derive(Debug, Clone, PartialEq)]
pub enum SExp {
    Symbol(Symbol),
    Keyword(Keyword),
    String(StringLit),
    Number(Number),
    Nil(Nil),
    List(List),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub value: String,
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Keyword {
    pub name: String,
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLit {
    pub value: String,
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Number {
    pub value: String,
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Nil {
    pub pos: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub elements: Vec<SExp>,
    pub pos: Position,
}

impl Symbol {
    pub fn new(value: impl Into<String>, pos: Position) -> Self {
        Self {
            value: value.into(),
            pos,
        }
    }
}

impl Keyword {
    pub fn new(name: impl Into<String>, pos: Position) -> Self {
        Self {
            name: name.into(),
            pos,
        }
    }
}

impl StringLit {
    pub fn new(value: impl Into<String>, pos: Position) -> Self {
        Self {
            value: value.into(),
            pos,
        }
    }
}

impl Number {
    pub fn new(value: impl Into<String>, pos: Position) -> Self {
        Self {
            value: value.into(),
            pos,
        }
    }
}

impl Nil {
    pub fn new(pos: Position) -> Self {
        Self { pos }
    }
}

impl List {
    pub fn new(elements: Vec<SExp>, pos: Position) -> Self {
        Self { elements, pos }
    }
}

pub trait HasPosition {
    fn position(&self) -> Position;
}

impl HasPosition for SExp {
    fn position(&self) -> Position {
        match self {
            SExp::Symbol(s) => s.pos,
            SExp::Keyword(k) => k.pos,
            SExp::String(s) => s.pos,
            SExp::Number(n) => n.pos,
            SExp::Nil(n) => n.pos,
            SExp::List(l) => l.pos,
        }
    }
}
