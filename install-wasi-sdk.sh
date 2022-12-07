#!/usr/bin/env bash

set -euo pipefail

if [[ "$(basename $(pwd))" != "ruvy" ]]; then
    echo "Run this inside in the root of the ruvy repo" 1>&2
    exit 1
fi

# Don't try and install the wasi-sdk if the user has specified the wasi-sdk is installed elsewhere
set +u
if [[ -n "$WASM_SYS_WASI_SDK_PATH" ]]; then
    # Check that something is present where the user says the wasi-sdk is located
    if [[ ! -d "$WASM_SYS_WASI_SDK_PATH" ]]; then
        echo "Download the wasi-sdk to $WASM_SYS_WASI_SDK_PATH" 1>&2
        exit 1
    fi
    exit 0
fi
set -u

PATH_TO_SDK="crates/wasm-sys/wasi-sdk"
if [[ ! -d $PATH_TO_SDK ]]; then
    TMPGZ=$(mktemp)
    VERSION_MAJOR="12"
    VERSION_MINOR="0"
    if [[ "$(uname -s)" == "Darwin" ]]; then
        curl --fail --location --silent https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${VERSION_MAJOR}/wasi-sdk-${VERSION_MAJOR}.${VERSION_MINOR}-macos.tar.gz --output $TMPGZ
    else
        curl --fail --location --silent https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${VERSION_MAJOR}/wasi-sdk-${VERSION_MAJOR}.${VERSION_MINOR}-linux.tar.gz --output $TMPGZ
    fi
    mkdir $PATH_TO_SDK
    tar xf $TMPGZ -C $PATH_TO_SDK --strip-components=1
fi
