.ONESHELL:
.PHONY: all README.md

all: README.md

README.md: README.md.hms examples/hello.hms
	cargo run --quiet -- $< > $@

examples/hello.hms: examples/hello.hms.hms examples/hello/*
	cargo run --quiet -- $< > $@
