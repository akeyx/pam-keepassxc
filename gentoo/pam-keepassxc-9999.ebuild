# Copyright 2025-2026 Gentoo Authors
# Distributed under the terms of the GNU General Public License v2

EAPI=8

DESCRIPTION="PAM module for auto-unlocking KeePassXC"
HOMEPAGE="https://github.com/akeyx/pam-keepassxc"

CRATES="
	anyhow@1.0.90
	autocfg@1.4.0
	basic-toml@0.1.9
	bitflags@1.3.2
	cfg-if@1.0.0
	deranged@0.3.11
	hostname@0.4.0
	itoa@1.0.11
	libc@0.2.161
	log@0.4.22
	memoffset@0.7.1
	nix@0.26.4
	num-conv@0.1.0
	num_threads@0.1.7
	pamsm@0.5.5
	pin-utils@0.1.0
	powerfmt@0.2.0
	proc-macro2@1.0.88
	quote@1.0.37
	rustbus@0.19.3
	rustbus_derive@0.5.0
	secrecy@0.10.3
	serde@1.0.210
	serde_derive@1.0.210
	syn@1.0.109
	syn@2.0.79
	syslog@7.0.0
	thiserror-impl@1.0.64
	thiserror@1.0.64
	time-core@0.1.2
	time-macros@0.2.18
	time@0.3.36
	unicode-ident@1.0.13
	zeroize@1.8.1
"

inherit cargo git-r3 systemd

EGIT_REPO_URI="https://github.com/akeyx/pam-keepassxc.git"

LICENSE="GPL-3 MPL-2.0"
LICENSE+=" GPL-3 MIT Unicode-DFS-2016"
SLOT="0"
KEYWORDS="~amd64 ~arm64"

DEPEND="sys-libs/pam"
RDEPEND="${DEPEND}"

src_unpack() {
	git-r3_src_unpack
	cargo_live_src_unpack
}

src_install() {
	exeinto /lib64/security
	newexe "target/release/libpam_keepassxc.so" pam_keepassxc.so

	systemd_douserunit systemd/keepassxc.service
	dosym keepassxc.service "$(systemd_get_userunitdir)/dbus-org.keepassxc.KeePassXC.MainWindow.service"

	insinto /usr/share/dbus-1/services
	doins systemd/org.keepassxc.KeePassXC.MainWindow.service

	einstalldocs
}
