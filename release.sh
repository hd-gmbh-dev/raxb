#!/bin/bash

set -e

RELEASE_TYPE=${RELEASE_TYPE:-patch}
if [ "$RELEASE_TYPE" != "current" ]; then
    cargo set-version --workspace --bump ${RELEASE_TYPE}
fi
VERSION=`cargo pkgid -p raxb | cut -d "#" -f2`
cargo update
cargo build

git add .
git commit -m "version ${VERSION}"
git push

git tag v${VERSION}
git push --tag
