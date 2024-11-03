use crate::{repo::SelfRepo, submodule_add::submodule_add, DEPLOY, REPOS};
use duct::cmd;
use indexmap::{indexmap, IndexMap, IndexSet};
use plugin_cargo::{prelude::*, repo::Repo, write_json};

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

    pub fn cargo_doc(&self) -> Result<Docs> {
        let url_prefix = std::env::var("DOCS_URL")
            .map(|s| s.trim_end_matches('/').to_owned())
            .unwrap_or_default();

        let mut docs = UserRepoPkgCrate::with_capacity(128);
        let mut dirs = Vec::with_capacity(128);
        let current_dir = Utf8PathBuf::from_path_buf(std::env::current_dir().unwrap()).unwrap();
        let repos_dir = current_dir.join(REPOS);
        info!(%current_dir, %repos_dir);

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
                    .workspace_packages()
                    .iter()
                    .map(|p| (p.name.clone(), p.name.replace("-", "_")))
                    .collect();
                let mut doc_path = IndexSet::with_capacity(pkg_crate_names.len());

                let doc_dir = meta.target_directory.join("doc");

                for entry in doc_dir.read_dir_utf8()? {
                    let entry = entry?;
                    if entry.path().is_dir() {
                        doc_path.insert(entry.file_name().to_owned());
                    }
                }

                let ws_stripped = ws_dir.strip_prefix(&repos_dir)?; // user/repo/ws
                info!(?pkg_crate_names, %ws_stripped, %doc_dir, ?doc_path);

                let mut urls =
                    IndexMap::<String, Option<String>>::with_capacity(pkg_crate_names.len());

                // check missing docs
                for (pkg, krate) in pkg_crate_names {
                    let url = if !doc_path.contains(&krate) {
                        error!("crate `{krate}` does not generate rustdoc");
                        None
                    } else {
                        Some(format!("{url_prefix}/{ws_stripped}/{krate}"))
                    };
                    urls.insert(pkg, url);
                }

                dirs.push(DocDir {
                    src: doc_dir,
                    dst: repos_dir.join(DEPLOY).join(ws_stripped),
                });

                let (user, repo) = (data.user.as_str(), data.repo.as_str());
                match docs.get_mut(user) {
                    Some(map_repo) => match map_repo.get_mut(repo) {
                        Some(map_pkgs) => map_pkgs.extend(urls),
                        None => _ = map_repo.insert(repo.to_owned(), urls),
                    },
                    None => {
                        let map = indexmap! { repo.to_owned() =>urls };
                        docs.insert(user.to_owned(), map);
                    }
                }
            }
        }

        Ok(Docs { docs, dirs })
    }
}

/// Crate doc is possible to be missing due to build failure
/// The url constains workspace dir.
pub type UserRepoPkgCrate = IndexMap<String, IndexMap<String, IndexMap<String, Option<String>>>>;

pub struct Docs {
    /// docs.json
    docs: UserRepoPkgCrate,
    dirs: Vec<DocDir>,
}

impl Docs {
    pub fn finish(&self) -> Result<()> {
        info!(
            "doc = {}\ndirs = {:#?}",
            serde_json::to_string_pretty(&self.docs)?,
            self.dirs
        );

        for dir in &self.dirs {
            let parent = dir.dst.parent().unwrap();
            std::fs::create_dir_all(parent)?;

            let DocDir { src, dst } = dir;
            info!("mv {src} {dst}");
            cmd!("mv", src, dst).run()?;
        }

        write_json(
            &Utf8PathBuf::from_iter([REPOS, "deploy", "docs.json"]),
            &self.docs,
        )?;

        Ok(())
    }
}

#[derive(Debug)]
struct DocDir {
    src: Utf8PathBuf,
    dst: Utf8PathBuf,
}

#[test]
#[ignore = "should be comfirmed to call this"]
fn update_a_user_repo() -> Result<()> {
    plugin_cargo::logger::init();
    let mut manage = Manage::new()?;
    manage.update_submodules(&["os-checker/plugin-cargo".to_owned()])?;
    Ok(())
}
