PREFIX ?= /usr/local
BINARY = fbench
DESKTOP = assets/fbench.desktop

all:
	cargo build --release

install:
	install -Dm755 target/release/$(BINARY) $(DESTDIR)$(PREFIX)/bin/$(BINARY)
	install -Dm644 $(DESKTOP) $(DESTDIR)/usr/share/applications/$(DESKTOP)

uninstall:
	rm -f $(PREFIX)/bin/$(BINARY)
	rm -f /usr/share/applications/$(DESKTOP)
