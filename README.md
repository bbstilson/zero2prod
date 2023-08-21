# Zero 2 Prod

## Getting Started

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

cargo install \
  # Only supports x86 Linux...comment if on mac
  cargo-tarpaulin \   # code coverage
  cargo-watch \       # reload/compile on save
  cargo-audit \       # security alerts
  cargo-expand \      # macro expansion
  sccache             # better caching

# For macro expansion
rustup toolchain install nightly --allow-downgrade
cargo +nightly expand

# Migrations
cargo install --version="~0.6" sqlx-cli --no-default-features --features rustls,postgres
```

On Mac:

```bash
brew install michaeleisel/zld/zld
```

On Linux:

```bash
sudo apt install lld clang
```

## Code Coverage

Only on Linux:

```bash
cargo tarpaulin --ignore-tests
```
