use plugin_cargo::{logger, prelude::*};

#[macro_use]
extern crate tracing;

mod repo;
mod submodule_add;

mod manage;

const REPOS: &str = "repos";

fn main() -> Result<()> {
    logger::init();

    let arg = std::env::args().nth(1);
    let list_json = Utf8PathBuf::from(arg.as_deref().unwrap_or("list.json"));

    let list: Vec<String> = serde_json::from_slice(&std::fs::read(&list_json)?)?;

    let mut manage = manage::Manage::new()?;
    for user_repo in &list {
        manage.process(user_repo)?;
    }

    Ok(())
}
