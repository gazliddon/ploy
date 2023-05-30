use logos::Logos;

#[derive(Logos, Copy, Clone, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t\f\n]+")]
pub enum TokenKind {
    Error,

    #[regex("[0-9][0-9_]*")]
    DecNumber,

    #[regex(r"(0[xX]|\$)([a-fA-F0-9][a-fA-F0-9_]*)")]
    HexNumber,

    #[regex("(0[bB])|%[0-1][0-1_]*")]
    BinNumber,

    #[token("[")]
    OpenSquareBracket,

    #[token("]")]
    CloseSquareBracket,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("(")]
    OpenBracket,

    #[token(")")]
    CloseBracket,

    #[token("*")]
    Star,

    #[token("+")]
    Plus,

    #[token("/")]
    Slash,

    #[token("\\")]
    BackSlash,

    #[token("-")]
    Minus,

    #[regex(";.*\n")]
    Comment,

    #[token("&")]
    Ampersand,

    #[regex("[a-zA-Z_]+[a-zA-Z_0-9]*")]
    Identifier,

    #[regex(r"([a-zA-Z_]+[a-zA-Z_0-9]*)(::[a-zA-Z_]+[a-zA-Z_0-9]*)+")]
    FqnIdentifier,

    #[token("=")]
    Equals,

    #[token("==")]
    DoubleEqual,

    #[token("!=")]
    NotEqual,

    #[regex("'.'")]
    Char,

    #[regex(r#""([^"\\]|\\t|\\u|\\n|\\")*""#)]
    QuotedString,

    #[token(",")]
    Comma,

    #[token(">")]
    GreaterThan,

    #[token("<")]
    LessThan,

    #[token("|")]
    Bar,

    #[token("^")]
    Caret,

    #[token("%")]
    Percentage,

    #[token("#")]
    Hash,

    #[token("'")]
    Quote,

    #[token("`")]
    BackTick,

    #[token("false")]
    False,

    #[token("true")]
    True,

    #[token(":")]
    Colon,
}

impl From<std::ops::Range<usize>> for Location {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self {
            start : value.start,
            len: value.len()
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Location {
    pub start: usize,
    pub len: usize,
}
impl Location {
    pub fn as_range(&self) -> std::ops::Range<usize> {
        self.start..self.start+self.len
    }
}

impl Default for Location {
    fn default() -> Self {
        Self {
            start: 0,
            len: 0,
        }
    }

}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParseText<'a> {
    txt: &'a str,
}


impl<'a> ParseText<'a> {
    pub fn new(txt: &'a str) -> Self {
        Self { txt }
    }
}
#[derive(Clone, Debug, Copy, PartialEq)]
pub struct FragementLocation<X: Clone> {
    pub loc: Location,
    pub extra: X,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Token<X: Clone> {
    pub kind: TokenKind,
    pub location: FragementLocation<X>,
}

fn to_tokens_kinds(program_txt: &str) -> Vec<(TokenKind, std::ops::Range<usize>)> {
    TokenKind::lexer(program_txt)
        .spanned()
        .map(|(tok_res, pos)| match tok_res {
            Ok(kind) => (kind, pos),
            Err(_) => (TokenKind::Error, pos),
        })
        .collect()
}

