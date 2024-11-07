use plugin_cargo::{logger, prelude::*};

#[macro_use]
extern crate tracing;

mod generate_rustdoc;

const DEPLOY: &str = "deploy";

fn main() -> Result<()> {
    logger::init();

    let arg = std::env::args().nth(1);
    let list_json = Utf8PathBuf::from(arg.as_deref().unwrap_or("list.json"));

    let list: Vec<String> = serde_json::from_slice(&std::fs::read(&list_json)?)?;

    let mut docs = generate_rustdoc::Docs::new();

    for user_repo in &list {
        if let Ok(manage) = generate_rustdoc::Manage::new(user_repo).inspect_err(inspect) {
            _ = manage.cargo_doc(&mut docs).inspect_err(inspect);
        }
    }

    _ = docs.finish().inspect_err(inspect);

    Ok(())
}

fn inspect(err: &eyre::Error) {
    error!(?err);
}
