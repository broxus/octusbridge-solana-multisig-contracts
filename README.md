# Solana Multisig program

#### Build docker container
```bash
docker build -t contract-builder .
```

#### Build contracts
```bash
# Run docker container
docker run --volume ${PWD}:/root -it --rm contract-builder:latest

# Build solana programs
./scripts/build.sh --programs

# Build WASM binding
./scripts/build.sh --wasm

# Build Rust binding
./scripts/build.sh --bindings

# Verify solana programs
./scripts/verify.sh \
  --address msigDiHoyMYxDmLsPYQzvCKuw23yET41p8HM7aMZw6q \
  --binary dist/program/multisig.so \
  --url https://api.mainnet-beta.solana.com

# Leave docker container
exit
```

#### Deploy
```bash
solana program deploy ./dist/program/multisig.so
```

#### Prepare to upgrade
```bash
solana program write-buffer --ws wss://api.mainnet-beta.solana.com dist/program/${PROGRAM_BIN}
solana program set-buffer-authority ${BUFFER_PROGRAM_ID} --new-buffer-authority ${MSIG_AUTHORITY}
```

#### Resuming failed deploy
```bash
solana-keygen recover -o dist/program/multisig-buffer-keypair.json
solana program deploy --buffer dist/program/multisig-buffer-keypair.json dist/program/multisig.so
```
