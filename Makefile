# Captured before anything else is read, so `help` parses this file.
MAKEFILE_PATH := $(lastword $(MAKEFILE_LIST))

.DEFAULT_GOAL := help

CARGO := cargo
CARGO_CLIPPY := $(CARGO) clippy --workspace --all-targets --all-features --locked
# Agents get lint-wip after each edit: the caller of code they just wrote often does not exist yet.
LINT_WIP_ALLOWED := dead_code
# Listed with --others, so a script is linted before its first commit too.
SHELL_SCRIPTS = $(shell git ls-files --cached --others --exclude-standard '*.sh')
TYPST_FILES = $(or $(FILES),$(shell git ls-files --cached --others --exclude-standard '*.typ'))
KARIERA_CONFIG ?= $(HOME)/.config/kariera/config.toml
PROTECTED_REF := refs/heads/main

HOST_OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
HOST_ARCH := $(shell uname -m)
HOST_ARCH_GNU := $(subst arm64,aarch64,$(HOST_ARCH))
RUST_TARGET_VENDOR_OS_darwin := apple-darwin
RUST_TARGET_VENDOR_OS_linux := unknown-linux-gnu
RUST_TARGET := $(HOST_ARCH_GNU)-$(RUST_TARGET_VENDOR_OS_$(HOST_OS))
# Several tools ship their Linux builds as static musl binaries only.
RUST_TARGET_MUSL := $(subst unknown-linux-gnu,unknown-linux-musl,$(RUST_TARGET))

INSTALL_PINNED_BINARY := scripts/install-pinned-binary.sh
PIN_RELEASE_ASSETS := scripts/pin-release-assets.sh

pinned-version = $(shell cat tools/$(1).version)

# One pinned binary from a GitHub release, plus a pin-<tool> target that rewrites its checksum file for a new version.
# $(1) tool, $(2) GitHub repo, $(3) release tag, $(4) release asset for this host,
# $(5) binary path inside the asset (empty if the asset is the binary), $(6) regex of the assets of every platform
# Arguments continued onto a new line start with whitespace, which strip removes.
define pinned-binary
bin/$(1): tools/$(1).version tools/$(1).sha256 $(INSTALL_PINNED_BINARY)
	$(INSTALL_PINNED_BINARY) $$@ https://github.com/$(2)/releases/download/$(3)/$(strip $(4)) tools/$(1).sha256 $(5)

.PHONY: pin-$(1)
pin-$(1): bin/jq
	JQ=bin/jq $(PIN_RELEASE_ASSETS) $(2) $(3) '$(strip $(6))' > tools/$(1).sha256
endef

CARGO_DENY_VERSION := $(call pinned-version,cargo-deny)
CARGO_DENY_ASSET := cargo-deny-$(CARGO_DENY_VERSION)-$(RUST_TARGET_MUSL)
$(eval $(call pinned-binary,cargo-deny,EmbarkStudios/cargo-deny,$(CARGO_DENY_VERSION),$(CARGO_DENY_ASSET).tar.gz,\
	$(CARGO_DENY_ASSET)/cargo-deny,cargo-deny-.*-(aarch64-apple-darwin|aarch64-unknown-linux-musl|x86_64-unknown-linux-musl)\.tar\.gz))

CARGO_MACHETE_TAG := v$(call pinned-version,cargo-machete)
CARGO_MACHETE_ASSET := cargo-machete-$(CARGO_MACHETE_TAG)-$(RUST_TARGET_MUSL)
$(eval $(call pinned-binary,cargo-machete,bnjbvr/cargo-machete,$(CARGO_MACHETE_TAG),$(CARGO_MACHETE_ASSET).tar.gz,\
	$(CARGO_MACHETE_ASSET)/cargo-machete,cargo-machete-.*-(aarch64|x86_64)-(apple-darwin|unknown-linux-musl)\.tar\.gz))

CARGO_NEXTEST_TAG := cargo-nextest-$(call pinned-version,cargo-nextest)
# Upstream ships one universal binary for both Mac architectures.
CARGO_NEXTEST_TARGET := $(if $(filter darwin,$(HOST_OS)),universal-apple-darwin,$(RUST_TARGET))
$(eval $(call pinned-binary,cargo-nextest,nextest-rs/nextest,$(CARGO_NEXTEST_TAG),\
	$(CARGO_NEXTEST_TAG)-$(CARGO_NEXTEST_TARGET).tar.gz,cargo-nextest,\
	cargo-nextest-.*-(universal-apple-darwin|aarch64-unknown-linux-gnu|x86_64-unknown-linux-gnu)\.tar\.gz))

TYPSTYLE_TAG := v$(call pinned-version,typstyle)
$(eval $(call pinned-binary,typstyle,typstyle-rs/typstyle,$(TYPSTYLE_TAG),typstyle-$(RUST_TARGET),,\
	typstyle-(aarch64|x86_64)-(apple-darwin|unknown-linux-gnu)))

