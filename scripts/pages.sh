#!/usr/bin/env bash
# Builds the documentation of every part into target/pages for GitHub Pages:
# one directory per part, plus an index in the look of rustdoc linking them.
# docs.rs covers a single feature set, and each part's page is exact only for
# that part.
#
# Needs nightly with the target, for -Zrustdoc-map — the mechanism docs.rs uses
# to link the dependencies to their own pages there instead of rendering them
# as plain text:
#   rustup toolchain install nightly --target thumbv8m.base-none-eabi

set -euo pipefail
cd "$(dirname "$0")/.."

site=target/pages
doc=target/thumbv8m.base-none-eabi/doc
version=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)

# The part table of build.rs, one line per part: feature, package, flash and
# SRAM in kilobytes.
table=$(sed -n 's/^ *Chip::new("\([a-z0-9]*\)", Flash::[A-Za-z0-9]*, Pads::\([A-Za-z0-9]*\), Timers::[A-Za-z]*, \([0-9]*\), \([0-9]*\)),$/\1 \2 \3 \4/p' build.rs)
if [ -z "$table" ]; then
    echo 'no parts found in the table of build.rs' >&2
    exit 1
fi
parts=$(cut -d' ' -f1 <<< "$table")
first=$(head -n1 <<< "$parts")

package() {
    case $1 in
        Pins20) echo '20 pins' ;;
        Pins24) echo '24 pins' ;;
        Pins28) echo '28 pins' ;;
        Lqfp32) echo '32 pins, LQFP' ;;
        Qfn32) echo '32 pins, QFN' ;;
        Pins48) echo '48 pins' ;;
        *) echo "unknown package $1 in build.rs" >&2; return 1 ;;
    esac
}

rm -rf "$site"
mkdir -p "$site"
for part in $parts; do
    printf '\033[36m==> %s\033[0m\n' "$part"
    # Starts empty: rustdoc never deletes the page of an item the previous part
    # had and this one lacks. Moved out below, so this only clears what an
    # interrupted run left behind.
    rm -rf "$doc"
    cargo +nightly -Zrustdoc-map --config 'doc.extern-map.registries.crates-io="https://docs.rs/"' \
        doc --no-deps --target thumbv8m.base-none-eabi --features "defmt,$part"
    mv "$doc" "$site/$part"
done

# The index borrows the stylesheets and the theme script of rustdoc itself, so
# it looks like the pages it links to and follows the theme picked there.
static=$site/static.files
cp -r "$site/$first/static.files" "$static"
asset() {
    basename "$(compgen -G "$static/$1")"
}
vars=$(grep -o '<meta name="rustdoc-vars"[^>]*>' "$site/$first/gd32e2_hal/index.html" |
    sed 's|data-root-path="../"|data-root-path="./"|; s|data-static-root-path="../static.files/"|data-static-root-path="static.files/"|')

{
    cat <<EOF
<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"><meta name="description" content="Documentation of gd32e2-hal $version, one page per part."><title>gd32e2_hal $version - Rust</title><link rel="stylesheet" href="static.files/$(asset 'normalize-*.css')"><link rel="stylesheet" href="static.files/$(asset 'rustdoc-*.css')">$vars<script src="static.files/$(asset 'storage-*.js')"></script><noscript><link rel="stylesheet" href="static.files/$(asset 'noscript-*.css')"></noscript><link rel="alternate icon" type="image/png" href="static.files/$(asset 'favicon-32x32-*.png')"><link rel="icon" type="image/svg+xml" href="static.files/$(asset 'favicon-*.svg')"></head>
<body class="rustdoc mod crate"><nav class="sidebar"><div class="sidebar-crate"><h2><a href="index.html">gd32e2_<wbr>hal</a><span class="version">$version</span></h2></div></nav><main><div class="width-limiter"><section id="main-content" class="content"><div class="main-heading"><h1>Crate <span>gd32e2_<wbr>hal</span></h1></div><div class="docblock"><p>The documentation is built once per part, so each page shows exactly the pins and peripherals of its part. Pick the feature your chip is built with.</p></div><h2 id="parts" class="section-header">Parts<a href="#parts" class="anchor">§</a></h2><dl class="item-table">
EOF
    while read -r part pads flash sram; do
        desc=$(package "$pads")
        printf '<dt><a class="mod" href="%s/gd32e2_hal/index.html" title="gd32e2_hal for %s">%s</a></dt><dd>%s, %sK flash, %sK SRAM</dd>\n' \
            "$part" "$part" "$part" "$desc" "$flash" "$sram"
    done <<< "$table"
    cat <<'EOF'
</dl></section></div></main></body></html>
EOF
} > "$site/index.html"

printf '\033[32mSite in %s.\033[0m\n' "$site"
