%global debug_package %{nil}

Name:           pam-keepassxc
Version:        0.2.0
Release:        1%{?dist}
Summary:        PAM module for auto-unlocking KeePassXC (including KDE & fingerprint support)
License:        MPL-2.0 AND GPL-3.0-or-later
URL:            https://github.com/akeyx/pam-keepassxc
Requires:       pam
Requires:       keepassxc

%description
A PAM module that automatically unlocks your KeePassXC database over D-Bus on
login and screen unlock, with support for KDE Plasma (plasmalogin / kscreenlocker)
and biometric fingerprint unlocking via the Linux user kernel keyring (@u).

%install
install -Dm0755 "%{_sourcedir}/libpam_keepassxc.so" "%{buildroot}%{_libdir}/security/pam_keepassxc.so"
install -Dm0644 "%{_sourcedir}/keepassxc.service" "%{buildroot}%{_userunitdir}/keepassxc.service"
ln -s "keepassxc.service" "%{buildroot}%{_userunitdir}/dbus-org.keepassxc.KeePassXC.MainWindow.service"
install -Dm0644 "%{_sourcedir}/org.keepassxc.KeePassXC.MainWindow.service" "%{buildroot}%{_datadir}/dbus-1/services/org.keepassxc.KeePassXC.MainWindow.service"
install -Dm0644 "%{_sourcedir}/README.md" "%{buildroot}%{_docdir}/%{name}/README.md"
install -Dm0644 "%{_sourcedir}/LICENSE" "%{buildroot}%{_licensedir}/%{name}/LICENSE"

%files
%license %{_licensedir}/%{name}/LICENSE
%doc %{_docdir}/%{name}/README.md
%{_libdir}/security/pam_keepassxc.so
%{_userunitdir}/keepassxc.service
%{_userunitdir}/dbus-org.keepassxc.KeePassXC.MainWindow.service
%{_datadir}/dbus-1/services/org.keepassxc.KeePassXC.MainWindow.service

%changelog
* Tue Oct 27 2026 a.key <a.key@moronet.pl> - 0.2.0-1
- Add Linux user kernel keyring (@u) support for fingerprint screen unlock
- Fix KDE/Wayland screen unlock deadlock by using _exit(0) and close_range
- Fix systemd ExecStart verification for XDG autostart KeePassXC instances
- Retry D-Bus openDatabase during cold-login session open
