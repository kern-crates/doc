use plugin::{logger, prelude::*, repos};

#[macro_use]
extern crate tracing;

mod generate_rustdoc;

const DEPLOY: &str = "deploy";

fn main() -> Result<()> {
    logger::init();

    let list = repos()?;

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
