Prepare and create a commit.

1. Format code: `cargo fmt`
2. Run linter: `cargo clippy -- -D warnings`
3. Run tests: `cargo test`
4. If all pass:
   - Stage changes: `git add -A`
   - Create commit with conventional commit message
   - Show `git status` to confirm

Use conventional commits: feat:, fix:, docs:, refactor:, test:, chore:
