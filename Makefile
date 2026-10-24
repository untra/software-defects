SERVER_DIR := tooling/state-lens
SERVER_MANIFEST := $(SERVER_DIR)/Cargo.toml
SERVER_RELEASE := $(SERVER_DIR)/target/release/state-lens
INSTALL_ROOT ?= $(HOME)/.local
VSCODE_DIR := extensions/vscode
VSCODE_VERSION := $(shell node -p "require('./$(VSCODE_DIR)/package.json').version")
VSIX := $(VSCODE_DIR)/state-lens-$(VSCODE_VERSION).vsix
ZED_MANIFEST := extensions/zed/Cargo.toml
ZED_TARGET := wasm32-wasip2

.PHONY: all server install-server vscode install-vscode zed test install

all: server vscode zed

server:
	cargo build --release --locked --manifest-path $(SERVER_MANIFEST)
	$(SERVER_RELEASE) check-queries

install-server:
	cargo install --locked --force --path $(SERVER_DIR) --root $(INSTALL_ROOT)

vscode:
	npm ci --prefix $(VSCODE_DIR)
	npm run package --prefix $(VSCODE_DIR)

install-vscode: vscode
	code --install-extension $(VSIX) --force

zed:
	cargo build --target $(ZED_TARGET) --manifest-path $(ZED_MANIFEST)

install: install-server install-vscode
	@echo "Zed: run 'zed: install dev extension' and choose extensions/zed"

test:
	cargo fmt --manifest-path $(SERVER_MANIFEST) --check
	cargo clippy --locked --manifest-path $(SERVER_MANIFEST) --all-targets -- -D warnings
	cargo test --locked --manifest-path $(SERVER_MANIFEST)
	cargo run --locked --manifest-path $(SERVER_MANIFEST) -- score $(SERVER_DIR)/fixtures/expectations.json
	python3 -m unittest discover -s $(SERVER_DIR)/scripts -p 'test_*.py'
	npm test --prefix $(VSCODE_DIR)
	cargo fmt --manifest-path $(ZED_MANIFEST) --check
	cargo clippy --target $(ZED_TARGET) --manifest-path $(ZED_MANIFEST) -- -D warnings

SMOKE_REPORT := build/smoke

.PHONY: smoke smoke-audit smoke-update

smoke: server vscode
	@status=0; \
	python3 $(SERVER_DIR)/scripts/smoke.py --report $(SMOKE_REPORT)/protocol || status=1; \
	npm run test:e2e --prefix $(VSCODE_DIR) -- --report $(CURDIR)/$(SMOKE_REPORT)/editor || status=1; \
	python3 $(SERVER_DIR)/scripts/smoke_report.py $(SMOKE_REPORT)/protocol/smoke-report.json $(SMOKE_REPORT)/editor/smoke-report.json --report $(SMOKE_REPORT) || status=1; \
	exit $$status

smoke-audit: smoke
	@echo "$(SMOKE_REPORT)/smoke-report.md"

smoke-update: server
	python3 $(SERVER_DIR)/scripts/smoke.py --update
