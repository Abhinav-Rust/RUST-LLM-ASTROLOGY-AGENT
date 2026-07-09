# Contributing to rust-llm-astrology-agent

Thank you for your interest in contributing! This project is primarily a **structural showcase**, meaning much of the proprietary domain logic (such as exact astronomical calculations and master prompts) has been redacted and replaced with stubs.

However, contributions that improve the structural integrity, multi-agent AI pipeline logic, performance, or overall Rust best practices are highly welcome!

## Getting Started

1. Ensure you have Rust installed (1.80+ recommended).
2. Fork the repository and clone it locally.
3. Obtain a Gemini API key and set it in your environment:
   ```bash
   export GEMINI_API_KEY="your_api_key_here"
   ```

## Development Guidelines

- **Code Quality**: We strictly enforce zero-warning builds. Before submitting a PR, ensure your code passes both:
  ```bash
  cargo fmt --all -- --check
  cargo clippy -- -D warnings
  ```
- **Tests**: If you add new pipeline features or utility functions, please include tests. Run all tests with:
  ```bash
  cargo test
  ```
- **Redacted Logic**: Please do not submit PRs attempting to "fill in" the stubbed astrological rules engines (`math.rs`, `rules.rs`, etc.) with real astrological logic, as this repo is intended solely to showcase the underlying AI orchestration and system architecture.

## Pull Requests

1. Create a new branch from `main`.
2. Follow the formatting in the PR template.
3. If your PR addresses an open issue, link it in the description.

Thank you!