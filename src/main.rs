use git2::{IndexAddOption, Repository, StatusOptions};
use regex::Regex;

trait Ticket {
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
fn capitalize_first(s: &str) -> String {
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
fn commit(br: &str, msg: &str) -> Result<String, String> {
  match (br.to_ticket(), msg.to_ticket()) {
    ((Some(t1), _), (Some(t2), _)) if t1 != t2 => Err("Branch and message tickets do not match".to_string()),

    ((Some(ticket), _), (None, Some(msg))) => Ok(format!("{} {}", ticket, capitalize_first(msg))),

    (_, (Some(ticket), Some(msg))) => Ok(format!("{} {}", ticket, capitalize_first(msg))),

    (_, (_, None)) => Err("No commit message".to_string()),

    ((None, _), (None, Some(msg))) => Ok(capitalize_first(msg)),
  }
}

// Check if there are any uncommitted changes
fn has_changes(repo: &Repository) -> Result<bool, String> {
  let mut options = StatusOptions::new();
  options.include_untracked(true).recurse_untracked_dirs(true);

  match repo.statuses(Some(&mut options)) {
    Ok(statuses) => Ok(statuses.iter().any(|s| s.status() != git2::Status::CURRENT)),
    Err(e) => Err(format!("Failed to get statuses: {}", e)),
  }
}

fn main() {
  let current_dir = std::env::current_dir().expect("Failed to get current directory");
  let repo = Repository::open_ext(
    current_dir,
    git2::RepositoryOpenFlags::empty(),
    &[] as &[&str],
  )
  .expect("Failed to open repository");

  let has_changes = has_changes(&repo).expect("Failed to check for uncommitted changes");
  if !has_changes {
    eprintln!("No uncommitted changes found");
    std::process::exit(1);
  }

  let message = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
  let branch_name_option = if !repo.is_empty().unwrap() {
    let head = repo.head().expect("Failed to get HEAD");
    head.shorthand().map(|s| s.to_string())
  } else {
    None
  };

  let Some(branch_name) = branch_name_option.as_deref() else {
    eprintln!("Failed to get branch name");
    std::process::exit(1);
  };

  let commit_msg = commit(branch_name, &message).expect("Failed to create commit message");

  let mut index = repo.index().expect("Failed to get current index");
  index.add_all(["."].iter(), IndexAddOption::DEFAULT, None).expect("Failed to run `git add`");
  index.write().expect("Failed to write index from `git add`");

  let tree_oid = index.write_tree().expect("Failed to write index tree");
  let tree = repo.find_tree(tree_oid).expect("Failed to find index tree");

  let signature = repo.signature().expect("Failed to get current signature");

  if repo.is_empty().unwrap() {
    // No commits yet, create an initial commit
    repo
      .commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_msg,
        &tree,
        &[],
      )
      .expect("Failed to create an initial commit");
  } else {
    let oid = repo.head().unwrap().target().expect("Failed to get HEAD target");
    let parent = repo.find_commit(oid).expect("Failed to find parent commit");
    repo
      .commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_msg,
        &tree,
        &[&parent],
      )
      .expect("Failed to commit");
  }
}
