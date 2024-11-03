use eyre::ContextCompat;
use git2::Repository;
use plugin_cargo::{prelude::*, repo};

pub struct SelfRepo {
    this: Repository,
    submodules: Vec<Submodule>,
}

impl SelfRepo {
    pub fn vec_of_user_repo(&self) -> Vec<String> {
        self.submodules
            .iter()
            .map(|m| m.user_repo.clone())
            .collect()
    }

    pub fn update_submodules(&mut self) -> Result<()> {
        let submodules: Vec<_> = self
            .this
            .submodules()?
            .into_iter()
            .map(Submodule::new)
            .collect::<Result<_>>()?;

        info!(
            old_submodules = self.submodules.len(),
            new_submodules = submodules.len()
        );

        self.submodules = submodules;
        Ok(())
    }
}

pub struct Submodule {
    // relative dir path
    local: Utf8PathBuf,
    url: String,
    user_repo: String,
}

impl Submodule {
    pub fn new(m: git2::Submodule) -> Result<Self> {
        let local = Utf8PathBuf::from_path_buf(m.path().into()).unwrap();
        let url = m.url().unwrap().to_owned();

        let user_repo = url
            .strip_prefix("https://github.com/")
            .with_context(|| format!("{url} can't strip prefix `https://github.com/`"))?;
        let user_repo = user_repo
            .strip_suffix(".git")
            .unwrap_or(user_repo)
            .to_owned();
        Ok(Self {
            local,
            url,
            user_repo,
        })
    }

    pub fn repo_metadata(&self) -> Result<repo::Repo> {
        repo::Repo::new(&self.user_repo, repo::RepoSource::Local(self.local.clone()))
    }
}

fn self_repo() -> Result<SelfRepo> {
    let mut this = SelfRepo {
        this: Repository::open(".")?,
        submodules: vec![],
    };
    this.update_submodules();
    Ok(this)
}

#[test]
fn parse_submodules() -> Result<()> {
    let repo = self_repo()?;

    let v: Vec<_> = repo
        .submodules
        .iter()
        .map(|m| {
            let meta = m.repo_metadata()?;
            Ok((meta.user, meta.repo, &m.local))
        })
        .collect::<Result<_>>()?;
    dbg!(&v);

    Ok(())
}
