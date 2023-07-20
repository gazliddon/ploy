use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum IrErrorKind  {
}

pub struct IrError {
    kind: IrErrorKind,
}
