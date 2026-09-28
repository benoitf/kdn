CARGO ?= cargo

.PHONY: build release test lint fmt ci-checks install clean

build:
	$(CARGO) build

release:
	$(CARGO) build --release

test:
	$(CARGO) test --locked

lint:
	$(CARGO) fmt --check
	$(CARGO) clippy --all-targets --locked -- -D warnings

fmt:
	$(CARGO) fmt

install:
	$(CARGO) install --path .

clean:
	$(CARGO) clean

ci-checks: lint test

