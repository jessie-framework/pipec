use pipec_args::{Args, Parser};
use pipec_ast::{ASTFileReader, RecursiveGuard};
use pipec_semantic_analysis::semantic_analyzer::GlobalSymbolTree;

use std::sync::Arc;
/// This is where the compiler code begins.
pub fn run_compiler() {
    let args = Args::parse();
    let cache_dir = Arc::new(args.cache);

    let mut guard = RecursiveGuard::default();
    let (mut reader, reader_link) = ASTFileReader::new(&args.file, cache_dir.clone()).unwrap();
    let mut hirtree = reader.generate_hir(&mut guard, cache_dir.clone());
    reader.upload_to_cache(reader_link, cache_dir.clone());

    let mut gst = GlobalSymbolTree::default();
    gst.gen_symbols(&mut hirtree);
    println!("{:#?}", gst);
}
