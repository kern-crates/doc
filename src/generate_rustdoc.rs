use crate::DEPLOY;
use duct::cmd;
use indexmap::{indexmap, IndexSet};
use plugin_cargo::{prelude::*, repo::Repo, write_json};

pub struct Manage {
    repo: Repo,
}

impl Manage {
    pub fn new(user_repo: &str) -> Result<Self> {
        let repo = Repo::new(user_repo, plugin_cargo::repo::RepoSource::Github)?;
        Ok(Manage { repo })
    }

    pub fn cargo_doc(&self, docs_: &mut Docs) -> Result<()> {
        let url_prefix = std::env::var("DOCS_URL")
            .map(|s| s.trim_end_matches('/').to_owned())
            .unwrap_or_default();

        let repos_dir = Utf8PathBuf::from_iter(["/tmp", "os-checker-plugin-cargo"]);

        // cargo doc --document-private-items --workspace --no-deps
        let data = &self.repo;

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

            match doc_dir.read_dir_utf8() {
                Ok(dir) => {
                    for entry in dir {
                        let entry = entry?;
                        if entry.path().is_dir() {
                            doc_path.insert(entry.file_name().to_owned());
                        }
                    }
                }
                Err(err) => {
                    error!(?err);
                    continue;
                }
            }

            let ws_stripped = ws_dir.strip_prefix(&repos_dir)?; // user/repo/ws
            info!(?pkg_crate_names, %ws_stripped, %doc_dir, ?doc_path);

            let mut urls = IndexMap::<String, Option<String>>::with_capacity(pkg_crate_names.len());

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
            urls.sort_unstable_keys();

            docs_.dirs.push(DocDir {
                src: doc_dir,
                dst: Utf8PathBuf::from(DEPLOY).join(ws_stripped),
            });

            let (user, repo) = (data.user.as_str(), data.repo.as_str());
            match docs_.docs.get_mut(user) {
                Some(map_repo) => match map_repo.get_mut(repo) {
                    Some(map_pkgs) => map_pkgs.extend(urls),
                    None => _ = map_repo.insert(repo.to_owned(), urls),
                },
                None => {
                    let map = indexmap! { repo.to_owned() =>urls };
                    docs_.docs.insert(user.to_owned(), map);
                }
            }
        }

        Ok(())
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
    pub fn new() -> Self {
        let docs = UserRepoPkgCrate::with_capacity(128);
        let dirs = Vec::with_capacity(128);
        Docs { docs, dirs }
    }

    pub fn finish(&mut self) -> Result<()> {
        self.docs.values_mut().for_each(|m| m.sort_unstable_keys());
        self.docs.sort_unstable_keys();

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

        write_json(&Utf8PathBuf::from_iter([DEPLOY, "docs.json"]), &self.docs)?;

        Ok(())
    }
}

#[derive(Debug)]
struct DocDir {
    src: Utf8PathBuf,
    dst: Utf8PathBuf,
}
