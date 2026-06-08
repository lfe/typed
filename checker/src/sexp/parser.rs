use crate::error::{ParseError, Result};
use crate::sexp::lexer::{Lexer, Token, TokenType};
use crate::sexp::types::*;
use std::path::Path;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[expect(
    dead_code,
    reason = "public API; parse_str/parse_file/parse used in tests"
)]
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse_str(input: &str) -> Result<SExp> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<SExp> {
        let content =
            std::fs::read_to_string(path.as_ref()).map_err(|e| ParseError::FileReadError {
                path: path.as_ref().to_path_buf(),
                source: e,
            })?;
        Self::parse_str(&content)
    }

    pub fn parse_all_str(input: &str) -> Result<Vec<SExp>> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let mut forms = Vec::new();
        while !parser.is_at_end() {
            forms.push(parser.parse_sexp()?);
        }
        Ok(forms)
    }

    pub fn parse_all_file<P: AsRef<Path>>(path: P) -> Result<Vec<SExp>> {
        let content =
            std::fs::read_to_string(path.as_ref()).map_err(|e| ParseError::FileReadError {
                path: path.as_ref().to_path_buf(),
                source: e,
            })?;
        Self::parse_all_str(&content)
    }

    pub fn parse(&mut self) -> Result<SExp> {
        if self.is_at_end() {
            return Err(ParseError::EmptyInput);
        }
        self.parse_sexp()
    }

    fn parse_sexp(&mut self) -> Result<SExp> {
        let token = self.current_token()?;
        match token.typ {
            TokenType::LParen => self.parse_list(),
            TokenType::Symbol => self.parse_symbol(),
            TokenType::Keyword => self.parse_keyword(),
            TokenType::String => self.parse_string(),
            TokenType::Number => self.parse_number(),
            TokenType::Nil => self.parse_nil(),
            TokenType::Quote => self.parse_quote(),
            TokenType::Backquote => self.parse_wrapper("backquote"),
            TokenType::Comma => self.parse_wrapper("comma"),
            TokenType::CommaAt => self.parse_wrapper("comma-at"),
            TokenType::HashParen => self.parse_tuple(),
            TokenType::Binary => self.parse_binary(),
            TokenType::Dot => Err(ParseError::UnexpectedToken {
                token: ".".to_string(),
                pos: token.pos,
            }),
            TokenType::RParen => Err(ParseError::UnexpectedCloseParen { pos: token.pos }),
            TokenType::Eof => Err(ParseError::EmptyInput),
        }
    }

    fn parse_list(&mut self) -> Result<SExp> {
        let open_paren = self.current_token()?.clone();
        let pos = open_paren.pos;
        self.advance();
        let mut elements = Vec::new();
        loop {
            if self.is_at_end() {
                return Err(ParseError::UnterminatedList { pos });
            }
            if self.check(&TokenType::RParen) {
                self.advance();
                break;
            }
            if self.check(&TokenType::Dot) {
                self.advance(); // consume dot
                let tail = self.parse_sexp()?;
                if !self.check(&TokenType::RParen) {
                    return Err(ParseError::Expected {
                        expected: ")".to_string(),
                        found: format!("{:?}", self.current_token()?.typ),
                        pos: self.current_token()?.pos,
                    });
                }
                self.advance(); // consume )
                elements.push(SExp::Symbol(Symbol::new(".", pos)));
                elements.push(tail);
                break;
            }
            elements.push(self.parse_sexp()?);
        }
        Ok(SExp::List(List::new(elements, pos)))
    }

    fn parse_symbol(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        self.advance();
        Ok(SExp::Symbol(Symbol::new(token.lexeme, token.pos)))
    }

    fn parse_keyword(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        self.advance();
        Ok(SExp::Keyword(Keyword::new(token.lexeme, token.pos)))
    }

    fn parse_string(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        self.advance();
        Ok(SExp::String(StringLit::new(token.lexeme, token.pos)))
    }

    fn parse_number(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        self.advance();
        Ok(SExp::Number(Number::new(token.lexeme, token.pos)))
    }

    fn parse_nil(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        self.advance();
        Ok(SExp::Nil(Nil::new(token.pos)))
    }

    fn parse_quote(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        let pos = token.pos;
        self.advance();
        let inner = self.parse_sexp()?;
        Ok(SExp::List(List::new(
            vec![SExp::Symbol(Symbol::new("quote", pos)), inner],
            pos,
        )))
    }

    fn parse_wrapper(&mut self, name: &str) -> Result<SExp> {
        let token = self.current_token()?.clone();
        let pos = token.pos;
        self.advance();
        let inner = self.parse_sexp()?;
        Ok(SExp::List(List::new(
            vec![SExp::Symbol(Symbol::new(name, pos)), inner],
            pos,
        )))
    }

    fn parse_tuple(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        let pos = token.pos;
        self.advance(); // consume HashParen token
        let mut elements = Vec::new();
        loop {
            if self.is_at_end() {
                return Err(ParseError::UnterminatedList { pos });
            }
            if self.check(&TokenType::RParen) {
                self.advance();
                break;
            }
            elements.push(self.parse_sexp()?);
        }
        Ok(SExp::Tuple(Tuple::new(elements, pos)))
    }

    fn parse_binary(&mut self) -> Result<SExp> {
        let token = self.current_token()?.clone();
        self.advance();
        Ok(SExp::List(List::new(
            vec![
                SExp::Symbol(Symbol::new("binary", token.pos)),
                SExp::String(StringLit::new(token.lexeme, token.pos)),
            ],
            token.pos,
        )))
    }

    fn current_token(&self) -> Result<&Token> {
        self.tokens.get(self.current).ok_or(ParseError::EmptyInput)
    }

    fn check(&self, typ: &TokenType) -> bool {
        matches!(self.current_token(), Ok(token) if token.typ == *typ)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_token(), Ok(token) if token.typ == TokenType::Eof)
            || self.current >= self.tokens.len()
    }
}
