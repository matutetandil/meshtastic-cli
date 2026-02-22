# Contributing

Contributions are welcome! Here are some guidelines to keep in mind.

## Code Standards

- All code follows **SOLID principles** — one responsibility per module, depend on abstractions
- New commands are added as independent modules under `src/commands/`
- Each command implements the `Command` trait defined in `commands/mod.rs`

## Before Submitting

Make sure all checks pass:

```bash
cargo clippy -- -D warnings   # no warnings allowed
cargo fmt --check              # formatting must be applied
cargo test                     # all tests must pass
cargo build                    # clean build
```

## Adding a New Command

1. Create a new file under `src/commands/` (e.g., `my_command.rs`)
2. Implement the `Command` trait
3. Add `mod my_command;` to `src/commands/mod.rs`
4. Add the CLI variant to `src/cli.rs`
5. Add the match arm in the `create_command()` factory in `src/commands/mod.rs`
6. Update documentation (README command table and relevant docs page)

## Project Structure

See the [Architecture](./architecture.md) page for details on the project structure and design patterns.

## Reporting Issues

If you find a bug or have a feature request, please open an issue at [GitHub Issues](https://github.com/matutetandil/mttctl/issues).
