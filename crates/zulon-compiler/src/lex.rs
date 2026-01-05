use crate::diag::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Ident,
    Number,
    String,
    Symbol,
    Eof,
}

#[derive(Clone, Debug, Default)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
}
