use pipec_arena::{Arena, Size};
use pipec_args::{Args, Parser};
use pipec_ast::{RecursiveGuard, ast::ASTGenerator, tokenizer::Tokenizer};
use pipec_file_loader::*;

/// This is where the compiler code begins.
pub fn run_compiler() {
    let args = Args::parse();
    println!("{:#?}", &args.file);

    let mut loader = FileLoader::default();
    let file_id = loader.open(&args.file).unwrap();
    let file_contents = loader.load(file_id);

    let mut tokentree = Tokenizer::new(&file_contents).tree();
    let mut arena = Arena::new(Size::Gigs(1));
    let mut guard = RecursiveGuard::default();

    let mut ast_generator = ASTGenerator::new(
        file_id,
        &mut tokentree,
        &mut arena,
        args.file,
        &mut guard,
        &mut loader,
    );
    loop {
        let next = ast_generator.parse_value();
        if matches!(next, pipec_ast::ast::ASTNode::EOF) {
            break;
        }
    }
    println!("success!");
}
