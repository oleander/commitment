
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use git2::{IndexAddOption, Repository, StatusOptions};
use regex::Regex;

pub(crate) trait Ticket {
  fn to_ticket(&self) -> (Option<&str>, Option<&str>);
}

impl Ticket for str {
  // Parse a string into a ticket and the rest of the string
  fn to_ticket(&self) -> (Option<&str>, Option<&str>) {
    if self.is_empty() {
      return (None, None);
    }

    let re = Regex::new(r"^([A-Z]+-\d+)(\S*)?(?:\s+(.*))?$").unwrap();

    if let Some(cap) = re.captures(self) {
      let ticket = cap.get(1).map(|m| m.as_str());
      let rest = cap.get(3).map(|m| m.as_str());
      return (ticket, rest);
    }
    (None, Some(self))
  }
}

// Capitalize the first letter of a string
pub(crate) fn capitalize_first(s: &str) -> String {
  let mut c = s.chars();
  match c.next() {
    None => String::new(),
    Some(f) => f.to_uppercase().chain(c).collect(),
  }
}

// Create a commit message from the branch name and the commit message
// * ABC-123 Message      -> ABC-123 Message
// * ABC-123x Message     -> ABC-123 Message
// * ABC-123-NOPE Message -> ABC-123 Message
// * ABC-123 Message      -> ABC-123 Message
// * Message              -> Message
// * ABC-123x             -> ABC-123
// * ABC-123-NOPE         -> ABC-123
pub(crate) fn commit(br: &str, msg: &str) -> Result<String> {
  match (br.to_ticket(), msg.to_ticket()) {
    ((Some(t1), _), (Some(t2), _)) if t1 != t2 => bail!("Branch and message tickets do not match".to_string()),
    ((Some(ticket), _), (None, Some(msg))) => Ok(format!("{} {}", ticket, capitalize_first(msg))),
    (_, (Some(ticket), Some(msg))) => Ok(format!("{} {}", ticket, capitalize_first(msg))),
    (_, (_, None)) => bail!("Failed to parse commit message"),
    ((None, _), (None, Some(msg))) => Ok(capitalize_first(msg)),
  }
}

// Check if there are any uncommitted changes
pub(crate) fn has_changes(repo: &Repository) -> Result<bool, anyhow::Error> {
  let mut options = StatusOptions::new();
  options.include_untracked(true).recurse_untracked_dirs(true);

  match repo.statuses(Some(&mut options)) {
    Ok(statuses) => Ok(statuses.iter().any(|s| s.status() != git2::Status::CURRENT)),
    Err(e) => Err(anyhow!("Failed to get statuses: {}", e)),
  }
}

fn main() -> anyhow::Result<()> {
  let current_dir = std::env::current_dir().context("Failed to get current directory")?;
  let repo = Repository::open_ext(
    current_dir,
    git2::RepositoryOpenFlags::empty(),
    &[] as &[&str],
  )
  .context("Failed to open repository")?;

  if !has_changes(&repo)? {
    anyhow::bail!("No uncommitted changes found");
  }

  let message = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
  let head = repo.head().context("Failed to get HEAD")?;
  // Branch name used to generate prefixes for commit messages
  let Some(branch_name) = head.shorthand() else {
    bail!("Could not find branch name");
  };

  let commit_msg = commit(branch_name, &message)?;

  let mut index = repo.index().context("Failed to get current index")?;
  index.add_all(["."].iter(), IndexAddOption::DEFAULT, None).context("Failed to run `git add`")?;
  index.write().context("Failed to write index from `git add`")?;

  let tree_oid = index.write_tree().context("Failed to write index tree")?;
  let tree = repo.find_tree(tree_oid).context("Failed to find index tree")?;

  let signature = repo.signature().context("Failed to get current signature")?;

  // No commits yet, create an initial commit
  if repo.is_empty().unwrap() {
    repo
      .commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_msg,
        &tree,
        &[],
      )
      .context("Failed to create an initial commit")?;
  } else {
    let oid = repo
      .head()
      .context("Failed to get HEAD")?
      .target()
      .ok_or_else(|| anyhow::anyhow!("Failed to get HEAD target"))?;
    let parent = repo.find_commit(oid).context("Failed to find parent commit")?;
    repo
      .commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_msg,
        &tree,
        &[&parent],
      )
      .context("Failed to commit")?;
  }

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
  assert_eq!(
    "Head Tail1 Tail2 Tail3".to_ticket(),
    (None, Some("Head Tail1 Tail2 Tail3"))
  );
  assert_eq!(
    "ABC-123 Tail1 Tail2".to_ticket(),
    (Some("ABC-123"), Some("Tail1 Tail2"))
  );
  assert_eq!("ABC-123 Tail".to_ticket(), (Some("ABC-123"), Some("Tail")));
  assert_eq!("ABC-123".to_ticket(), (Some("ABC-123"), None));
  assert_eq!("ABC-123".to_ticket(), (Some("ABC-123"), None));
  assert_eq!("Head".to_ticket(), (None, Some("Head")));
  assert_eq!("".to_ticket(), (None, None));
}
