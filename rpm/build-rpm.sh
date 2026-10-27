#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RPM_TOPDIR="$PROJECT_DIR/target/rpm"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$PROJECT_DIR/Cargo.toml" | head -n1)"

cd "$PROJECT_DIR"

echo "Preparing rpmbuild tree for version ${VERSION}..."
rm -rf "$RPM_TOPDIR"
mkdir -p "$RPM_TOPDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

git archive --format=tar.gz --prefix="pam-keepassxc-${VERSION}/" HEAD \
    > "$RPM_TOPDIR/SOURCES/pam-keepassxc-${VERSION}.tar.gz"
sed "s/^Version:.*/Version:        ${VERSION}/" "$SCRIPT_DIR/pam-keepassxc.spec" \
    > "$RPM_TOPDIR/SPECS/pam-keepassxc.spec"

echo "Building SRPM and binary RPM packages..."
rpmbuild -ba \
    --nodeps \
    --define "_topdir $RPM_TOPDIR" \
    "$RPM_TOPDIR/SPECS/pam-keepassxc.spec"

echo "RPM packages built successfully:"
ls -lh "$RPM_TOPDIR"/SRPMS/*.src.rpm "$RPM_TOPDIR"/RPMS/*/*.rpm
