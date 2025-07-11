# Contributing to Canvas Contracts

Thank you for your interest in contributing to Canvas Contracts! This document provides guidelines and information for contributors.

## 🤝 How to Contribute

### Types of Contributions

We welcome various types of contributions:

- **Bug Reports**: Help us identify and fix issues
- **Feature Requests**: Suggest new features or improvements
- **Code Contributions**: Submit pull requests with code changes
- **Documentation**: Improve or add documentation
- **Testing**: Help test features and report issues
- **Community Support**: Help other users in discussions

### Before You Start

1. **Read the Documentation**: Familiarize yourself with the project structure and architecture
2. **Check Existing Issues**: Look for existing issues or discussions related to your contribution
3. **Join the Community**: Participate in discussions and ask questions

## 🚀 Development Setup

### Prerequisites

- **Rust** (latest stable)
- **Node.js** (v22+)
- **Git**
- **wasm-pack** (`cargo install wasm-pack`)
- **wasmtime** (`cargo install wasmtime-cli`)

### Local Development

1. **Fork and Clone**
   ```bash
   git clone https://github.com/YOUR_USERNAME/CanvasContracts.git
   cd CanvasContracts
   ```

2. **Setup Rust Backend**
   ```bash
   cargo build
   cargo test
   ```

3. **Setup Frontend**
   ```bash
   npm install
   npm run dev
   ```

4. **Run Development Server**
   ```bash
   npm run dev
   ```

## 📝 Code Style Guidelines

### Rust Code

- Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/style/naming/README.html)
- Use `cargo fmt` to format code
- Use `cargo clippy` to check for common issues
- Write comprehensive tests for new functionality

### TypeScript/JavaScript Code

- Use ESLint and Prettier configurations
- Follow TypeScript best practices
- Write JSDoc comments for public APIs
- Use meaningful variable and function names

### Git Commit Messages

Use conventional commit messages:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(compiler): add support for custom node types
fix(ui): resolve node connection validation issue
docs(readme): update installation instructions
```

## 🧪 Testing

### Running Tests

```bash
# Rust tests
cargo test

# Frontend tests
npm test

# Integration tests
npm run test:integration

# All tests
npm run test:all
```

### Writing Tests

- Write tests for all new functionality
- Include both unit and integration tests
- Test edge cases and error conditions
- Ensure good test coverage

### Test Structure

```
tests/
├── unit/              # Unit tests
├── integration/       # Integration tests
├── e2e/              # End-to-end tests
└── fixtures/         # Test data and fixtures
```

## 🔧 Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Write clean, well-documented code
- Add tests for new functionality
- Update documentation as needed
- Follow the code style guidelines

### 3. Test Your Changes

```bash
# Run all tests
npm run test:all

# Check code quality
npm run lint
npm run format
```

### 4. Commit Your Changes

```bash
git add .
git commit -m "feat(scope): description of changes"
```

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub.

## 📋 Pull Request Guidelines

### Before Submitting

- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] Documentation is updated
- [ ] No breaking changes (or clearly documented)
- [ ] Commit messages follow conventional format

### Pull Request Template

Use the following template when creating a pull request:

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No console errors
- [ ] No TypeScript errors

## Screenshots (if applicable)
Add screenshots for UI changes

## Additional Notes
Any additional information
```

## 🐛 Bug Reports

### Before Reporting

1. Check existing issues for duplicates
2. Try to reproduce the issue
3. Gather relevant information

### Bug Report Template

```markdown
## Bug Description
Clear description of the issue

## Steps to Reproduce
1. Step 1
2. Step 2
3. Step 3

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- OS: [e.g., Windows 10, macOS 12, Ubuntu 20.04]
- Rust Version: [e.g., 1.70.0]
- Node.js Version: [e.g., 22.0.0]
- Canvas Contracts Version: [e.g., 0.1.0]

## Additional Information
- Screenshots
- Error messages
- Console logs
```

## 💡 Feature Requests

### Before Requesting

1. Check if the feature already exists
2. Consider the impact on existing functionality
3. Think about implementation complexity

### Feature Request Template

```markdown
## Feature Description
Clear description of the requested feature

## Use Case
Why this feature is needed

## Proposed Solution
How you think it should work

## Alternatives Considered
Other approaches you've considered

## Additional Context
Any other relevant information
```

## 🏷️ Issue Labels

We use the following labels to categorize issues:

- `bug`: Something isn't working
- `enhancement`: New feature or request
- `documentation`: Improvements or additions to documentation
- `good first issue`: Good for newcomers
- `help wanted`: Extra attention is needed
- `question`: Further information is requested
- `wontfix`: This will not be worked on

## 📞 Getting Help

### Communication Channels

- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Discord**: For real-time chat (if available)

### Before Asking for Help

1. Check the documentation
2. Search existing issues and discussions
3. Try to reproduce the issue
4. Provide relevant information

## 🎉 Recognition

Contributors will be recognized in:

- **Contributors section** in the README
- **Release notes** for significant contributions
- **Project documentation** for major features

## 📄 License

By contributing to Canvas Contracts, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Canvas Contracts! Your help makes this project better for everyone. 