use plugin_cargo::{logger, prelude::*};

mod repo;

fn main() -> Result<()> {
    logger::init();

    Ok(())
}
