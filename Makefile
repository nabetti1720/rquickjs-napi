# SPDX-License-Identifier: MIT OR Apache-2.0
CARGO ?= cargo
CC ?= cc
TARGET ?=
CARGO_TARGET_ARG = $(if $(strip $(TARGET)),--target $(TARGET),)

NAPI_FIXTURE_DIR := tests/fixtures/napi
NAPI_FIXTURE_ADDON := $(NAPI_FIXTURE_DIR)/addon.node
UNAME_S := $(shell uname -s)

ifeq ($(UNAME_S),Darwin)
SAMPLE_LDFLAGS := -undefined dynamic_lookup
else
SAMPLE_LDFLAGS :=
endif

.DEFAULT_GOAL := help

.PHONY: help build release check test test-napi fix fixture-build ci clean

help:
	@printf '%s\n' \
		'Available targets:' \
		'  build        Build the crate (optional: TARGET=<triple>)' \
		'  release      Build the crate in release mode (optional: TARGET=<triple>)' \
		'  check        Run Clippy and check compilation' \
		'  test         Run all Rust tests' \
		'  test-napi    Load and test the N-API sample in rquickjs' \
		'  fix          Apply Cargo fixes, Clippy fixes, and formatting' \
		'  fixture-build Build the N-API test addon' \
		'  ci            Run checks, Rust tests, and N-API integration test' \
		'  clean         Remove Cargo and test addon build artifacts'

build:
	$(CARGO) build $(CARGO_TARGET_ARG)

release:
	$(CARGO) build --release $(CARGO_TARGET_ARG)

check:
	$(CARGO) clippy --all-targets --all-features -- -D warnings
	$(CARGO) check --all-targets $(CARGO_TARGET_ARG)

test:
	$(CARGO) test --all-targets

fix:
	$(CARGO) fix --allow-dirty --all-targets
	$(CARGO) clippy --fix --allow-dirty --all-targets --all-features
	$(CARGO) fmt --all

$(NAPI_FIXTURE_ADDON): $(NAPI_FIXTURE_DIR)/addon.c
	$(CC) -O2 -shared -fPIC $(SAMPLE_LDFLAGS) -pthread -o $@ $<

fixture-build: $(NAPI_FIXTURE_ADDON)

test-napi: fixture-build
	$(CARGO) test --test napi_loader -- --ignored --nocapture

ci: check test test-napi

clean:
	$(CARGO) clean
	rm -f $(NAPI_FIXTURE_ADDON)