GITLEAKS_VERSION := $(call pinned-version,gitleaks)
GITLEAKS_ARCH := $(subst aarch64,arm64,$(subst x86_64,x64,$(HOST_ARCH)))
$(eval $(call pinned-binary,gitleaks,gitleaks/gitleaks,v$(GITLEAKS_VERSION),\
	gitleaks_$(GITLEAKS_VERSION)_$(HOST_OS)_$(GITLEAKS_ARCH).tar.gz,gitleaks,gitleaks_.*_(darwin|linux)_(arm64|x64)\.tar\.gz))

LEFTHOOK_VERSION := $(call pinned-version,lefthook)
LEFTHOOK_OS_darwin := MacOS
LEFTHOOK_OS_linux := Linux
$(eval $(call pinned-binary,lefthook,evilmartians/lefthook,v$(LEFTHOOK_VERSION),\
	lefthook_$(LEFTHOOK_VERSION)_$(LEFTHOOK_OS_$(HOST_OS))_$(HOST_ARCH),,lefthook_.*_(MacOS|Linux)_(arm64|aarch64|x86_64)))

COG_VERSION := $(call pinned-version,cog)
# Upstream ships Linux x86_64 only as musl.
COG_TARGET := $(subst x86_64-unknown-linux-gnu,x86_64-unknown-linux-musl,$(RUST_TARGET))
$(eval $(call pinned-binary,cog,cocogitto/cocogitto,$(COG_VERSION),cocogitto-$(COG_VERSION)-$(COG_TARGET).tar.gz,\
	$(COG_TARGET)/cog,cocogitto-.*-(aarch64-apple-darwin|x86_64-apple-darwin|aarch64-unknown-linux-gnu|x86_64-unknown-linux-musl)\.tar\.gz))

JQ_OS_darwin := macos
JQ_OS_linux := linux
JQ_ARCH := $(subst aarch64,arm64,$(subst x86_64,amd64,$(HOST_ARCH)))
$(eval $(call pinned-binary,jq,jqlang/jq,jq-$(call pinned-version,jq),jq-$(JQ_OS_$(HOST_OS))-$(JQ_ARCH),,\
	jq-(macos|linux)-(arm64|amd64)))

SHELLCHECK_TAG := v$(call pinned-version,shellcheck)
$(eval $(call pinned-binary,shellcheck,koalaman/shellcheck,$(SHELLCHECK_TAG),\
	shellcheck-$(SHELLCHECK_TAG).$(HOST_OS).$(HOST_ARCH_GNU).tar.gz,shellcheck-$(SHELLCHECK_TAG)/shellcheck,\
	shellcheck-.*\.(darwin|linux)\.(aarch64|x86_64)\.tar\.gz))

CARGO_INSTA_VERSION := $(call pinned-version,cargo-insta)
CARGO_INSTA_ASSET := cargo-insta-$(RUST_TARGET)
$(eval $(call pinned-binary,cargo-insta,mitsuhiko/insta,$(CARGO_INSTA_VERSION),$(CARGO_INSTA_ASSET).tar.xz,\
	$(CARGO_INSTA_ASSET)/cargo-insta,cargo-insta-(aarch64|x86_64)-(apple-darwin|unknown-linux-gnu)\.tar\.xz))

# Upstream publishes no Apple Silicon build, so cargo builds it from crates.io. --locked uses the dependency versions
# of the release, and cargo checks every crate against the registry's checksum.
bin/cargo-mutants: tools/cargo-mutants.version
	$(CARGO) install --quiet --locked --version $(call pinned-version,cargo-mutants) --root tmp/cargo-install cargo-mutants
	install -m 0755 tmp/cargo-install/bin/cargo-mutants $@

PINNED_TOOLS := $(addprefix bin/,cargo-deny cargo-machete cargo-nextest cargo-mutants cargo-insta typstyle gitleaks lefthook \
	cog jq shellcheck)

AGENTHOOK_EVENTS := $(addprefix agenthook-,pre-tool-use post-tool-use stop)

##@ General

.PHONY: help
help: ## Show this help
	@awk -f $(dir $(MAKEFILE_PATH))scripts/make-help.awk $(MAKEFILE_PATH)

##@ Setup

.PHONY: setup
setup: tools hooks-install ## Install the pinned toolchain and tools into this checkout and install the git hooks

.PHONY: tools
tools: toolchain $(PINNED_TOOLS) ## Install the pinned Rust toolchain and the pinned tools into bin/

# rustup installs the toolchain rust-toolchain.toml pins on the first cargo command run here.
.PHONY: toolchain
toolchain:
	$(CARGO) --version

.PHONY: hooks-install
hooks-install: bin/lefthook
	bin/lefthook install

##@ Code

.PHONY: fmt
fmt: fmt-rust fmt-typst ## Format Rust and Typst code, or only FILES

