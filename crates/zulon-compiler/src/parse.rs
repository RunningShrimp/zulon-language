use crate::diag::Diagnostic;
use crate::ids::FileId;

#[derive(Clone, Debug)]
pub struct AstFile {
    pub file: FileId,
    pub diagnostics: Vec<Diagnostic>,
}

impl AstFile {
    pub fn new(file: FileId) -> Self {
        Self {
            file,
            diagnostics: vec![],
        }
    }
}
