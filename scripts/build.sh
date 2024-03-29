#!/usr/bin/env bash

function print_help() {
  echo 'Usage: build.sh [OPTIONS]'
  echo ''
  echo 'Options:'
  echo '  -h,--help         Print this help message and exit'
  echo '  -p,--program      Build solana program'
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
      -p|--program)
        shift # past argument

        cargo-build-sbf --manifest-path=./program/Cargo.toml --sbf-out-dir=dist/program
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

        cargo-test-sbf --manifest-path=./program/Cargo.toml
      ;;
      *) # unknown option
        echo 'ERROR: Unexpected'
        exit 1
      ;;
  esac
done
