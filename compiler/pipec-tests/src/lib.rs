#[cfg(test)]
mod ast;

#[macro_export]
macro_rules! test_file_generation {
    ($filename:  literal) => {
        use pipec_arena::{Arena, Size};
        use pipec_ast::{RecursiveGuard, ast::ASTGenerator, tokenizer::Tokenizer};
        use pipec_file_loader::FileLoader;

        let file_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(file!())
            .parent()
            .unwrap()
            .join($filename);
        let c = file_dir.clone();

        let mut arena = Arena::new(Size::Megs(10));
        let mut loader = FileLoader::default();
        let file_id = loader.open(&file_dir, &mut arena).unwrap();

        let file_contents = include_str!($filename);
        let mut tokentree = Tokenizer::new(&file_contents).tree();
        let mut guard = RecursiveGuard::default();

        let mut ast_generator = ASTGenerator::new(
            file_id,
            &mut tokentree,
            &mut arena,
            file_dir,
            &mut guard,
            &mut loader,
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
