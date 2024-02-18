# Zero 2 Prod

## Getting Started

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# cargo-tarpaulin  # code coverage. Only supports x86 Linux...remove if on mac/windows
# cargo-watch      # reload/compile on save
# cargo-audit      # security alerts
# cargo-expand     # macro expansion
# cargo-udeps      # remove unused dependencies
# sccache          # better caching
cargo install \
  cargo-tarpaulin cargo-watch cargo-audit cargo-expand cargo-udeps sccache

rustup toolchain install nightly --allow-downgrade

cargo +nightly expand
cargo +nightly udeps

# Migrations
cargo install --version="~0.7" sqlx-cli --no-default-features --features rustls,postgres
```

On Linux:

```bash
sudo apt install mold clang
```

## IDE Tooling

### VSCod{e,ium}

* rust-analyzer
* Even Better TOML

## Code Coverage

Only on Linux:

```bash
cargo tarpaulin --ignore-tests
```

## Prepare SQLx Queries

```bash
cargo sqlx prepare -- --lib
```

## Rust Topics

* sqlx
* lifetimes ('static / 'a)
* trait bounds

## Docker Build/Run

```bash
docker build --tag zero2prod --file Dockerfile .
```
