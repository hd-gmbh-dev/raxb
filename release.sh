#!/bin/bash

set -e

export SQLX_OFFLINE=true

cargo set-version --workspace --bump patch
VERSION=`cargo pkgid -p raxb | cut -d "#" -f2`
cargo update
cargo build

git add .
git commit -m "version ${VERSION}"
git push

git tag v${VERSION}
git push --tag
