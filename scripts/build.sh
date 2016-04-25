#!/usr/bin/env bash
# Builds Rote from scratch.

export CARGO_TARGET_DIR=./target

# Build all of the crates.
for component in ./components/*; do
    if [ ! -d $component ]; then
        continue
    fi

    crate=$(basename $component)
    manifest=$component/Cargo.toml

    if [ ! -f $manifest ]; then
        echo "No manifest file for component '$crate', skipping"
        continue
    fi

    echo "Building crate '$crate'..."
    cargo build --manifest-path $manifest "$@" || exit
    echo
done

rote_bin=$CARGO_TARGET_DIR/debug/rote

# Now that the binary is built, and the necessary modules, use Rote to finish the build process.
if [ ! -f $rote_bin ]; then
    echo "Build failed: could not find intermediate Rote binary"
    exit 1
fi

eval $rote_bin build-finish
