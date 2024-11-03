use crate::repo::SelfRepo;
use indexmap::IndexSet;
use plugin_cargo::{prelude::*, repo::Repo};

/// This map is a collection of user_repo string, and
/// must ensure there exist a local repo.
pub type Local = IndexMap<String, Repo>;

pub struct Manage {
    self_repo: SelfRepo,
    local: Local,
}
