.PHONY: srpm
srpm:
	dnf install -y git rpm-build systemd-rpm-macros
	mkdir -p /tmp/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
	VERSION=$$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//' || echo "0.0.0"); \
	sed -i "0,/^version = .*/s//version = \"$${VERSION}\"/" Cargo.toml; \
	sed -i "/^name = \"pam-keepassxc\"$$/{n;s/^version = .*/version = \"$${VERSION}\"/;}" Cargo.lock; \
	tar --exclude="./target" --exclude="./.git" --transform="s,^\.,pam-keepassxc-$${VERSION}," -czf "/tmp/rpmbuild/SOURCES/pam-keepassxc-$${VERSION}.tar.gz" .; \
	sed "s/^Version:.*/Version:        $${VERSION}/" rpm/pam-keepassxc.spec > /tmp/rpmbuild/SPECS/pam-keepassxc.spec; \
	rpmbuild -bs --define "_topdir /tmp/rpmbuild" /tmp/rpmbuild/SPECS/pam-keepassxc.spec
	cp /tmp/rpmbuild/SRPMS/*.src.rpm "$(outdir)/"
