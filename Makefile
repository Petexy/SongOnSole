# A convenience wrapper, and deliberately a thin one.
#
# What a package installs is decided in exactly one place — packaging/install.sh
# — so that `make install` and a .deb cannot come to ship different sets of
# files. Everything here either calls cargo or calls that.

PREFIX ?= /usr/local
DESTDIR ?=
CARGO ?= cargo

.PHONY: build test check run install uninstall clean

build:
	$(CARGO) build --release --locked

test:
	$(CARGO) test --locked
	$(CARGO) fmt --check
	$(CARGO) clippy --locked --all-targets -- -D warnings

# What a package would have to agree with: the version in every recipe, the
# toolkit requirement, the three names this application answers to, and the
# staged payload.
check:
	packaging/build.sh check

run:
	$(CARGO) run --release --locked

install: build
	packaging/install.sh --destdir "$(if $(DESTDIR),$(DESTDIR),/)" --prefix "$(PREFIX)"
	install -Dm0644 LICENSE \
	    "$(DESTDIR)$(PREFIX)/share/licenses/songonsole/LICENSE"
	install -Dm0644 README.md \
	    "$(DESTDIR)$(PREFIX)/share/doc/songonsole/README.md"

uninstall:
	rm -f "$(DESTDIR)$(PREFIX)/bin/songonsole"
	rm -f "$(DESTDIR)$(PREFIX)/share/applications/songonsole.desktop"
	rm -f "$(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/songonsole.svg"
	rm -f "$(DESTDIR)$(PREFIX)/share/metainfo/io.github.petexy.songonsole.metainfo.xml"
	rm -f "$(DESTDIR)$(PREFIX)/share/licenses/songonsole/LICENSE"
	rm -f "$(DESTDIR)$(PREFIX)/share/doc/songonsole/README.md"

clean:
	$(CARGO) clean
	rm -rf packaging/out
