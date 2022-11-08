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

# this will change with the manually test files
run(){
 # rustc debug.rs
 # rustc list.rs
 # rustc primitives.rs
 # rustc tuples.rs
 # rustc array.rs
 rustc custom_enum_types.rs

}

show(){
    # ./debug
    # ./list
    # ./primitives
    # ./tuples
    # ./array
    ./custom_enum_types
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
run) 
 run
 ;;
show)
 show
 ;;
fmt)
 fmt
 ;;
 esac
