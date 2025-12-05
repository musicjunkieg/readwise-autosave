# readwise-autosave

Rust service integrating Bluesky bookmarks/DMs with Readwise.

## Quick Reference
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings && cargo fmt --check`
- Run: `cargo run`

## Workflow
1. All features on new branch: `git checkout -b feature/xxx`
2. Write tests FIRST (TDD)
3. Implement until tests pass
4. Lint before commit: `/lint`
5. Capture learnings: `/learn`

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
- Run `cargo fmt` after creating files
