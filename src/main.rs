use git2::{IndexAddOption, Repository, StatusOptions};
use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
use anyhow::anyhow;
use regex::Regex;
use log::debug;

lazy_static! {
  static ref RE: Regex = Regex::new(r"^([A-Z]+-\d+)(\S*)?(?:\s+(.*))?$").unwrap();
}

pub(crate) trait Ticket {
  fn to_ticket(&self) -> (Option<&str>, Option<&str>);
}

impl Ticket for str {
  // Parse a string into a ticket and the rest of the string
  fn to_ticket(&self) -> (Option<&str>, Option<&str>) {
    if self.is_empty() {
      return (None, None);
    }

    if let Some(cap) = RE.captures(self) {
      let ticket = cap.get(1).map(|m| m.as_str());
      let rest = cap.get(3).map(|m| m.as_str());
      return (ticket, rest);
    }
    (None, Some(self))
  }
}

// Capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
  s.chars().next().map_or(String::new(), |f| f.to_uppercase().to_string() + &s[1..])
}

// Create a commit message from the branch name and the commit message
// * ABC-123 Message      -> ABC-123 Message
// * ABC-123x Message     -> ABC-123 Message
// * ABC-123-NOPE Message -> ABC-123 Message
// * ABC-123 Message      -> ABC-123 Message
// * Message              -> Message
// * ABC-123x             -> ABC-123
// * ABC-123-NOPE         -> ABC-123
fn create_commit(br: &str, msg: &str) -> Result<String> {
  match (br.to_ticket(), msg.to_ticket()) {
    ((Some(t1), _), (Some(t2), _)) if t1 != t2 => bail!("Branch and message tickets do not match".to_string()),
    ((Some(ticket), _), (None, Some(msg))) => Ok(format!("{} {}", ticket, capitalize_first(msg))),
    (_, (Some(ticket), Some(msg))) => Ok(format!("{} {}", ticket, capitalize_first(msg))),
    (_, (_, None)) => bail!("No commit message found".to_string()),
    ((None, _), (None, Some(msg))) => Ok(capitalize_first(msg))
  }
}

// Check if there are any uncommitted changes
fn has_repo_uncommited_changes(repo: &Repository) -> Result<bool> {
  let mut options = StatusOptions::new();
  options.include_untracked(true).recurse_untracked_dirs(true);

  match repo.statuses(Some(&mut options)) {
    Ok(statuses) => Ok(statuses.iter().any(|s| s.status() != git2::Status::CURRENT)),
    Err(e) => bail!("Failed to get statuses: {}", e)
  }
}


pub fn add_and_commit(repo: &Repository, msg: &str) -> Result<()> {
  debug!("[commit] Committing with message");

  let mut index = repo.index().expect("Failed to get index");
  index.add_all(["."].iter(), IndexAddOption::DEFAULT, None).context("Failed to run `git add`")?;
  index.write().context("Failed to write index from `git add`")?;
  let oid = index.write_tree().context("Failed to write tree")?;
  let tree = repo.find_tree(oid).context("Failed to find tree")?;
  let signature = repo.signature().context("Failed to get signature")?;

  match repo.head() {
    Ok(ref head) => {
      let parent = head
        .resolve()
        .context("Failed to resolve head")?
        .peel(ObjectType::Commit)
        .context("Failed to peel head")?
        .into_commit()
        .map_err(|_| anyhow!("Failed to resolve parent commit"))?;

      repo.commit(Some("HEAD"), &signature, &signature, &msg, &tree, &[&parent]).context("Failed to commit (1)")?;
    },
    Err(_) => {
      repo.commit(Some("HEAD"), &signature, &signature, &msg, &tree, &[]).context("Failed to commit (2)")?;
    }
  }

  Ok(())
}

// Get current branch name
fn get_branch_name(repo: &Repository) -> Result<String> {
  let head = repo.head().context("Failed to get HEAD")?;

  let Some(branch_name) = head.shorthand() else {
    bail!("Could not find branch name");
  };

  Ok(branch_name.to_string())
}

use git2::ObjectType;

fn main() -> Result<()> {
  // Recursively search for a git repository
  let current_dir = std::env::current_dir()?;
  let flags = git2::RepositoryOpenFlags::empty();
  let repo = Repository::open_ext(current_dir, flags, &[] as &[&str])?;

  if !has_repo_uncommited_changes(&repo)? {
    bail!("No uncommitted changes found");
  }

  let message = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
  let branch_name = get_branch_name(&repo)?;
  let msg = create_commit(branch_name.as_str(), &message)?;

  add_and_commit(&repo, &msg)?;

  Ok(())
}

// #[test]
// fn test_to_commit() {
//   assert_eq!(
//     commit("INVALID INVALID", "ABC-123 Message"),
//     Ok("ABC-123 Message".to_string())
//   );
//   assert_eq!(
//     commit("INVALID", "ABC-123 Message"),
//     Ok("ABC-123 Message".to_string())
//   );
//   assert_eq!(
//     commit("", "ABC-123 Message"),
//     Ok("ABC-123 Message".to_string())
//   );
//   assert_eq!(
//     commit("ABC-123", "Message"),
//     Ok("ABC-123 Message".to_string())
//   );
//   assert_eq!(
//     commit("ABC-123-DEF", "Tail"),
//     Ok("ABC-123 Tail".to_string())
//   );
//   assert_eq!(commit("", "message"), Ok("Message".to_string()));
//   assert_eq!(commit("X", "message"), Ok("Message".to_string()));
//   assert_eq!(
//     commit("ABC-123", "message"),
//     Ok("ABC-123 Message".to_string())
//   );
//   assert!(commit("ABC-123", "ABC-123 Tail1 Tail2").is_ok());
//   assert!(commit("ABC-123", "DEF-456 Tail").is_err());
//   assert!(commit("ABC-123", "ABC-123 Tail").is_ok());
//   assert!(commit("ABC-123", "DEF-456").is_err());
//   assert!(commit("ABC-123-DEF", "").is_err());
//   assert!(commit("ABC-123", "").is_err());
//   assert!(commit("", "").is_err());
// }

#[test]
fn test_capitalize_first() {
  assert_eq!(capitalize_first("abc"), "Abc");
  assert_eq!(capitalize_first("Abc"), "Abc");
  assert_eq!(capitalize_first(""), "");
}

#[test]
fn test_to_ticket() {
  assert_eq!("Head Tail1 Tail2 Tail3".to_ticket(), (None, Some("Head Tail1 Tail2 Tail3")));
  assert_eq!("ABC-123 Tail1 Tail2".to_ticket(), (Some("ABC-123"), Some("Tail1 Tail2")));
  assert_eq!("ABC-123 Tail".to_ticket(), (Some("ABC-123"), Some("Tail")));
  assert_eq!("ABC-123".to_ticket(), (Some("ABC-123"), None));
  assert_eq!("ABC-123".to_ticket(), (Some("ABC-123"), None));
  assert_eq!("Head".to_ticket(), (None, Some("Head")));
  assert_eq!("".to_ticket(), (None, None));
}
