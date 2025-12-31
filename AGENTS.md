---
description: Best practices and rules for AI developers working on this project
---

# AI Developer Guidelines

## Build & Test Rules

### Cargo Build
- **DO NOT** run `cargo build` or `cargo run` or `dx serve` — the user will do this manually
- There are CLI integration issues that prevent the AI from running these commands reliably

### Cargo Test
- **OK to run** `cargo test` — this can be executed by the AI for verification

### Cargo Check
- **OK to run** `cargo check` — useful for quick syntax/type verification without full build

## Development Workflow

1. **Make changes** to source files
2. **Run `cargo check`** if you want to verify syntax/types
3. **Run `cargo test`** if there are tests to verify
4. **Notify the user** that changes are ready for them to build/run

## Code Style

- Follow standard Rust conventions (rustfmt defaults)
- Use `snake_case` for functions and variables
- Use `PascalCase` for types and structs
- Keep functions focused and reasonably sized
- Add doc comments (`///`) for public APIs

## Dioxus Specifics

- Use RSX syntax for UI components
- Keep components in separate files under `src/components/`
- State management goes in `src/state/`
- Core logic (non-UI) goes in `src/core/`

## File Organization

```
src/
├── main.rs              # Entry point only
├── app.rs               # Root App component
├── components/          # UI components
├── state/               # State management
├── core/                # Non-UI logic
└── providers/           # Provider adapters
```

## Communication

- Surface to the user frequently during iterative work
- Don't make sweeping changes without check-ins
- When making UI changes, describe what was changed so user knows what to look for when they build
