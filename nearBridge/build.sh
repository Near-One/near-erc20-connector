#!/bin/bash
set -e
cd "`dirname $0`"
source ./flags.sh
cargo build --target wasm32-unknown-unknown --release
mkdir -p res
cp target/wasm32-unknown-unknown/release/near_bridge.wasm ./res/
