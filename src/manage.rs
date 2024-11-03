use crate::{repo::SelfRepo, submodule_add::submodule_add};
use indexmap::IndexMap;
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

    pub fn process(&mut self, user_repo: &str) -> Result<()> {
        // add and commit a new submodule if applies
        submodule_add(user_repo, &mut self.local)?;

        // update submodules
        self.self_repo.update_submodules()?;

        for submodule in self.self_repo.submodules() {
            let key = &submodule.user_repo;
            if !self.local.contains_key(key) {
                info!("append a new user_repo: {key}");
                let repo = submodule.repo_metadata()?;
                self.local.insert(key.clone(), repo);
            }
        }

        Ok(())
    }
}
