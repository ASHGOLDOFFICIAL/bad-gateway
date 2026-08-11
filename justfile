build:
    cargo build --workspace --locked --verbose

deps:
    cargo machete

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

format:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets --locked -- -D warnings

run:
    cargo run --locked

test:
    cargo test --workspace --locked --verbose

typos:
    typos

ci: format lint doc build test deps typos
