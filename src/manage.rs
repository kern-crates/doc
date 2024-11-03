use crate::{repo::SelfRepo, submodule_add::submodule_add};
use duct::cmd;
use indexmap::{indexmap, IndexMap, IndexSet};
use plugin_cargo::{prelude::*, repo::Repo};

/// This map is a collection of user_repo string, and
/// must ensure there exist a local repo.
pub type Local = IndexMap<String, Repo>;

pub struct Manage {
    self_repo: SelfRepo,
    local: Local,
}

impl Manage {
    pub fn new() -> Result<Manage> {
        let self_repo = SelfRepo::new()?;
        let local = self_repo
            .submodules()
            .iter()
            .map(|m| {
                let user_repo = m.user_repo.clone();
                let repo = m.repo_metadata()?;
                Ok((user_repo, repo))
            })
            .collect::<Result<_>>()?;
        Ok(Manage { self_repo, local })
    }

    pub fn update_submodules(&mut self, v_user_repo: &[String]) -> Result<()> {
        // add and commit a new submodule if applies
        for user_repo in v_user_repo {
            let _span = error_span!("update_submodules", user_repo).entered();
            submodule_add(user_repo, &mut self.local)?;
        }

        // update submodules
        self.self_repo.update_submodules()?;

        for submodule in self.self_repo.submodules() {
            let key = &submodule.user_repo;
            if !self.local.contains_key(key) {
                info!(key, "append a new user_repo");
                let repo = submodule.repo_metadata()?;
                self.local.insert(key.clone(), repo);
            }
        }

        Ok(())
    }

    pub fn cargo_doc(&self) -> Result<UserRepoPkgCrate> {
        let mut docs = UserRepoPkgCrate::with_capacity(128);

        // cargo doc --document-private-items --workspace --no-deps
        for (user_repo, data) in &self.local {
            let _span = error_span!("cargo_doc", user_repo).entered();

            for (ws_dir, meta) in &data.workspaces {
                // don't early return due to all kinds of errors, just log them
                let expr = &cmd!(
                    "cargo",
                    "doc",
                    "--document-private-items",
                    "--workspace",
                    "--no-deps"
                );
                if let Err(err) = expr.dir(ws_dir).run() {
                    error!(
                        ?err,
                        "cargo doc exits with failure, \
                         but maybe useful artifacts are still generated."
                    );
                };

                // crate names are package names with - converted to _
                let pkg_crate_names: IndexMap<_, _> = meta
                    .packages
                    .iter()
                    .map(|p| (p.name.clone(), p.name.replace("-", "_")))
                    .collect();
                let mut doc_path = IndexSet::with_capacity(pkg_crate_names.len());

                let doc_dir = meta.target_directory.join("doc");
                for entry in doc_dir.read_dir_utf8()? {
                    let entry = entry?;
                    doc_path.insert(entry.file_name().to_owned());
                }

                // check missing docs
                for krate in pkg_crate_names.values() {
                    if !doc_path.contains(krate) {
                        error!("crate `{krate}` does not generate rustdoc");
                    }
                }

                match docs.get_mut(&data.user) {
                    Some(repo) => match repo.get_mut(&data.repo) {
                        Some(pkgs) => pkgs.extend(pkg_crate_names),
                        None => _ = repo.insert(data.repo.clone(), pkg_crate_names),
                    },
                    None => {
                        let map = indexmap! { data.repo.clone() =>  pkg_crate_names };
                        docs.insert(data.user.clone(), map);
                    }
                }
            }
        }

        Ok(docs)
    }
}

pub type UserRepoPkgCrate = IndexMap<String, IndexMap<String, IndexMap<String, String>>>;

#[test]
#[ignore = "should be comfirmed to call this"]
fn update_a_user_repo() -> Result<()> {
    plugin_cargo::logger::init();
    let mut manage = Manage::new()?;
    manage.update_submodules(&["os-checker/plugin-cargo".to_owned()])?;
    Ok(())
}
