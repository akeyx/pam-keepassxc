#!/usr/bin/env bash
# Installation script for pam-keepassxc (Plasma Login Manager + KDE Screen Locker + Fingerprint)
# Run with sudo: sudo /home/a.key/projects/a.key/pam-keepassxc/INSTALL.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/target/release"
PAM_MODULE_NAME="libpam_keepassxc.so"
PAM_INSTALL_DIR="/usr/lib64/security"

if [ ! -f "$BUILD_DIR/$PAM_MODULE_NAME" ]; then
    echo "Error: PAM module not found at $BUILD_DIR/$PAM_MODULE_NAME"
    echo "Please build it first with: cargo build --release"
    exit 1
fi

echo "1. Installing $PAM_INSTALL_DIR/pam_keepassxc.so..."
install -m 755 -Z "$BUILD_DIR/$PAM_MODULE_NAME" "$PAM_INSTALL_DIR/pam_keepassxc.so"

echo "2. Configuring /etc/pam.d/kde (password screen unlock)..."
cp -a /etc/pam.d/kde /etc/pam.d/kde.backup
cat > /etc/pam.d/kde << 'EOF'
auth        substack      password-auth
-auth       optional      pam_keepassxc.so
auth        include       postlogin

account     required      pam_nologin.so
account     include       password-auth

password    include       password-auth

session     required      pam_selinux.so close
session     required      pam_loginuid.so
session     required      pam_selinux.so open
session     optional      pam_keyinit.so force revoke
session     required      pam_namespace.so
session     include       password-auth
session     include       postlogin
EOF

echo "3. Configuring /etc/pam.d/kde-fingerprint (fingerprint screen unlock)..."
cp -a /etc/pam.d/kde-fingerprint /etc/pam.d/kde-fingerprint.backup
cat > /etc/pam.d/kde-fingerprint << 'EOF'
auth        substack      fingerprint-auth
-auth       optional      pam_keepassxc.so
auth        include       postlogin

account     required      pam_nologin.so
account     include       fingerprint-auth

password    include       fingerprint-auth

session     required      pam_selinux.so close
session     required      pam_loginuid.so
session     required      pam_selinux.so open
session     optional      pam_keyinit.so force revoke
session     required      pam_namespace.so
session     include       fingerprint-auth
session     include       postlogin
EOF

echo "4. Configuring /etc/pam.d/plasmalogin (Plasma Login Manager)..."
if [ -f /etc/pam.d/plasmalogin ]; then
    cp -a /etc/pam.d/plasmalogin /etc/pam.d/plasmalogin.backup
fi
cat > /etc/pam.d/plasmalogin << 'EOF'
auth     [success=done ignore=ignore default=bad] pam_selinux_permit.so
auth        substack      password-auth
-auth        optional      pam_gnome_keyring.so
-auth        optional      pam_kwallet5.so
-auth        optional      pam_kwallet.so
-auth        optional      pam_oo7.so
-auth        optional      pam_keepassxc.so
auth        include       postlogin

account     required      pam_nologin.so
account     include       password-auth

password    include       password-auth

session     required      pam_selinux.so close
session     required      pam_loginuid.so
-session    optional    pam_ck_connector.so
session     required      pam_selinux.so open
session     optional      pam_keyinit.so force revoke
session     required      pam_namespace.so
session     include       password-auth
-session     optional      pam_gnome_keyring.so auto_start
-session     optional      pam_kwallet5.so auto_start
-session     optional      pam_kwallet.so auto_start
-session     optional      pam_oo7.so auto_start
-session     optional      pam_keepassxc.so
session     include       postlogin
EOF

restorecon -v "$PAM_INSTALL_DIR/pam_keepassxc.so" /etc/pam.d/kde /etc/pam.d/kde-fingerprint /etc/pam.d/plasmalogin

echo "Done! Backups saved to /etc/pam.d/*.backup"
