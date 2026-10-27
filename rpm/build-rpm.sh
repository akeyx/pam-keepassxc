#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RPM_TOPDIR="$PROJECT_DIR/target/rpm"

cd "$PROJECT_DIR"

echo "Building release binary..."
cargo build --release --locked

echo "Preparing rpmbuild tree..."
rm -rf "$RPM_TOPDIR"
mkdir -p "$RPM_TOPDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

cp "$PROJECT_DIR/target/release/libpam_keepassxc.so" "$RPM_TOPDIR/SOURCES/"
cp "$PROJECT_DIR/systemd/keepassxc.service" "$RPM_TOPDIR/SOURCES/"
cp "$PROJECT_DIR/systemd/org.keepassxc.KeePassXC.MainWindow.service" "$RPM_TOPDIR/SOURCES/"
cp "$PROJECT_DIR/README.md" "$RPM_TOPDIR/SOURCES/"
cp "$PROJECT_DIR/LICENSE" "$RPM_TOPDIR/SOURCES/"
cp "$SCRIPT_DIR/pam-keepassxc.spec" "$RPM_TOPDIR/SPECS/"

echo "Building RPM package..."
rpmbuild -bb \
    --define "_topdir $RPM_TOPDIR" \
    "$RPM_TOPDIR/SPECS/pam-keepassxc.spec"

echo "RPM built successfully:"
ls -lh "$RPM_TOPDIR"/RPMS/*/*.rpm
