#[cfg(test)]
mod ast;

#[macro_export]
macro_rules! test_file_generation {
    ($filename:  literal) => {
        use pipec_arena::{Arena, Size};
        use pipec_ast::{RecursiveGuard, ast::ASTGenerator, tokenizer::Tokenizer};

        let file_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(file!())
            .parent()
            .unwrap()
            .join($filename);
        let c = file_dir.clone();

        let file_contents = include_str!($filename);
        let mut tokentree = Tokenizer::new(&file_contents).tree();
        let mut arena = Arena::new(Size::Megs(10));
        let mut guard = RecursiveGuard::new();

        let mut ast_generator = ASTGenerator::new(
            file_contents,
            &mut tokentree,
            &mut arena,
            file_dir,
            &mut guard,
        );

        loop {
            let next = ast_generator.parse_value();
            if matches!(next, pipec_ast::ast::ASTNode::EOF) {
                break;
            }
        }
        println!("{c:#?}");
    };
}
