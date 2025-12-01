use clap::Parser;
use pipec_args::Args;
use std::sync::LazyLock;

pub static GLOBALS: LazyLock<Args> = LazyLock::new(Args::parse);
