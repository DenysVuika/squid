# Documentation Structure

This document provides an overview of the squid project's documentation organization.

## 📁 Project Structure

```
squid/
├── docs/                       # Documentation directory
│   ├── README.md              # Documentation index and quick reference
│   ├── QUICKSTART.md          # Get started in 5 minutes
│   ├── REVIEW_GUIDE.md        # Code review feature guide
│   ├── EXAMPLES.md            # Comprehensive usage examples
│   ├── FILE_CONTEXT.md        # File context feature architecture
│   ├── IMPLEMENTATION_SUMMARY.md  # Implementation details
│   ├── CHANGELOG.md           # Version history
│   ├── STRUCTURE.md           # This file
│   └── sample.txt             # Sample file for testing
│
├── examples/                   # Example files for testing
│   ├── README.md              # Guide to using example files
│   ├── test-reviews.sh        # Automated test script
│   ├── example.rs             # Rust code with issues
│   ├── example.ts             # TypeScript code with issues
│   ├── example.js             # JavaScript code with issues
│   ├── example.html           # HTML with accessibility issues
│   ├── example.css            # CSS with performance issues
│   └── example.py             # Python with best practice violations
│
├── src/                        # Source code
│   ├── assets/                # Embedded assets
│   │   ├── code-review.md     # Generic review prompt
│   │   ├── review-rust.md     # Rust-specific review prompt
│   │   ├── review-typescript.md  # TypeScript/JS review prompt
│   │   ├── review-html.md     # HTML review prompt
│   │   └── review-css.md      # CSS review prompt
│   ├── main.rs                # Main application
│   └── logger.rs              # Logging configuration
│
├── README.md                   # Main project README
├── Cargo.toml                  # Rust package configuration
├── LICENSE                     # Apache-2.0 license
└── .env                        # Environment configuration (not in repo)
```

## 📚 Documentation Guide

### For New Users

Start here to get productive quickly:

1. **[README.md](../README.md)** - Project overview, features, and basic usage
2. **[docs/QUICKSTART.md](QUICKSTART.md)** - 5-minute getting started guide
3. **[docs/EXAMPLES.md](EXAMPLES.md)** - Common use cases and workflows

### For Code Review Feature

Learn about AI-powered code reviews:

1. **[docs/REVIEW_GUIDE.md](REVIEW_GUIDE.md)** - Complete code review guide
2. **[examples/README.md](../examples/README.md)** - Test files and examples
3. Run `./examples/test-reviews.sh` - Automated testing

### For Developers

Technical documentation for contributors:

1. **[docs/FILE_CONTEXT.md](FILE_CONTEXT.md)** - Architecture and design
2. **[docs/IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)** - Implementation details
3. **[docs/CHANGELOG.md](CHANGELOG.md)** - Version history

### For Maintainers

Project maintenance and evolution:

1. **[docs/CHANGELOG.md](CHANGELOG.md)** - Track changes
2. **[docs/STRUCTURE.md](STRUCTURE.md)** - This document
3. **[src/assets/](../src/assets/)** - Review prompt templates

## 📖 Document Purposes

### Main Documents

| Document | Audience | Purpose |
|----------|----------|---------|
| `README.md` | All users | Project overview, quick examples, installation |
| `docs/README.md` | All users | Documentation index and navigation |
| `docs/QUICKSTART.md` | New users | Fast onboarding and first steps |
| `docs/REVIEW_GUIDE.md` | All users | Code review feature documentation |
| `docs/EXAMPLES.md` | Users | Detailed examples and workflows |
| `docs/FILE_CONTEXT.md` | Developers | Technical architecture |
| `docs/IMPLEMENTATION_SUMMARY.md` | Developers | Implementation details |
| `docs/CHANGELOG.md` | All | Version history and changes |
| `examples/README.md` | Testers | Example file guide |

### Asset Files

| File | Purpose |
|------|---------|
| `src/assets/code-review.md` | Generic code review prompt |
| `src/assets/review-rust.md` | Rust-specific review guidelines |
| `src/assets/review-typescript.md` | TypeScript/JavaScript review guidelines |
| `src/assets/review-html.md` | HTML review guidelines |
| `src/assets/review-css.md` | CSS review guidelines |

### Example Files

| File | Purpose | Issues Demonstrated |
|------|---------|---------------------|
| `examples/example.rs` | Rust testing | Panics, clones, non-idiomatic code |
| `examples/example.ts` | TypeScript testing | Any types, XSS, memory leaks |
| `examples/example.js` | JavaScript testing | Eval, callbacks, security issues |
| `examples/example.html` | HTML testing | Semantics, accessibility, SEO |
| `examples/example.css` | CSS testing | Performance, specificity, magic numbers |
| `examples/example.py` | Generic prompt testing | Best practices, security, patterns |

## 🎯 Documentation Goals

### User-Focused
- Clear, concise explanations
- Practical examples
- Common use cases first
- Progressive complexity

### Developer-Focused
- Technical architecture details
- Implementation rationale
- Code organization
- Extension points

### Maintainer-Focused
- Change tracking
- Version history
- Structure documentation
- Contribution guidelines

## 🔄 Documentation Workflow

### Adding New Features

1. Update `README.md` with basic usage
2. Add detailed guide in `docs/` if substantial
3. Update relevant existing docs
4. Add examples to `docs/EXAMPLES.md`
5. Update `docs/CHANGELOG.md`
6. Update this structure document if needed

### Adding New Language Support

1. Create prompt: `src/assets/review-{language}.md`
2. Update `src/main.rs` with constant and matcher
3. Create example: `examples/example.{ext}`
4. Update `examples/README.md`
5. Update `docs/REVIEW_GUIDE.md`
6. Update `README.md` feature list

### Versioning

- Major changes: Update all relevant docs
- Minor features: Update specific docs + CHANGELOG
- Bug fixes: Update CHANGELOG
- Documentation fixes: No version change needed

## 📝 Writing Guidelines

### Style
- Use clear, concise language
- Include code examples
- Use tables for comparisons
- Add emoji for visual hierarchy (sparingly)
- Use `code blocks` for commands and paths

### Structure
- Start with overview/purpose
- Quick start section
- Detailed information
- Examples
- Troubleshooting (if applicable)

### Code Examples
- Use realistic examples
- Show both input and output
- Include comments for clarity
- Demonstrate best practices

## 🔗 Cross-References

Documents should reference each other appropriately:

- `README.md` → All major feature docs
- `docs/README.md` → All documentation files
- `docs/QUICKSTART.md` → EXAMPLES.md for more details
- `docs/REVIEW_GUIDE.md` → examples/README.md for testing
- `examples/README.md` → docs/REVIEW_GUIDE.md for usage

## 🚀 Future Documentation

Planned documentation additions:

- **API Reference** - Detailed API documentation
- **Contributing Guide** - How to contribute
- **Architecture Decision Records** - Design decisions
- **Performance Guide** - Optimization tips
- **Integration Guide** - CI/CD and IDE integration

---

Last Updated: 2024
Maintained by: Squid Team