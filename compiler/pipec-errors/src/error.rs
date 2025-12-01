#[allow(dead_code)]
pub trait Error {
    fn throw(&self) {
        print!("ERROR: {}", self.name());
        std::process::exit(1);
    }
    fn name(&self) -> &'static str;
    fn code(&self) -> u16;
}
