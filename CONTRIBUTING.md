# Contributing to DeepWiki MCP Server Extension

Thank you for your interest in contributing to the DeepWiki MCP Server Extension! This guide will help you get started with contributing to both the extension and its documentation.

## 🏗️ Project Architecture

This project uses a **separated architecture**:

- **Extension Repository** (this repo): Zed extension (WASM)
- **Proxy Repository**: [zed-mcp-proxy](https://github.com/keshav1998/zed-mcp-proxy) (Native binary)

## 🚀 Getting Started

### Prerequisites

- **Rust** (latest stable): [Install Rust](https://rustup.rs/)
- **WASM target**: `rustup target add wasm32-wasip1`
- **Development tools**:
  ```bash
  cargo install cargo-nextest
  cargo install lefthook
  ```

### Development Setup

1. **Fork and clone the repository**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/deepwiki-mcp-server.git
   cd deepwiki-mcp-server
   ```

2. **Install Git hooks**:
   ```bash
   lefthook install
   ```

3. **Verify setup**:
   ```bash
   cargo build --target wasm32-wasip1
   cargo nextest run
   ```

## 🔧 Development Workflow

### Code Quality Standards

We maintain high code quality through automated tools:

```bash
# Format code (required before commit)
cargo fmt --all

# Lint code (must pass)
cargo clippy --target wasm32-wasip1 -- -D warnings

# Run tests (must pass)
cargo nextest run

# Build extension
cargo build --target wasm32-wasip1 --release
```

### Pre-commit Hooks

Lefthook automatically runs quality checks:
- **Pre-commit**: Format and lint checks
- **Pre-push**: Full test suite and build verification

## 📝 Contributing Guidelines

### Code Contributions

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/amazing-feature
   ```

2. **Make your changes** following our standards:
   - Write comprehensive tests for new functionality
   - Add rustdoc comments for public APIs
   - Follow Rust naming conventions
   - Ensure WASM compatibility (no async/HTTP in extension)

3. **Test thoroughly**:
   ```bash
   cargo nextest run
   cargo clippy --target wasm32-wasip1 -- -D warnings
   cargo build --target wasm32-wasip1 --release
   ```

4. **Commit with clear messages**:
   ```bash
   git commit -m "feat: add amazing feature that does X"
   ```

5. **Push and create a Pull Request**:
   ```bash
   git push origin feature/amazing-feature
   ```

### Documentation Contributions

Documentation is crucial for user adoption and developer onboarding.

#### Types of Documentation

1. **API Documentation** (`src/lib.rs`):
   - Use comprehensive rustdoc comments
   - Include examples where helpful
   - Document error conditions
   - Explain parameter meanings

2. **User Documentation** (`README.md`):
   - Installation instructions
   - Configuration examples
   - Usage guidelines
   - Troubleshooting tips

3. **Developer Documentation** (`docs/DOCS.md`, `docs/BUILD.md`):
   - Architecture explanations
   - Development setup
   - Testing procedures
   - Contribution guidelines

#### Documentation Standards

- **Clear and Concise**: Use simple, direct language
- **Practical Examples**: Include working code samples
- **Up-to-date**: Keep documentation synchronized with code changes
- **Comprehensive**: Cover both happy path and edge cases
- **Consistent**: Use consistent terminology and formatting

#### Rustdoc Guidelines

```rust
/// Brief one-line description of the function.
///
/// More detailed explanation of what the function does,
/// how it works, and when to use it.
///
/// # Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
///
/// Description of return value and possible variants.
///
/// # Errors
///
/// Description of error conditions and when they occur.
///
/// # Examples
///
/// ```rust,no_run
/// let result = example_function("input");
/// assert_eq!(result, expected_output);
/// ```
pub fn example_function(param1: &str) -> Result<String> {
    // Implementation
}
```

#### Documentation Build Process

```bash
# Generate API documentation
cargo doc --target wasm32-wasip1 --no-deps --open

# Verify documentation quality
cargo doc --target wasm32-wasip1 --no-deps --quiet
```

## 🧪 Testing Guidelines

### Test Structure

- **Unit Tests**: Test individual functions and components
- **Integration Tests**: Test end-to-end functionality
- **Documentation Tests**: Ensure examples in docs work (where applicable)

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_functionality() {
        // Arrange
        let input = "test input";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
}
```

### Running Tests

```bash
# Run all tests
cargo nextest run

# Run specific test module
cargo test unit_tests

# Run with output for debugging
cargo test test_name -- --nocapture
```

## 🔒 Security Guidelines

- **No hardcoded secrets**: Use environment variables
- **Input validation**: Validate all user inputs
- **WASM safety**: Ensure WASM compatibility
- **Dependency auditing**: Run `cargo audit` regularly

## 📋 Pull Request Process

### Before Submitting

- [ ] Code follows project style guidelines
- [ ] All tests pass (`cargo nextest run`)
- [ ] Documentation is updated
- [ ] Commit messages are clear and descriptive
- [ ] No merge conflicts with main branch

### PR Description Template

```markdown
## Description
Brief description of the changes and their purpose.

## Changes Made
- [ ] Feature A added/modified
- [ ] Documentation updated
- [ ] Tests added/updated

## Testing
- [ ] All existing tests pass
- [ ] New tests added for new functionality
- [ ] Manual testing completed

## Documentation
- [ ] API documentation updated
- [ ] README updated (if needed)
- [ ] Examples provided

## Breaking Changes
List any breaking changes and migration steps.
```

### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by maintainers
3. **Documentation review** for clarity and accuracy
4. **Testing verification** on multiple platforms (if applicable)

## 🐛 Bug Reports

### Before Reporting

1. Check existing issues for duplicates
2. Test with the latest version
3. Gather relevant information:
   - Operating system and architecture
   - Zed version
   - Extension version
   - Error messages and logs
   - Steps to reproduce

### Bug Report Template

```markdown
**Bug Description**
Clear description of the bug.

**Steps to Reproduce**
1. Step one
2. Step two
3. Step three

**Expected Behavior**
What should happen.

**Actual Behavior**
What actually happens.

**Environment**
- OS: [e.g., macOS 14.0, Ubuntu 22.04]
- Architecture: [e.g., x86_64, aarch64]
- Zed version: [e.g., 0.123.0]
- Extension version: [e.g., 0.1.0]

**Additional Context**
Logs, screenshots, or other relevant information.
```

## 🌟 Feature Requests

### Before Requesting

1. Check existing issues for similar requests
2. Consider if the feature aligns with project goals
3. Think about implementation complexity

### Feature Request Template

```markdown
**Feature Description**
Clear description of the desired feature.

**Use Case**
Why is this feature needed? What problem does it solve?

**Proposed Solution**
How should this feature work?

**Alternatives Considered**
Other approaches you've considered.

**Additional Context**
Any other relevant information.
```

## 📞 Getting Help

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and general discussion
- **Documentation**: Check [DOCS.md](docs/DOCS.md) for comprehensive information
- **Code Examples**: Look at existing tests for usage patterns

## 🎯 Contribution Areas

We welcome contributions in these areas:

### High Priority
- Bug fixes and stability improvements
- Documentation improvements
- Test coverage expansion
- Performance optimizations

### Medium Priority
- New MCP server integrations
- Additional configuration options
- Developer experience improvements
- Platform compatibility enhancements

### Low Priority
- UI/UX improvements
- Optional features
- Code refactoring (must maintain backward compatibility)

## 📄 License

By contributing to this project, you agree that your contributions will be licensed under the MIT License.

## 🙏 Recognition

Contributors are recognized in:
- README.md acknowledgments
- Git commit history
- Release notes for significant contributions

Thank you for contributing to the DeepWiki MCP Server Extension! 🚀