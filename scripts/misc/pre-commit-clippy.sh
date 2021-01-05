#!/bin/sh

find . -name '*.rs' | xargs touch
cargo clippy -- -D clippy::all
