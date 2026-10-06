# Lists all available commands.
default:
  just --list

# Install tools required by other recipes.
install-tools:
    rustup toolchain add nightly
    rustup toolchain add 1.88.0
    cargo +stable install cargo-hack --locked
    cargo +stable install cargo-minimal-versions --locked
    cargo +stable install cargo-msrv --locked
    cargo +stable install cargo-sort --locked

# Check if the current dependency version bounds are sufficient.
minimal-versions:
    cargo minimal-versions check --workspace --direct

# Find the minimum supported rust version.
msrv:
    cargo msrv find --path leptos-routes-macro

# Format all code.
fmt:
    cargo fmt --all

# Check that all code is formatted.
fmt-check:
    cargo fmt --all -- --check

# Sort all Cargo.toml deps.
sort:
    cargo sort --workspace

# Check that all Cargo.toml deps are sorted.
sort-check:
    cargo sort --workspace --check

# Type-check all code.
check:
    cargo check --workspace --all-targets

# Verify the library code compiles with the MSRV. Dev-dependencies may require a newer toolchain.
check-msrv:
    cargo +1.88.0 check --workspace --lib

# Type-check the examples, which are separate workspaces.
check-examples:
    cd examples/basic && cargo check --all-targets
    cd examples/routes-only && cargo check --all-targets

# Lint the code.
clippy:
    cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic

# Run all tests.
test:
    cargo test --workspace

# Build documentation with rustdoc warnings denied.
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps

# Regenerate the expected `.stderr` output of all compile-fail trybuild tests.
trybuild-overwrite:
    TRYBUILD=overwrite cargo test -p leptos-routes-macro

# Update all deps; sort all Cargo.toml deps; format all code.
tidy:
    cargo update --workspace
    just sort
    just fmt

# Check formatting and dep sorting; lint all code; run all tests; build docs; check MSRV and examples.
verify: fmt-check sort-check clippy test doc check-msrv check-examples
