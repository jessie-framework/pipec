use pipec_arena::{Arena, Size};
use pipec_args::{Args, Parser};
use pipec_ast::{RecursiveGuard, ast::ASTGenerator, tokenizer::Tokenizer};
use pipec_file_loader::*;
use pipec_gst::GlobalSymbolTree;

/// This is where the compiler code begins.
pub fn run_compiler() {
    let args = Args::parse();
    println!("{:#?}", &args.file);

    let mut arena = Arena::new(Size::Gigs(1));
    let mut loader = FileLoader::default();
    let file_id = loader.open(&args.file, &mut arena).unwrap();
    let file_contents_slice = loader.load(file_id);
    let file_source = arena.take_str_slice(file_contents_slice);

    let mut tokentree = Tokenizer::new(file_source).tree();
    let mut guard = RecursiveGuard::default();

    let ast_generator = ASTGenerator::new(
        file_id,
        &mut tokentree,
        &mut arena,
        args.file,
        &mut guard,
        &mut loader,
    );
    let ast_tree = ast_generator.tree();

    let mut gst = GlobalSymbolTree::new(&mut arena, &mut loader, ast_tree);
    gst.generate();
    println!("{:#?}", gst.map);
}