.PHONY: fmt-rust
fmt-rust: ## Format Rust code, or only the Rust files in FILES
	$(if $(FILES),rustfmt $(FILES),$(CARGO) fmt --all)

.PHONY: fmt-typst
fmt-typst: bin/typstyle ## Format Typst templates, or only the Typst files in FILES
	$(if $(TYPST_FILES),bin/typstyle --inplace $(TYPST_FILES))

.PHONY: fmt-check
fmt-check: bin/typstyle ## Fail if Rust or Typst code is not formatted
	$(CARGO) fmt --all --check
	$(if $(TYPST_FILES),bin/typstyle --check $(TYPST_FILES))

.PHONY: lint
lint: ## Lint the workspace with Clippy, failing on every warning
	$(CARGO_CLIPPY) -- -D warnings

.PHONY: lint-wip
lint-wip:
	$(CARGO_CLIPPY) --quiet -- -D warnings $(addprefix -A ,$(LINT_WIP_ALLOWED))

.PHONY: deny
deny: deny-dependencies deny-crate-boundaries ## Check the crate allowlist, licences, advisories and the crate boundaries of ADR 0001

.PHONY: deny-dependencies
deny-dependencies: bin/cargo-deny
	bin/cargo-deny --locked check

# Wrappers listed for crates they do not depend on yet are boundaries, not mistakes, so their warning is off.
.PHONY: deny-crate-boundaries
deny-crate-boundaries: bin/cargo-deny
	bin/cargo-deny --locked --all-features --config crate-boundaries.toml check --allow unused-wrapper bans

.PHONY: machete
machete: bin/cargo-machete ## Fail on dependencies no crate uses
	bin/cargo-machete --with-metadata

.PHONY: lint-shell
lint-shell: bin/shellcheck ## Lint every shell script with shellcheck, or only FILES
	bin/shellcheck -x $(or $(FILES),$(SHELL_SCRIPTS))

.PHONY: lint-secrets
lint-secrets: bin/gitleaks ## Scan the staged changes for secrets with gitleaks
	bin/gitleaks git --staged --no-banner --redact --verbose

.PHONY: lint-commit-msg
lint-commit-msg: bin/cog ## Check that the message in COMMIT_MSG_FILE is a Conventional Commit
	bin/cog verify --ignore-fixup-commits --ignore-merge-commits --file $(COMMIT_MSG_FILE)

.PHONY: check
check: fmt-check lint deny machete lint-shell test ## Run every check an agent must pass before it finishes

##@ Test

.PHONY: test
test: test-nextest test-doc ## Run the unit and integration tests with nextest, then the doc tests

.PHONY: test-nextest
test-nextest: bin/cargo-nextest
	bin/cargo-nextest nextest run --workspace --locked

# nextest does not run doc tests.
.PHONY: test-doc
test-doc:
	$(CARGO) test --doc --workspace --locked

# insta writes a changed or new snapshot next to the old one as .snap.new, and the test fails until it is accepted.
.PHONY: snapshots
snapshots: bin/cargo-insta ## Review the snapshots the last test run changed, one by one (interactive)
	bin/cargo-insta review --workspace

.PHONY: snapshots-accept
snapshots-accept: bin/cargo-insta ## Accept every snapshot the last test run changed; read each .snap.new first
	bin/cargo-insta accept --workspace

.PHONY: mutants
mutants: bin/cargo-mutants ## Mutate the code and run the tests against each mutant, writing tmp/mutants.out/
	bin/cargo-mutants mutants --workspace --output tmp

##@ Build

.PHONY: build
build: ## Build the workspace in debug mode
	$(CARGO) build --workspace --locked

.PHONY: install
install: ## Install the kariera binary into cargo's bin directory for this machine
	$(CARGO) install --path crates/cli --locked

.PHONY: check-vault
check-vault: ## Smoke-test kariera against the real vault named in KARIERA_CONFIG (local only, never in hooks)
	scripts/check-vault.sh $(KARIERA_CONFIG)

##@ Upgrade

# cargo-mutants has no checksum file: cargo verifies it against the crates.io index.
.PHONY: pin-tools
pin-tools: $(addprefix pin-,$(filter-out cargo-mutants,$(notdir $(PINNED_TOOLS)))) ## Rewrite every tools/<tool>.sha256 for the release in tools/<tool>.version (needs gh)

# The git hooks (lefthook.yml) and the Claude Code hooks (.claude/settings.json) call these.

.PHONY: refuse-commit-on-main
refuse-commit-on-main:
	scripts/refuse-commit-on-ref.sh $(PROTECTED_REF)

.PHONY: refuse-push-to-main
refuse-push-to-main:
	scripts/refuse-push-to-ref.sh $(PROTECTED_REF)

.PHONY: $(AGENTHOOK_EVENTS)
$(AGENTHOOK_EVENTS): agenthook-%: bin/jq
	JQ=bin/jq scripts/agenthook/$*.sh
