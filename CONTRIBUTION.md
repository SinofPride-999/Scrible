# Contributing to Scribble

Thank you for your interest in contributing to **Scribble**!

We welcome bug fixes, improvements, documentation updates, and new features.

---

## Getting Started

1. Fork the repository
2. Clone your fork

```bash
git clone https://github.com/SinofPride-999/Scribble
cd scribble
```

3. Create a new branch

```bash
git checkout -b feature/my-feature
```

---

## Development Setup

Install Rust if you haven't already:

https://rust-lang.org

Build the project:

```bash
cargo build
```

Run the editor:

```bash
cargo run -- test.txt
```

Run checks:

```bash
cargo check
cargo clippy
cargo fmt
```

---

## Code Style

Please follow these guidelines:

* Use `cargo fmt` before committing
* Ensure `cargo clippy` shows no warnings
* Keep functions small and readable
* Prefer clear naming over clever code
* Document complex logic

---

## Commit Guidelines

Write clear commit messages.

Example:

```
add undo history support
fix cursor overflow bug
improve renderer performance
```

---

## Pull Requests

When submitting a pull request:

1. Ensure the project builds
2. Run `cargo fmt`
3. Run `cargo clippy`
4. Describe what your change does
5. Include screenshots or examples if UI changes are involved

---

## Reporting Bugs

If you find a bug, please open an issue with:

* Description of the problem
* Steps to reproduce
* Expected behavior
* Terminal / OS details

---

## Feature Requests

Feature suggestions are welcome. Please open an issue describing:

* The feature
* Why it would be useful
* Possible implementation ideas (optional)

---

## Community Guidelines

Please be respectful and constructive when contributing.

We aim to keep Scribble:

* Friendly
* Open
* Collaborative

---

## Reach Me:

`jhay@luxid.dev`

---

## Thank You

Every contribution helps improve Scribble.
Thank you for taking the time to contribute!