DESTDIR =
PREFIX = /usr/local
CARGO_FLAGS =

.PHONY: all target/release/raisen install help

all: target/release/raisen

target/release/raisen:
	cargo build --release $(CARGO_FLAGS)

install: target/release/raisen
	install -s -D -m755 -- target/release/raisen "$(DESTDIR)$(PREFIX)/bin/raisen"
	install -D -m644 -- man/raisen.1 "$(DESTDIR)$(PREFIX)/share/man/man1/raisen.1"

help:
	@echo "Available make targets:"
	@echo "  all      - Build raisen (default)"
	@echo "  install  - Build and install raisen and manual pages"
	@echo "  help     - Print this help"
