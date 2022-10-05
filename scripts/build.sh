#!/usr/bin/env bash

function print_help() {
  echo 'Usage: build.sh [OPTIONS]'
  echo ''
  echo 'Options:'
  echo '  -h,--help         Print this help message and exit'
  echo '  -p,--programs     Build solana program'
  echo '  -w,--wasm         Build WASM bindings'
  echo '  -b,--bindings     Build Rust bindings'
}

while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
      -h|--help)
        print_help
        exit 0
      ;;
      -p|--programs)
        shift # past argument

        cargo build-bpf --manifest-path=./program/Cargo.toml  --bpf-out-dir=dist/program
      ;;
      -w|--wasm)
        shift # past argument

        wasm-pack build --target web --out-name index program  -- --features wasm
      ;;
      -b|--bindings)
        shift # past argument

        cargo build --release --manifest-path=./program/Cargo.toml  --features=bindings
      ;;
      -t|--tests)
        shift # past argument

        cargo test-bpf --manifest-path=./program/Cargo.toml
      ;;
      *) # unknown option
        echo 'ERROR: Unexpected'
        exit 1
      ;;
  esac
done
