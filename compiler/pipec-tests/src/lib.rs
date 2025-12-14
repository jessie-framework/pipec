#[cfg(test)]
mod ast;

#[macro_export]
macro_rules! test_file_generation {
    ($filename:  literal) => {
        use pipec_ast::tokenizer::Tokenizer;
        use pipec_ast::{RecursiveGuard, hir::HIRGenerator};
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("tests")
            .to_path_buf();
        let tokenizer = Tokenizer::new(include_str!($filename));
        let mut guard = RecursiveGuard::default();
        let mut tree = tokenizer.tree();
        let hirtree = HIRGenerator::new(&mut tree, path, &mut guard);
        hirtree.tree();
    };
}
