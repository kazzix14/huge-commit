# Huge Commit

Huge Commit is an application designed to generate commit messages using OpenAI's GPT models. It analyzes git diffs to generate meaningful commit messages and provides interactive confirmation prompts for users.

## Features

- Analyze git diffs
- Generate commit messages
- Interactive confirmation prompts

## Requirements

To use Huge Commit, you need the following:

- Rust environment
- Git repository
- OpenAI API key

## Installation

Install Huge Commit using cargo with the following command:

```sh
cargo install huge-commit
```

## Usage


```sh
Usage: huge-commit [OPTIONS] [COMMAND]

Commands:
  commit  Commit changes.
  config  Get or set configuration.
  model   models.
  help    Print this message or the help of the given subcommand(s)

Options:
  -m, --base-message <BASE_MESSAGE>  The base message to use for the commit.
  -y, --assume-yes                   Assume yes to all prompts.
  -h, --help                         Print help
```

## Contributions

Contributions to Huge Commit are welcome! Please feel free to contribute by opening issues or submitting pull requests.

## License

Huge Commit is released under the MIT License. For more information, see the LICENSE file in the repository.

## Dependencies

Huge Commit requires the following dependencies:

- Rust
- Git repository
- OpenAI API key
