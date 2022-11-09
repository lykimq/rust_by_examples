#!/usr/bin/env bash

check(){
    cargo check
}

# compile but do not run
build(){
    cargo build
}

# remove folder target
clean(){
    cargo clean
}

run(){
    cargo run
}

case "$1" in 
run)
 run
;;
check)
 check
;;
build)
 build
;;
clean)
 clean
;;
esac