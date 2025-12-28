#[cfg(test)]
mod ast;

#[macro_export]
macro_rules! test_file_generation {
    ($filename:  literal) => {
        use pipec_ast::tokenizer::Tokenizer;
        use pipec_ast::{RecursiveGuard, ast::ASTGenerator};
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("tests")
            .to_path_buf();
        let mut tokenizer = Tokenizer::new(include_str!($filename));
        let mut tree = tokenizer.tree();
        let filecontents = std::fs::read_to_string(&path).unwrap();
        let mut guard = RecursiveGuard::new();
        let hirtree = ASTGenerator::new(&filecontents, &mut tree, path, &mut guard);
        hirtree.tree();
    };
}
