#[derive(Default, PartialEq, Clone, Debug   )]
pub enum Type {
    #[default]
    ToInfer,
    Bool,
    F32,
    F64,
    Integer,
    String,
    User(String),
    Char,
    Struct,
    Lambda,
    Void,
}

