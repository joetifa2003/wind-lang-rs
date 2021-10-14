#[derive(Debug, Clone)]
pub enum LiteralType {
    Nil,
    Number(f32),
    String(String),
    Bool(bool),
}
