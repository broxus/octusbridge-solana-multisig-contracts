# Multisig Solana Program

#### Build
```bash
cargo build-bpf --manifest-path=./program/Cargo.toml --bpf-out-dir=dist/program
```

#### Deploy
```bash
solana-keygen recover -o dist/program/multisig-buffer-keypair.json
solana program deploy --buffer dist/program/multisig-buffer-keypair.json dist/program/multisig.so
```

#### Run tests
```bash
cargo test-bpf --manifest-path=./program/Cargo.toml
```

#### Build WASM bindings
```bash
wasm-pack build --target web --out-name index program -- --features wasm
```

#### Build Rust bindings
```bash
cargo build --release --manifest-path=./program/Cargo.toml --features=bindings
```
