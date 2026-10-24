# SPDX-License-Identifier: MIT OR Apache-2.0
# Tools and build configuration
CARGO ?= cargo
CC ?= cc
TARGET ?=
CARGO_TARGET_ARG = $(if $(strip $(TARGET)),--target $(TARGET),)

NAPI_FIXTURE_DIR := tests/fixtures/napi
NAPI_FIXTURE_ADDON := $(NAPI_FIXTURE_DIR)/addon.node
UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Darwin)
NAPI_FIXTURE_LDFLAGS := -undefined dynamic_lookup
else
NAPI_FIXTURE_LDFLAGS :=
endif

.DEFAULT_GOAL := help

.PHONY: help build release check lint test test-napi fix ci clean

help:
	@printf '%s\n' \
		'Rust:' \
		'  lint       Check formatting and run Clippy' \
		'  build      Build all targets (optional: TARGET=<triple>)' \
		'  check      Check all targets (optional: TARGET=<triple>)' \
		'  test       Run Rust unit tests' \
		'  release    Build all targets in release mode (optional: TARGET=<triple>)' \
		'  fix        Apply Cargo fixes, Clippy fixes, and formatting' \
		'Integration:' \
		'  test-napi  Build the C fixture and test it through NapiLoader' \
		'  ci         Run lint, build, unit, and integration tests' \
		'Maintenance:' \
		'  clean      Remove Cargo and test addon build artifacts'

# Rust checks and builds
lint:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --all-targets --all-features -- -D warnings

build:
	$(CARGO) build --all-targets $(CARGO_TARGET_ARG)

release:
	$(CARGO) build --release --all-targets $(CARGO_TARGET_ARG)

check:
	$(CARGO) check --all-targets $(CARGO_TARGET_ARG)

test:
	$(CARGO) test --lib

fix:
	$(CARGO) fix --allow-dirty --all-targets
	$(CARGO) clippy --fix --allow-dirty --all-targets --all-features
	$(CARGO) fmt --all

# N-API integration test
$(NAPI_FIXTURE_ADDON): $(NAPI_FIXTURE_DIR)/addon.c
	$(CC) -O2 -shared -fPIC $(NAPI_FIXTURE_LDFLAGS) -pthread -o $@ $<

test-napi: $(NAPI_FIXTURE_ADDON)
	$(CARGO) test --test napi_loader -- --ignored --nocapture

# Aggregate and cleanup
ci: lint build test test-napi

clean:
	$(CARGO) clean
	rm -f $(NAPI_FIXTURE_ADDON)
