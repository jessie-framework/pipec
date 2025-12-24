use pipec_args::{Args, Parser};
use pipec_ast::{ASTFileReader, RecursiveGuard};
use pipec_semantic_analysis::semantic_analyzer::GlobalSymbolTree;

/// This is where the compiler code begins.
pub fn run_compiler() {
    let args = Args::parse();

    let mut guard = RecursiveGuard::default();
    let mut reader = ASTFileReader::new(&args.file).unwrap();
    let mut hirtree = reader.generate_hir(&mut guard);

    let mut gst = GlobalSymbolTree::default();
    gst.gen_symbols(&mut hirtree);
    println!("{:#?}", gst);
}
