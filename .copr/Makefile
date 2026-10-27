.PHONY: srpm
srpm:
	dnf install -y git rpm-build systemd-rpm-macros
	mkdir -p /tmp/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
	VERSION=$$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1); \
	git archive --format=tar.gz --prefix="pam-keepassxc-$${VERSION}/" HEAD > "/tmp/rpmbuild/SOURCES/pam-keepassxc-$${VERSION}.tar.gz"; \
	sed "s/^Version:.*/Version:        $${VERSION}/" rpm/pam-keepassxc.spec > /tmp/rpmbuild/SPECS/pam-keepassxc.spec; \
	rpmbuild -bs --define "_topdir /tmp/rpmbuild" /tmp/rpmbuild/SPECS/pam-keepassxc.spec
	cp /tmp/rpmbuild/SRPMS/*.src.rpm "$(outdir)/"
