use plugin_cargo::{logger, prelude::*};

#[macro_use]
extern crate tracing;

mod repo;
mod submodule_add;

const REPOS: &str = "repos";

fn main() -> Result<()> {
    logger::init();

    Ok(())
}
