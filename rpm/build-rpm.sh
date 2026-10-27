#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RPM_TOPDIR="$PROJECT_DIR/target/rpm"

cd "$PROJECT_DIR"

VERSION="${RELEASE_VERSION:-$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || echo "0.0.0")}"

restore_files() {
    git checkout -- Cargo.toml Cargo.lock 2>/dev/null || true
}
trap restore_files EXIT

echo "Stamping version ${VERSION} into Cargo.toml and Cargo.lock..."
sed -i "0,/^version = .*/s//version = \"${VERSION}\"/" Cargo.toml
sed -i "/^name = \"pam-keepassxc\"$/{n;s/^version = .*/version = \"${VERSION}\"/;}" Cargo.lock

echo "Preparing rpmbuild tree for version ${VERSION}..."
rm -rf "$RPM_TOPDIR"
mkdir -p "$RPM_TOPDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

tar --exclude="./target" --exclude="./.git" \
    --transform="s,^\.,pam-keepassxc-${VERSION}," \
    -czf "$RPM_TOPDIR/SOURCES/pam-keepassxc-${VERSION}.tar.gz" .

sed "s/^Version:.*/Version:        ${VERSION}/" "$SCRIPT_DIR/pam-keepassxc.spec" \
    > "$RPM_TOPDIR/SPECS/pam-keepassxc.spec"

echo "Building SRPM and binary RPM packages..."
rpmbuild -ba \
    --nodeps \
    --define "_topdir $RPM_TOPDIR" \
    "$RPM_TOPDIR/SPECS/pam-keepassxc.spec"

echo "RPM packages built successfully:"
ls -lh "$RPM_TOPDIR"/SRPMS/*.src.rpm "$RPM_TOPDIR"/RPMS/*/*.rpm
