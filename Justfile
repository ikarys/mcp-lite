# Run tests on file changes.
dev:
    cargo watch -x test 2>/dev/null || cargo test

# Build the crate.
build:
    cargo build

# Run all test configurations.
test:
    cargo test --all-features
    cargo test --no-default-features
    cargo test

# Check formatting and lints.
lint:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo clippy --no-default-features -- -D warnings

# Build documentation with warnings denied.
doc:
    RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features

# Run lint, tests, and docs.
check:
    just lint
    just test
    just doc

# Scan for secrets.
secrets:
    gitleaks git --no-banner --redact .

# Install the pre-commit secrets hook.
setup:
    git config core.hooksPath .githooks

# Write coverage, audit, and outdated evidence, then the convention status (run before a release:
# CI runs on the GitHub mirror and cannot commit back to Forgejo).
evidence:
    mkdir -p audit/evidence
    cargo llvm-cov --all-features --summary-only > audit/evidence/coverage.txt
    cargo audit --json > audit/evidence/audit.json || true
    cargo outdated --root-deps-only > audit/evidence/outdated.txt
    vibecheck . --write-status

# Check publishing; the real publish is a human decision.
publish:
    cargo publish --dry-run
