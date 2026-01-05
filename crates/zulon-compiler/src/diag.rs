use crate::ids::FileId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    Parse,
    Resolve,
    Type,
    Effect,
    Capability,
    Codegen,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: ErrorCode,
    pub message: String,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            span: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}
