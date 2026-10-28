%global debug_package %{nil}

Name:           pam-keepassxc
Version:        0.2.1
Release:        1%{?dist}
Summary:        PAM module for auto-unlocking KeePassXC (including KDE & fingerprint support)
License:        MPL-2.0 AND GPL-3.0-or-later
URL:            https://github.com/akeyx/pam-keepassxc
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  pam-devel
BuildRequires:  systemd-rpm-macros
BuildRequires:  checkpolicy
BuildRequires:  policycoreutils

Requires:       pam
Requires:       keepassxc
Requires(post):   policycoreutils
Requires(postun): policycoreutils

%description
A PAM module that automatically unlocks your KeePassXC database over D-Bus on
login and screen unlock, with support for KDE Plasma (plasmalogin / kscreenlocker)
and biometric fingerprint unlocking via the Linux user kernel keyring (@u).

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release --locked
checkmodule -M -m -o pam_keepassxc.mod selinux/pam_keepassxc.te
semodule_package -o pam_keepassxc.pp -m pam_keepassxc.mod

%install
install -Dm0755 "target/release/libpam_keepassxc.so" "%{buildroot}%{_libdir}/security/pam_keepassxc.so"
install -Dm0644 "systemd/keepassxc.service" "%{buildroot}%{_userunitdir}/keepassxc.service"
ln -s "keepassxc.service" "%{buildroot}%{_userunitdir}/dbus-org.keepassxc.KeePassXC.MainWindow.service"
install -Dm0644 "systemd/org.keepassxc.KeePassXC.MainWindow.service" "%{buildroot}%{_datadir}/dbus-1/services/org.keepassxc.KeePassXC.MainWindow.service"
install -Dm0644 "pam_keepassxc.pp" "%{buildroot}%{_datadir}/selinux/packages/pam_keepassxc.pp"

%post
%selinux_modules_install %{_datadir}/selinux/packages/pam_keepassxc.pp

%postun
%selinux_modules_uninstall pam_keepassxc

%files
%license LICENSE
%doc README.md
%{_libdir}/security/pam_keepassxc.so
%{_userunitdir}/keepassxc.service
%{_userunitdir}/dbus-org.keepassxc.KeePassXC.MainWindow.service
%{_datadir}/dbus-1/services/org.keepassxc.KeePassXC.MainWindow.service
%{_datadir}/selinux/packages/pam_keepassxc.pp

%changelog
* Wed Oct 28 2026 a.key <a.key@moronet.pl> - 0.2.1-1
- Add SELinux policy module to allow xdm_t service status queries on cold login

* Tue Oct 27 2026 a.key <a.key@moronet.pl> - 0.2.0-1
- Add Linux user kernel keyring (@u) support for fingerprint screen unlock
- Fix KDE/Wayland screen unlock deadlock by using _exit(0) and close_range
- Fix systemd ExecStart verification for XDG autostart KeePassXC instances
- Retry D-Bus openDatabase during cold-login session open
