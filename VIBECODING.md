---
version: "0.1.1"
level: copilot
processes:
  design: pair
  implementation: copilot
  testing: copilot
  documentation: copilot
  review: hint
  deployment: none
convention:
  version: "0.1"
  level: 2
  adopted_at: ADOPTED_AT
  status: active
  tools:
    - "Claude Code:claude-opus-5-5"
    - "local:qwen3.8-27b"
  attribution_trailers: [Assisted-by]
  maintainers: [ikarys]
  review: "single maintainer reads every diff; changes reach main by pull request"
  security_response: "7 days"
  licence: AGPL-3.0-only
  coverage: { threshold: 80, kind: line, tool: cargo-llvm-cov }
  maintainability: { tool: "clippy cognitive_complexity", duplication_max: 4, complexity_max: 25, function_max: 110, file_max: 800, markers_max: 0 }
  freshness: { max_outdated: 2 }
  evidence:
    coverage: audit/evidence/coverage.txt
    audit: audit/evidence/audit.json
    outdated: audit/evidence/outdated.txt
  commands:
    test: "just test"
    lint: "just lint"
    format: "just lint"
    coverage: "cargo llvm-cov --all-features --summary-only"
    audit: "cargo audit"
    static_analysis: "just lint"
    secrets: "gitleaks git --no-banner --redact ."
    outdated: "cargo outdated --root-deps-only"
    doc_lint: "just doc"
---

# Vibecoding declaration

This project follows the [Vibecoding Convention](https://github.com/ikarys/vibecoding-convention) version 0.1 at level 2. How each quality claim is checked is in [VERIFICATION.md](VERIFICATION.md); the security claims are in [SECURITY.md](SECURITY.md).

## AI assistance

mcp-lite was extracted from fennec, and most of its code is AI-written. The maintainer frames each change and decides; Claude Code reviews and fixes; a local model (qwen3.8-27b) writes bulk code. Every AI-written commit carries an `Assisted-by: <tool>:<model>` trailer.

## Review

Single maintainer. Every diff is read before it is merged into `main` by a pull request; `just check` must pass first, locally and in CI.

## Provenance and licence

This project is licensed under AGPL-3.0-only. Parts of it are written by AI assistants and may not be protected by copyright in some jurisdictions; the maintainer accepts this. To the maintainer's knowledge, no verbatim reproduction of licence-incompatible code was included. Safeguards used: the maintainer reads every diff.

## Status

Active, early. Vulnerabilities are reported privately and acknowledged within 7 days ([SECURITY.md](SECURITY.md)).

## Exceptions

- D3: the history starts at the extraction from fennec on 2026-11-13; earlier history of this code is in fennec's repository. Declared 2026-11-13.
- T4: no end-to-end test against third-party MCP servers; the client is tested against scripted stdio servers and local HTTP listeners. Declared 2026-11-13.
