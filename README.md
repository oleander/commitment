# Git Commit Formatter

Git Commit Formatter is a Rust project aimed to standardize git commit messages. It utilizes the branch name and commit message to create a formatted commit. It follows a ticketing system for tracking issues, using a pattern like `ABC-123` where `ABC` stands for a project key and `123` represents an issue number.

* ABC-123-v12 Message -> ABC-123 Message
* ABC-123b Message    -> ABC-123 Message
* ABC-123 Message     -> ABC-123 Message
* Message             -> Message

## Features

* Checks if there are any uncommitted changes in the current repository.
* Extracts the ticket from the branch name and the commit message.
* Creates a commit message following a specific format.
* Commits the changes if all the conditions are met.
* Prefixes the commit message with the ticket number
  * If the ticket number is not present in the branch name, the commit message is prefixed with the ticket number extracted from the commit message
  * If the ticket number is not present in the commit message, the commit message is prefixed with the ticket number extracted from the branch name
  * If the ticket number is not present in the branch name or the commit message, the commit message is not prefixed with a ticket number

## Install

1. Clone this repo
2. `cd` into the repo
3. `cargo install --path .`

## Usage

```bash
$ commitment "Your commit message"
```

## Tests

Tests are included for the functions `to_ticket()`, `commit()`, and `capitalize_first()`. These tests can be run using the command:

```bash
$ cargo test
```

## Dependencies

* [git2](https://crates.io/crates/git2): Rust bindings to the libgit2 library, provides the ability to create, manage, and manipulate Git repositories.
* [regex](https://crates.io/crates/regex): Rust library for parsing, compiling, and executing regular expressions.
