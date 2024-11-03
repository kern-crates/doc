use crate::{manage::Local, REPOS};
use duct::cmd;
use plugin_cargo::prelude::*;

fn git_link(user: &str, repo: &str) -> String {
    format!("https://github.com/{user}/{repo}.git")
}

pub fn submodule_add(user_repo: &str, set: &mut Local) -> Result<()> {
    let split: Vec<_> = user_repo.split("/").collect();
    let (user, repo) = (&split[0], &split[1]);
    let link = git_link(user, repo);

    if set.contains_key(user_repo) {
        return Ok(());
    }

    let path = Utf8PathBuf::from_iter([REPOS, user, repo]);
    cmd!("git", "submodule", "add", link, path).run()?;

    cmd!("git", "commit", "-m", format!("submodule: add {user_repo}")).run()?;

    Ok(())
}

pub fn submodule_remove(path: &Utf8Path) -> Result<()> {
    // git submodule deinit <path>
    // git rm <path>
    // rm -rf .git/modules/<path>
    // rm -rf <path>

    let _span = error_span!("submodule_remove", %path).entered();

    cmd!("git", "submodule", "deinit", path).run()?;
    cmd!("git", "rm", path).run()?;

    let mut git_module = Utf8PathBuf::from_iter([".git", "modules"]);
    git_module.push(path);
    cmd!("rm", "-rf", git_module).run()?;

    cmd!("rm", "-rf", path).run()?;

    let msg = format!("submodule: remove {path}");
    cmd!("git", "commit", "-m", msg).run()?;

    Ok(())
}

#[test]
fn add() -> Result<()> {
    // submodule_add("os-checker/os-checker-test-suite", &mut Default::default())
    Ok(())
}

#[test]
#[ignore = "should be confirmed to call this"]
fn remove() -> Result<()> {
    submodule_remove("repos/os-checker/plugin-cargo".into())?;
    Ok(())
}
