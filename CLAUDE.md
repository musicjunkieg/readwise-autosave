# readwise-autosave

Rust service integrating Bluesky bookmarks/DMs with Readwise.

## Quick Reference
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings && cargo fmt --check`
- Run: `cargo run`

## GitHub Issue Workflow (MANDATORY)

**BEFORE starting any work on a GitHub issue:**

1. **Check current branch**: `git branch --show-current`
2. **If on main, create issue branch**: `git checkout -b issue-{number}-{short-description}`
   - Example: `git checkout -b issue-2-oauth-flow`
3. **Verify you're on the correct branch** before making any changes

**Branch naming convention**: `issue-{number}-{description}`
- `issue-2-oauth-flow`
- `issue-3-database-crud`
- `issue-4-dm-polling`

**NEVER commit directly to main when working on an issue.**

## Development Workflow
1. Create branch for GitHub issue (see above)
2. Write tests FIRST (TDD)
3. Implement until tests pass
4. Commits happen automatically after file edits (via hooks)
5. When done, create PR: `gh pr create`
6. Capture learnings: `/learn`

## Tech Stack
- Rust + Tokio (async runtime)
- axum (web), sqlx (database)
- atproto-* crates (Nick Gerakines)

## Key Docs (see agent_docs/)
- architecture.md - System design
- testing-guide.md - TDD patterns
- atproto-api.md - Bluesky/AT Protocol APIs
- readwise-api.md - Readwise API reference

## Code Style
- Use `thiserror` for custom errors
- Traits for abstractions (testability)
- No unwrap() in production code
- Hooks auto-run `cargo fmt` and commit after file edits
