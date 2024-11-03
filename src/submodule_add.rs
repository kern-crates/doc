use crate::REPOS;
use indexmap::IndexSet;
use plugin_cargo::prelude::*;

fn git_link(user: &str, repo: &str) -> String {
    format!("https://github.com/{user}/{repo}.git")
}

fn submodule_add(user_repo: &str, set: &mut IndexSet<String>) -> Result<()> {
    let _span = error_span!("submodule_add", user_repo).entered();

    let split: Vec<_> = user_repo.split("/").collect();
    let (user, repo) = (&split[0], &split[1]);
    let link = git_link(user, repo);

    if set.contains(&link) {
        return Ok(());
    }

    let path = Utf8PathBuf::from_iter([REPOS, user, repo]);
    duct::cmd!("git", "submodule", "add", link, path).run()?;

    Ok(())
}

#[test]
fn add_plugin_cargo() -> Result<()> {
    submodule_add("os-checker/plugin-cargo", &mut Default::default())
}
