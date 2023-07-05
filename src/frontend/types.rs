#[derive(Default)]
pub enum Type {
    #[default]
    ToInfer,
    Bool,
    Float,
    Integer,
    String,
    User(String),
    Char,
    Lambda,
}
