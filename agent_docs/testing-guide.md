# Testing Guide

## TDD Workflow

1. **Write test** describing expected behavior
2. **Run test** - verify it fails (red)
3. **Write minimal code** to make it pass (green)
4. **Refactor** if needed
5. **Repeat**

## Test Organization

```
src/
├── lib.rs
├── module/
│   ├── mod.rs
│   └── tests.rs          # Unit tests for module
tests/
├── integration_test.rs   # Integration tests
└── fixtures/             # Test data
```

## Unit Tests

Place in `#[cfg(test)]` modules within each file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Arrange
        let input = "test";

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

## Async Tests

Use `#[tokio::test]` for async code:

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

## Mocking External Services

Use traits for external dependencies:

```rust
#[async_trait]
pub trait ReadwiseClient: Send + Sync {
    async fn save_highlight(&self, highlight: Highlight) -> Result<()>;
}

// In tests:
struct MockReadwiseClient;

#[async_trait]
impl ReadwiseClient for MockReadwiseClient {
    async fn save_highlight(&self, _: Highlight) -> Result<()> {
        Ok(())
    }
}
```

Consider `mockall` crate for complex mocking needs.

## Running Tests

```bash
# All tests
cargo test

# Single test
cargo test test_name

# With output
cargo test -- --nocapture

# Watch mode (requires cargo-watch)
cargo watch -x test
```

## Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html
```
