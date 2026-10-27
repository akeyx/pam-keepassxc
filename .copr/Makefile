.PHONY: srpm
srpm:
	dnf install -y git rpm-build systemd-rpm-macros
	mkdir -p /tmp/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
	git archive --format=tar.gz --prefix=pam-keepassxc-0.2.0/ HEAD > /tmp/rpmbuild/SOURCES/pam-keepassxc-0.2.0.tar.gz
	cp rpm/pam-keepassxc.spec /tmp/rpmbuild/SPECS/
	rpmbuild -bs --define "_topdir /tmp/rpmbuild" /tmp/rpmbuild/SPECS/pam-keepassxc.spec
	cp /tmp/rpmbuild/SRPMS/*.src.rpm "$(outdir)/"
