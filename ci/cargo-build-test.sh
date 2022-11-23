#!/usr/bin/env bash

set -e
cd "$(dirname "$0")/.."

source ./ci/rust-version.sh stable



export RUSTFLAGS="-D warnings"
export RUSTBACKTRACE=1

set -x

# Build/test all host crates
cargo +"$rust_stable" build
cargo +"$rust_stable" test -- --nocapture

exit 0



# for mac

#export LDFLAGS="-L/usr/local/opt/bzip2/lib"
#export CPPFLAGS="-I/usr/local/opt/bzip2/include"