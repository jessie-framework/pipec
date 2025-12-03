use pipec_ast::generate_ast;
pub fn read_file() -> std::io::Result<()> {
    generate_ast()?;
    Ok(())
}
