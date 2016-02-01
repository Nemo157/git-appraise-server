use std::vec::IntoIter;
use git2::{ Oid, Repository, Commit };

pub struct CommitTree<'repo> {
  repo: &'repo Repository,
  next: Option<Commit<'repo>>,
  commits: IntoIter<Commit<'repo>>,
  ignored: Vec<Oid>,
}

impl<'repo> CommitTree<'repo> {
  pub fn new(repo: &'repo Repository) -> CommitTree<'repo> {
    let mut walker = repo.revwalk().unwrap();
    walker.push_head().unwrap();
    walker.simplify_first_parent();
    let commits = walker.map(|id| repo.find_commit(id).unwrap()).collect();
    CommitTree::create(repo, commits, Vec::new())
  }

  pub fn is_empty(&self) -> bool {
    self.next.is_none()
  }

  fn between(repo: &'repo Repository, first: &Commit<'repo>, ignored: Vec<Oid>) -> CommitTree<'repo> {
    let mut walker = repo.revwalk().unwrap();
    for parent in first.parent_ids().skip(1) {
      walker.push(parent).unwrap();
    }
    for ignored in ignored.clone() {
      walker.hide(ignored).unwrap();
    }
    walker.simplify_first_parent();
    let commits = walker.map(|id| repo.find_commit(id).unwrap()).collect();
    CommitTree::create(repo, commits, ignored)
  }

  fn create(repo: &'repo Repository, commits: Vec<Commit<'repo>>, ignored: Vec<Oid>) -> CommitTree<'repo> {
    let mut iter = commits.into_iter();
    CommitTree {
      repo: repo,
      next: iter.next(),
      commits: iter,
      ignored: ignored,
    }
  }
}

impl<'repo> Iterator for CommitTree<'repo> {
  type Item = (Commit<'repo>, CommitTree<'repo>);

  fn next(&mut self) -> Option<Self::Item> {
    match self.next.take() {
      Some(commit) => {
        self.next = self.commits.next();
        let mut ignored = self.ignored.clone();
        if self.next.is_some() {
          ignored.push(self.next.as_ref().unwrap().id());
        }
        let sub = CommitTree::between(self.repo, &commit, ignored);
        Some((commit, sub))
      },
      None => None,
    }
  }
}