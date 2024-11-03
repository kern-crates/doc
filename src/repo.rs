use eyre::ContextCompat;
use git2::Repository;
use plugin_cargo::{prelude::*, repo};

pub struct SelfRepo {
    this: Repository,
    submodules: Vec<Submodule>,
}

pub struct Submodule {
    // relative dir path
    local: Utf8PathBuf,
    url: String,
    repo: repo::Repo,
}

impl Submodule {
    pub fn new(m: git2::Submodule) -> Result<Self> {
        let local = Utf8PathBuf::from_path_buf(m.path().into()).unwrap();
        let url = m.url().unwrap().to_owned();

        let user_repo = url
            .strip_prefix("https://github.com/")
            .with_context(|| format!("{url} can't strip prefix `https://github.com/`"))?;
        let user_repo = user_repo.strip_suffix(".git").unwrap_or(user_repo);
        let repo = repo::Repo::new(user_repo, repo::RepoSource::Local(local.clone()))?;
        Ok(Self { local, url, repo })
    }
}

fn self_repo() -> Result<SelfRepo> {
    let this = Repository::open(".")?;

    let submodules = this
        .submodules()?
        .into_iter()
        .map(Submodule::new)
        .collect::<Result<_>>()?;

    Ok(SelfRepo { this, submodules })
}

#[test]
fn parse_submodules() -> Result<()> {
    let repo = self_repo()?;

    let v: Vec<_> = repo
        .submodules
        .iter()
        .map(|m| (&m.repo.user, &m.repo.repo, &m.local))
        .collect();
    dbg!(&v);

    Ok(())
}
