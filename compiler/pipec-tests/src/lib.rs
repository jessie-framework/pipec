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

        let mut arena = Arena::new(Size::Megs(10));
        let mut loader = FileLoader::default();
        let file_id = loader.open(&file_dir, &mut arena).unwrap();

        let file_contents = include_str!($filename);
        let mut tokentree = Tokenizer::new(&file_contents).tree();
        let mut guard = RecursiveGuard::default();

        #[allow(unused_variables)]
        let ast_tree = ASTGenerator::new(
            file_id,
            &mut tokentree,
            file_dir,
            &mut arena,
            &mut guard,
            &mut loader,
        )
        .tree();
    };

    ($filename : literal,scope $scope:ident) => {
        use pipec_arena::{Arena, Size};
        use pipec_ast::{RecursiveGuard, ast::ASTGenerator, tokenizer::Tokenizer};
        use pipec_file_loader::FileLoader;
        use pipec_gst::GlobalSymbolTree;

        let file_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(file!())
            .parent()
            .unwrap()
            .join($filename);

        let mut arena = Arena::new(Size::Megs(10));
        let mut loader = FileLoader::default();
        let file_id = loader.open(&file_dir, &mut arena).unwrap();

        let file_contents = include_str!($filename);
        let mut tokentree = Tokenizer::new(&file_contents).tree();
        let mut guard = RecursiveGuard::default();

        let ast_tree = ASTGenerator::new(
            file_id,
            &mut tokentree,
            file_dir,
            &mut arena,
            &mut guard,
            &mut loader,
        )
        .tree();

        let mut gst = GlobalSymbolTree::new(&mut arena, &mut loader, ast_tree);
        let $scope = gst.generate();
    };
}
