use crate::REPOS;
use duct::cmd;
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
    cmd!("git", "submodule", "add", link, path).run()?;

    Ok(())
}

fn submodule_remove(path: &str) -> Result<()> {
    // git submodule deinit <path>
    // git rm <path>
    // rm -rf .git/modules/<path>

    let _span = error_span!("submodule_remove", path).entered();

    cmd!("git", "submodule", "deinit", path).run()?;
    cmd!("git", "rm", path).run()?;
    cmd!("rm", "-rf", path).run()?;

    Ok(())
}

#[test]
fn add_plugin_cargo() -> Result<()> {
    submodule_add("os-checker/plugin-cargo", &mut Default::default())
}

#[test]
fn remove() -> Result<()> {
    submodule_remove("repos/os-checker")?;
    submodule_remove("repos/os-checker-test-suite")?;
    Ok(())
}
