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
    local cmd="cargo rustc --manifest-path $manifest -- -C rpath $@"
    echo "   $cmd"
    eval $cmd || exit
    echo
}


# First build the core runtime library, since all other crates depend on it.
build-crate components/runtime -C prefer-dynamic
libs=(target/debug/deps/*.so)
rm $libs

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
    build-crate $component --extern runtime=target/debug/libruntime.so
done


rote_bin=$CARGO_TARGET_DIR/debug/rote

# Now that the binary is built, and the necessary modules, use Rote to finish the build process.
if [ ! -f $rote_bin ]; then
    echo "Build failed: could not find intermediate Rote binary"
    exit 1
fi

eval $rote_bin build-finish
