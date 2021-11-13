.ONESHELL:
.PHONY: all README.md

all: README.md

README.md: README.md.hms
	cargo run --quiet -- $< > $@
