# Multisig Solana Program

#### Build
```bash
cargo build-bpf --manifest-path=./program/Cargo.toml --bpf-out-dir=dist/program
```

#### Deploy
```bash
solana program deploy dist/program/multisig.so
```

#### Prepare to upgrade
```bash
solana program write-buffer --ws wss://api.mainnet-beta.solana.com dist/program/multisig.so
solana program set-buffer-authority ${PROGRAM_ID} --new-buffer-authority ${AUTHORITY}
```

#### Resuming failed deploy
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
