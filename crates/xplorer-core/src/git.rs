use std::collections::HashMap;
use std::path::Path;

use git2::{Repository, StatusOptions, StatusShow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitFileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Ignored,
    Conflicted,
}

#[derive(Debug, Clone)]
pub struct GitInfo {
    pub branch: String,
    pub repo_root: String,
    pub statuses: HashMap<String, GitFileStatus>,
}

pub fn get_git_info(dir_path: &str) -> Option<GitInfo> {
    let repo = Repository::discover(dir_path).ok()?;

    let branch = current_branch(&repo);
    let repo_root = repo
        .workdir()?
        .to_string_lossy()
        .trim_end_matches(['/', '\\'])
        .to_string();

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(false)
        .include_ignored(false)
        .show(StatusShow::IndexAndWorkdir);

    let statuses_raw = repo.statuses(Some(&mut opts)).ok()?;
    let mut statuses = HashMap::new();

    for entry in statuses_raw.iter() {
        let Some(rel_path) = entry.path() else {
            continue;
        };
        let abs_path = Path::new(&repo_root).join(rel_path);
        let abs_str = abs_path.to_string_lossy().to_string();
        let s = entry.status();

        let status = if s.is_conflicted() {
            GitFileStatus::Conflicted
        } else if s.is_index_new() || s.is_wt_new() {
            if s.is_wt_new() {
                GitFileStatus::Untracked
            } else {
                GitFileStatus::Added
            }
        } else if s.is_index_deleted() || s.is_wt_deleted() {
            GitFileStatus::Deleted
        } else if s.is_index_renamed() || s.is_wt_renamed() {
            GitFileStatus::Renamed
        } else if s.is_index_modified() || s.is_wt_modified() {
            GitFileStatus::Modified
        } else if s.is_ignored() {
            GitFileStatus::Ignored
        } else {
            continue;
        };

        statuses.insert(abs_str, status);
    }

    Some(GitInfo {
        branch,
        repo_root,
        statuses,
    })
}

fn current_branch(repo: &Repository) -> String {
    if let Ok(head) = repo.head() {
        if let Some(name) = head.shorthand() {
            return name.to_string();
        }
    }
    if let Ok(head) = repo.head() {
        let oid = head.target().unwrap_or_else(|| git2::Oid::zero());
        return format!("{:.7}", oid);
    }
    "HEAD".to_string()
}
