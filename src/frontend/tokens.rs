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

    #[regex("[!a-zA-Z-_]+[!a-zA-Z_0-9-_]*")]
    Identifier,

    #[regex(r"([a-zA-Z-_]+[a-zA-Z0-9-_]*)(::[a-zA-Z-_]+[a-zA-Z0-9-_]*)+")]
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

    #[regex(":[a-zA-Z_]+[a-zA-Z_0-9]*")]
    KeyWord,
}

impl From<std::ops::Range<usize>> for TextSpan {
    fn from(value: std::ops::Range<usize>) -> Self {
        Self {
            start : value.start,
            len: value.len()
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq,Default)]
pub struct TextSpan {
    pub start: usize,
    pub len: usize,
}

impl TextSpan {
    pub fn as_range(&self) -> std::ops::Range<usize> {
        self.start..self.start+self.len
    }
    pub fn new(start: usize, len: usize) -> Self {
        Self {start,len}
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParseText<'a> {
    pub base: &'a str,
    pub start: usize,
    pub len: usize,
}

impl<'a> ParseText<'a> {
    pub fn new(base: &'a str, range: std::ops::Range<usize> ) -> Self {
        Self { base, start: range.start, len: range.len() }
    }
}
impl <'a> ParseText<'a> {
    pub fn get_text(&self) -> &str {
        &self.base[self.as_range()]
    }

    pub fn as_range(&self) -> std::ops::Range<usize> {
        self.start..self.start+self.len
    }

}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Token<X: Clone> {
    pub kind: TokenKind,
    pub location: TextSpan,
    pub extra: X,
}

impl<X: Clone> Token<X> {
    pub fn text_span(a: &[Self]) -> std::ops::Range<usize>  {
        let start = a.first().unwrap().location.start;
        let end = a.last().unwrap().location.len + start;
        start..end
    }
}



