# Lists all available commands.
default:
  just --list

# Install tools required by other recipes.
install-tools:
    rustup toolchain add nightly
    cargo +stable install cargo-hack --locked
    cargo +stable install cargo-minimal-versions --locked

    cargo +stable install cargo-msrv --locked

# Check if the current dependency version bounds are sufficient.
minimal-versions:
    cargo minimal-versions check --workspace --direct

# Find the minimum supported rust version.
msrv:
    cargo msrv find --path leptos-routes-macro

# Lint the code.
clippy:
    cargo clippy --all -- -W clippy::pedantic

trybuild-overwrite:
    TRYBUILD=overwrite cargo test -p leptos-routes-macro

# Update all deps; sort all Cargo.toml deps; format, check and lint all code; run all tests.
tidy:
    cargo update --workspace
    cargo sort --workspace
    cargo fmt
    cargo check --all
    cargo clippy --all -- -W clippy::pedantic
    cargo test --all
