#!/usr/bin/env bash
# Cleans the entire project tree.

export CARGO_TARGET_DIR=./target

# Clean each component directory.
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

    echo "clean: $component"
    cargo clean --manifest-path $manifest "$@" || exit
done
