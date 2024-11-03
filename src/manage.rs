use crate::repo::SelfRepo;
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
}
