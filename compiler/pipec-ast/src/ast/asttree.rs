use super::ASTNode;
use pipec_file_loader::FileId;

#[derive(Debug, Clone)]
pub struct ASTTree {
    pub stream: Vec<ASTNode>,
    pub id: FileId,
}

impl ASTTree {
    pub fn new(stream: Vec<ASTNode>, id: FileId) -> Self {
        Self { stream, id }
    }
}
