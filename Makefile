.PHONY: help

VERSION := $(shell grep version Cargo.toml | head -n 1 | cut -d '"' -f 2)
PYTHON ?= python3
CARGO ?= cargo
UV ?= uv
MYPY ?= $(UV) run mypy
PYTEST ?= $(UV) run pytest

RUST_DIR = src
PYTHON_DIR = python/structpath
DOC_DIR = docs
RUST_FILES = $(shell find $(RUST_DIR) -type f -name "*.rs")
PYTHON_FILES = $(shell find $(PYTHON_DIR) -type f -name "*.py[i]")
DOC_FILES = $(shell find $(DOC_DIR) -type f -name "*.md") mkdocs.yml

help:
	@echo "Available commands:"
	@echo "  build         Build the Rust library"
	@echo "  build-python  Build the Python library"
	@echo "  test          Run Rust and Python tests"
	@echo "  test-rust     Run Rust tests only"
	@echo "  test-python   Run Python tests only"
	@echo "  verify-types  Run tests for python type annotations"
	@echo "  lint          Run linter"
	@echo "  docs          Build documentation"
	@echo "  docs-serve    Serve documentation locally with live reloading"
	@echo "  clean         Clean build artifacts"
	@echo "  release       Tag a new release"
	@echo "  publish       Publish to crates.io and PyPI"
	@echo "  publish-dev   Publish to TestPyPI"
	@echo "  help          Show this help message"

.build-timestamp: $(RUST_FILES) Cargo.toml
	$(CARGO) build --release
	touch $@

build: .build-timestamp

.python-build-timestamp: .build-timestamp pyproject.toml
	maturin develop --release
	touch $@

build-python: .python-build-timestamp

.PHONY: test test-rust verify-types

test: test-rust test-python

test-rust:
	$(CARGO) test

test-python: build-python
	$(PYTEST) python/tests

test-docs: $(PYTHON_FILES) $(DOC_FILES)
	$(PYTEST) python/tests/test_docs.py

verify-types:
	$(MYPY) --strict python/tests/type_tests.py


.PHONY: lint format

site: $(PYTHON_FILES) $(DOC_FILES)
	$(PYTEST) python/tests/test_docs.py
	$(UV) run python docs/generate_stubs.py
	$(UV) run mkdocs build

docs: site

docs-serve: docs
	$(UV) run mkdocs serve

lint:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy -- -D warnings

format:
	$(CARGO) fmt
	$(UV) run isort .
	$(UV) run ruff format .
	$(UV) run mdformat .


.PHONY: clean release publish publish-dev

check: format lint test verify-types

clean:
	$(CARGO) clean
	rm -rf target/
	rm -rf dist/
	rm -rf site/
	find . -type d -name __pycache__ -exec rm -rf {} +

release: V
release: clean check docs
	@echo "Current version is $(VERSION)"
	@echo "Releasing $(V)"
	sed -i "s/^version = \"$(VERSION)\"/version = \"$(V)\"/" Cargo.toml; \
	git add Cargo.toml; \
	sed -i "s/^version = \"$(VERSION)\"/version = \"$(V)\"/" pyproject.toml; \
	git add pyproject.toml; \
	git commit -m "Bump version to $(V)"; \
	git tag -a "v$new_version" -m "Version $(V)"; \
	git push --tags; \
	echo "Tagged v$(V)."

publish-dev: clean check docs
	@echo "Publishing to TestPyPI..."
	maturin publish --repository-url https://test.pypi.org/legacy/

publish: clean check docs
	@echo "Publishing to crates.io..."
	$(CARGO) publish
	@echo "Publishing to PyPI..."
