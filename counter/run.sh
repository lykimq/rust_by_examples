#!/usr/bin/env bash

# build.sh
# echo ">> Building"
# rustup target add wasm32-unknown-unknown
# cargo build --all --target wasm32-unknown-unknown --release

# deploy.sh
#./build.sh

#echo ">> Deploying contract"

#near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/contract.wasm

build(){
 cargo build
}

test(){
    cargo test
}

case "$1" in 
build) 
 build
 ;;
test)
 test
 ;;
esac