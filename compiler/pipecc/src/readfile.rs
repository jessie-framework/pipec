use pipec_ast::generate_ast;
use pipec_globals::GLOBALS;
use std::{fs::File, io::Read};
pub fn read_file() -> std::io::Result<()> {
    let mut file = File::open(&*GLOBALS.file)?;
    let mut buf = String::with_capacity(1000);
    file.read_to_string(&mut buf)?;
    generate_ast(&buf)?;
    Ok(())
}
