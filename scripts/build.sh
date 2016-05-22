#!/usr/bin/env bash
# Builds Rote from scratch.

export CARGO_TARGET_DIR=target


build-crate() {
    local path=$1
    shift
    local crate=$(basename $path)
    local manifest=$path/Cargo.toml

    if [ ! -f $manifest ]; then
        echo -e "No manifest file for component \033[1m$crate\033[0m, skipping\n"
        continue
    fi

    echo -e "Building crate \033[1m$crate\033[0m..."
    cargo build --manifest-path $manifest $@ || exit
    echo
}

TIMEFORMAT='All components built in %lR.'
time {
    # Build all of the remaining crates.
    for component in components/*; do
        if [ ! -d $component ]; then
            continue
        fi

        # Build it
        build-crate $component $@
    done
}

echo -e "\033[1m┬─┐┌─┐┌┬┐┌─┐  ┬┌─┐  ┬─┐┌─┐┌─┐┌┬┐┬ ┬  ┌┬┐┌─┐  ┬─┐┌─┐┬  ┬  ┬
├┬┘│ │ │ ├┤   │└─┐  ├┬┘├┤ ├─┤ ││└┬┘   │ │ │  ├┬┘│ ││  │  │
┴└─└─┘ ┴ └─┘  ┴└─┘  ┴└─└─┘┴ ┴─┴┘ ┴    ┴ └─┘  ┴└─└─┘┴─┘┴─┘o\033[0m"
