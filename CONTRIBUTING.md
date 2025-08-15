# Contributing to FocusFive

Thank you for your interest in contributing to FocusFive! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, inclusive, and constructive. We're building a tool to help people achieve their goals - let's embody that positive spirit in our collaboration.

## How to Contribute

### Reporting Issues
- Check existing issues first
- Provide clear reproduction steps
- Include system information (OS, terminal, Rust version)
- Attach relevant goal files (sanitized of personal data)

### Suggesting Features
- Open an issue with the "enhancement" label
- Explain the use case and value proposition
- Consider how it aligns with FocusFive's minimalist philosophy

### Submitting Code

1. **Fork and Clone**
   ```bash
   git clone https://github.com/YOUR_USERNAME/goal_setting.git
   cd goal_setting
   ```

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**
   - Follow existing code style
   - Add tests for new functionality
   - Update documentation as needed

4. **Test Thoroughly**
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt
   ```

5. **Commit with Clear Messages**
   ```bash
   git commit -m "feat: add keyboard shortcut for quick save"
   ```

6. **Push and Create PR**
   - Provide clear description of changes
   - Link related issues
   - Include screenshots for UI changes

## Development Setup

### Prerequisites
- Rust 1.75+
- Git
- A terminal that supports TUI applications

### Building
```bash
cargo build
cargo run
```

### Testing
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Code Style

### Rust Guidelines
- Use `rustfmt` for formatting
- Follow Rust API guidelines
- Prefer clarity over cleverness
- Document public APIs

### Commit Messages
Follow conventional commits:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `test:` Test additions/changes
- `refactor:` Code refactoring
- `style:` Formatting changes
- `perf:` Performance improvements

## Architecture Decisions

### Core Principles
1. **Local First** - No network requirements for core functionality
2. **Privacy** - User data never leaves their machine
3. **Simplicity** - Features must justify their complexity
4. **Speed** - Daily interaction must feel instant

### File Format
- Markdown for human readability
- YAML frontmatter for structured data (future)
- ISO 8601 dates for consistency

## Testing Philosophy

- Unit tests for data models and parsing
- Integration tests for file I/O
- Manual testing for TUI interactions
- Example files for documentation

## Documentation

- Update README for user-facing changes
- Document code with rustdoc comments
- Maintain examples in `examples/`
- Update build plan for architectural changes

## Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Tag release: `git tag v0.1.0`
4. Push tags: `git push --tags`
5. GitHub Actions handles the rest

## Questions?

Open an issue or reach out in discussions. We're here to help!

## Recognition

Contributors will be recognized in:
- README.md acknowledgments
- Release notes
- Project documentation

Thank you for helping make FocusFive better! 🎯