![image](https://github.com/user-attachments/assets/8e19637c-eb1f-4170-b202-663c1434e073)

# Building

## Native
```bash
# Prerequisites
rustup update

# Build
cargo run           # dev build
cargo run --release # optimized build
```

## Web Assembly

```bash
# Prerequisites
rustup update
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# Build
wasm-pack build --target web --out-dir dist
```
