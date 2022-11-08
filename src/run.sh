#!/usr/bin/env bash
# ./run.sh run_basic
# ./run.sh show_basic
# vscode extension prettier: ctrl + shift + i 

# Basic.rs
run_basic(){
 rustc basic.rs
}

show_basic(){
    ./basic
}

# Debug.rs
run_debug(){
 rustc debug.rs
}

show_debug(){
    ./debug
}

fmt(){
    prettier --write **/*.rs
}

case "$1" in 
run_basic) 
 run_basic
 ;;
show_basic)
 show_basic
 ;;
run_debug) 
 run_debug
 ;;
show_debug)
 show_debug
 ;;
fmt)
 fmt
 ;;
 esac
