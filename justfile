alias b := build
alias c := check
alias d := delete
alias f := fmt
alias t := test

_default:
    @just --list

# Build
build:
    cargo build

# Check code: formatting, compilation, linting, and commit signature
check:
    cargo +nightly fmt --all -- --check
    cargo check --workspace
    cargo clippy -- -D warnings

# Delete files: example, target, lockfile
delete item="examples":
    just _delete-{{ item }}

# Format code
fmt:
    cargo +nightly fmt

test:
    cargo test

_delete-target:
    rm -rf target/

_delete-lockfile:
    rm -f Cargo.lock
