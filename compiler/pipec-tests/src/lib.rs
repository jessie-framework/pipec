#[cfg(test)]
mod ast;

#[macro_export]
macro_rules! test_file_generation {
    ($filename:  literal) => {
        use pipec_arena::{Arena, Size};
        use pipec_ast::tokenizer::Tokenizer;
        use pipec_ast::{RecursiveGuard, ast::ASTGenerator, ast::ASTNode};

        let path = std::path::Path::new(concat!(
            ".",
            concat!(concat!(env!("CARGO_MANIFEST_DIR"), "src/tests/")),
            $filename
        ))
        .to_path_buf(); // FIXME : i have zero idea why this makes the tests work but if you do it otherwise it stack overflows even though the binaries have no issue??
        println!("{:#?}", &path);

        let filecontents = include_str!($filename);
        let tokenizer = Tokenizer::new(filecontents);
        let mut tree = tokenizer.tree();
        let mut arena = Arena::new(Size::Megs(1));
        let mut guard = RecursiveGuard::new();
        let mut hirtree = ASTGenerator::new(filecontents, &mut tree, &mut arena, path, &mut guard);
        loop {
            let next = hirtree.parse_value();
            if matches!(next, ASTNode::EOF) {
                break;
            }
        }
    };
}
