# pam-keepassxc

A PAM module for automatically unlocking your KeePassXC database on login and screen unlock over D-Bus, with native support for **KDE Plasma** (`plasmalogin` / `sddm` / `kscreenlocker`) and **biometric fingerprint unlocking** (`pam_fprintd.so`) via the volatile Linux user kernel keyring (`@u`).

## Features

- **Cold Login & Screen Unlock**: Unlocks your KeePassXC database automatically on initial desktop login (`plasmalogin`, `sddm`, `greetd`) and screen unlock (`/etc/pam.d/kde`, `swaylock`).
- **Fingerprint Unlock Support**: Caches your password in the in-memory Linux user kernel keyring (`@u`) upon password login, allowing biometric screen unlock (`/etc/pam.d/kde-fingerprint`) to unlock KeePassXC without storing secrets on disk.
- **Non-blocking PAM Execution**: Detaches cleanly via `_exit(0)` and `close_range` so GUI screen lockers (`kscreenlocker_greet`) dismiss instantaneously while KeePassXC finishes Argon2id decryption in the background.

## Installation (Fedora / RPM)

### Option 1: Install from Fedora COPR (Recommended)
```bash
sudo dnf copr enable akey/pam-keepassxc
sudo dnf install pam-keepassxc
```

### Option 2: Build and Install the RPM Locally
```bash
./rpm/build-rpm.sh
sudo dnf install -y ./target/rpm/RPMS/x86_64/pam-keepassxc-*.x86_64.rpm
```
*(Or download the pre-built `.rpm` / `.deb` from GitHub Releases.)*

### Option 3: Install & Configure PAM Automatically (KDE Plasma)
After building (`cargo build --release`), run:
```bash
sudo ./INSTALL.sh
```
This installs `/usr/lib64/security/pam_keepassxc.so` and configures `/etc/pam.d/kde`, `/etc/pam.d/kde-fingerprint`, and `/etc/pam.d/plasmalogin` (saving backups to `/etc/pam.d/*.backup`).

## Manual PAM Configuration (KDE Plasma)

1. **`/etc/pam.d/kde`** (Password screen unlock):
   Add `-auth optional pam_keepassxc.so` after `auth substack password-auth`:
   ```pam
   auth        substack      password-auth
   -auth       optional      pam_keepassxc.so
   ```

2. **`/etc/pam.d/kde-fingerprint`** (Fingerprint screen unlock):
   Add `-auth optional pam_keepassxc.so` after `auth substack fingerprint-auth`:
   ```pam
   auth        substack      fingerprint-auth
   -auth       optional      pam_keepassxc.so
   ```

3. **`/etc/pam.d/plasmalogin`** (or `/etc/pam.d/sddm`):
   Add `-auth optional pam_keepassxc.so` at the end of the `auth` stack (before `postlogin`) and `-session optional pam_keepassxc.so` at the end of the `session` stack (before `postlogin`).

## User Configuration

Create `$HOME/.config/security/pam_keepassxc.toml`:
```toml
database_path = "$HOME/.keepass/Passwords.kdbx"
keyfile_path = "$HOME/.keepass/Passwords.keyx"  # optional
```

Ensure KeePassXC has **Enable D-Bus integration** checked under **Tools → Settings → General → Basic Settings**, and that your database master password matches your Linux login password.

## License

The source code in this repository is licensed under the [Mozilla Public License 2.0 (MPL-2.0)](LICENSE). Due to the inclusion of the GPLv3-licensed `pamsm` crate, compiled binaries are distributed under the terms of the GPLv3.0.
