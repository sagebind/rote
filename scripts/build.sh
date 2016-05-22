#!/usr/bin/env bash
# Builds Rote from scratch.

export CARGO_TARGET_DIR=./target


build-crate() {
    local path=$1
    shift
    local crate=$(basename $path)
    local manifest=$path/Cargo.toml

    if [ ! -f $manifest ]; then
        echo "No manifest file for component '$crate', skipping"
        continue
    fi

    echo "Building crate '$crate'..."
    local cmd="cargo rustc --manifest-path $manifest -- $@"
    echo "   $cmd"
    export BUILD_TIME=$(date)
    eval $cmd || exit
    echo
}


# Build all of the remaining crates.
for component in components/*; do
    if [ ! -d $component ]; then
        continue
    fi

    # Skip runtime...
    if [ $(basename $component) = "runtime" ]; then
        continue
    fi

    # Build it
    build-crate $component
done
