# Huge Commit

Huge Commit is a Rust application designed to generate commit messages using OpenAI's GPT models. It leverages the git2 and openai crates to analyze git diffs and create meaningful, concise commit messages based on the changes made in the repository.

## Features
- Analyzes git diffs to understand the context of changes.
- Generates commit messages that focus on the why, not just the what of changes, adhering to best practices for commit messages.
- Prioritizes features, fixes, refactors, and style changes in commit messages.
- Interactive confirmation prompts for staging changes and committing with the generated message.

## Requirements

- Rust programming environment
- Git repository
- OpenAI API key

## Setup

1. Add your OpenAI API key to your environment variables:
   ```sh
   export OPENAI_API_KEY=your_openai_api_key_here
   ```

2. Ensure you have Rust and Cargo installed. If not, follow the installation instructions from the official Rust website.

## Installation
```sh
cargo install huge-commit
```

## Usage
```sh
$ huge-commit
> No changes to commit. stage all changes? Yes
Enhanced project metadata for better public visibility
> commit with this message? Yes
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributions

Contributions are welcome! Please feel free to submit a pull request or open an issue for discussion.

