#!/usr/bin/env bash

# Crate-specific settings
toolchains=( stable beta nightly "1.71.1" )

set -Eeuo pipefail

echo_and_run() { echo "$ ${*@Q}"; "$@"; }

echo_and_run cargo outdated --exit-code 1

for toolchain in "${toolchains[@]}"; do
    (
        export CARGO_TARGET_DIR="$PWD/target/check-$toolchain"
        echo_and_run cargo "+$toolchain" clippy --all-targets -- -D warnings
        echo_and_run cargo "+$toolchain" build --all-targets
        echo_and_run cargo "+$toolchain" test --all-targets
        echo_and_run cargo "+$toolchain" test --release --all-targets
        echo_and_run cargo "+$toolchain" test --doc
        echo_and_run cargo "+$toolchain" test --doc --release
    )
done

echo_and_run cargo semver-checks
echo_and_run cargo deny --workspace check

echo "All checks succeeded." 1>&2
