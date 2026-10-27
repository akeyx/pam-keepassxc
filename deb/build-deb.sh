#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$PROJECT_DIR/Cargo.toml" | head -n1)"
PKG_NAME="pam-keepassxc_${VERSION}-1_amd64"
DEB_ROOT="$PROJECT_DIR/target/deb/$PKG_NAME"

cd "$PROJECT_DIR"

echo "Building release binary..."
cargo build --release --locked

echo "Preparing Debian package tree..."
rm -rf "$PROJECT_DIR/target/deb"
mkdir -p "$DEB_ROOT/DEBIAN"
mkdir -p "$DEB_ROOT/usr/lib/x86_64-linux-gnu/security"
mkdir -p "$DEB_ROOT/usr/lib/systemd/user"
mkdir -p "$DEB_ROOT/usr/share/dbus-1/services"
mkdir -p "$DEB_ROOT/usr/share/doc/pam-keepassxc"

install -m0755 "$PROJECT_DIR/target/release/libpam_keepassxc.so" \
    "$DEB_ROOT/usr/lib/x86_64-linux-gnu/security/pam_keepassxc.so"
install -m0644 "$PROJECT_DIR/systemd/keepassxc.service" \
    "$DEB_ROOT/usr/lib/systemd/user/keepassxc.service"
ln -s "keepassxc.service" \
    "$DEB_ROOT/usr/lib/systemd/user/dbus-org.keepassxc.KeePassXC.MainWindow.service"
install -m0644 "$PROJECT_DIR/systemd/org.keepassxc.KeePassXC.MainWindow.service" \
    "$DEB_ROOT/usr/share/dbus-1/services/org.keepassxc.KeePassXC.MainWindow.service"
install -m0644 "$PROJECT_DIR/README.md" \
    "$DEB_ROOT/usr/share/doc/pam-keepassxc/README.md"
install -m0644 "$PROJECT_DIR/LICENSE" \
    "$DEB_ROOT/usr/share/doc/pam-keepassxc/copyright"

cat > "$DEB_ROOT/DEBIAN/control" << EOF
Package: pam-keepassxc
Version: ${VERSION}-1
Section: admin
Priority: optional
Architecture: amd64
Depends: libc6, libpam0g, keepassxc
Maintainer: a.key <a.key@moronet.pl>
Homepage: https://github.com/akeyx/pam-keepassxc
Description: PAM module for auto-unlocking KeePassXC
 Automatically unlocks your KeePassXC database over D-Bus on login and screen
 unlock, including KDE Plasma and fingerprint unlock via the Linux user keyring.
EOF

dpkg-deb --root-owner-group --build "$DEB_ROOT"
echo "DEB built successfully:"
ls -lh "$PROJECT_DIR/target/deb/${PKG_NAME}.deb"
