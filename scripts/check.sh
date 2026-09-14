#!/usr/bin/env bash
# The checks CI runs, in the same order. Stops at the first failure.
#
# The library is linted on both ends of the range: the largest part, and the
# smallest one, whose x4 pin map and small-package gates the largest never sees.

set -euo pipefail
cd "$(dirname "$0")/.."

run() {
    printf '\033[36m==> cargo %s\033[0m\n' "$*"
    if ! cargo "$@"; then
        printf '\033[31mFAILED: cargo %s\033[0m\n' "$*" >&2
        exit 1
    fi
}

run fmt --check
run libclippy -- -D warnings
run clippy --features gd32e230f4xx -- -D warnings
run lib
run examplesclippy -- -D warnings
run examples
RUSTDOCFLAGS='-D warnings' run doc --no-deps --target thumbv8m.base-none-eabi --features defmt,gd32e230c8xx

printf '\033[32mAll checks passed.\033[0m\n'
