#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    CurlyBraceOpen,
    CurlyBraceClose,
    ParenOpen,
    ParenClose,
    SemiColon,
    Return,
    Int,
    Ident(String),
    Integer(usize),
    Unknown(String),
    Negation,
    BitComplement,
    LogicalNegation,
    Add,
    Mul,
    Div,
    And,
    Or,
    Equal,
    NotEqual,
    LT,
    LTE,
    GT,
    GTE,
    Modulo,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    Assign,
    If,
    Else,
    Colon,
    QMark,
    EOF,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

pub struct Lexer {
    pub line: usize,
    pub offset: usize,
}
