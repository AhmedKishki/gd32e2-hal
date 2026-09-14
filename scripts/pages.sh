#!/usr/bin/env bash
# Builds the documentation of every part into target/pages for GitHub Pages:
# one directory per part feature, plus an index linking them. docs.rs covers a
# single feature set, and each part's page is exact only for that part.

set -euo pipefail
cd "$(dirname "$0")/.."

site=target/pages
doc=target/thumbv8m.base-none-eabi/doc
# The part features are the [features] entries named after a part number.
parts=$(sed -n 's/^\(gd32e230[0-9a-z]*\) = .*/\1/p' Cargo.toml)
version=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)

rm -rf "$site"
mkdir -p "$site"
for part in $parts; do
    printf '\033[36m==> %s\033[0m\n' "$part"
    # Emptied each time: rustdoc never deletes the page of an item the previous
    # part had and this one lacks.
    rm -rf "$doc"
    cargo doc --no-deps --target thumbv8m.base-none-eabi --features "defmt,$part"
    cp -r "$doc" "$site/$part"
done

{
    cat <<EOF
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>gd32e2-hal $version</title>
</head>
<body>
<h1>gd32e2-hal $version</h1>
<p>Documentation per part: pick the feature your chip is built with.</p>
<ul>
EOF
    for part in $parts; do
        printf '<li><a href="%s/gd32e2_hal/"><code>%s</code></a></li>\n' "$part" "$part"
    done
    cat <<'EOF'
</ul>
</body>
</html>
EOF
} > "$site/index.html"

printf '\033[32mSite in %s.\033[0m\n' "$site"
