# Verification

How mcp-lite is built and verified. AI assistance is stated openly in [VIBECODING.md](VIBECODING.md); trust comes from re-runnable checks, not from the model that wrote the code.

| Requirement | Verified by |
|---|---|
| Format and lint | `just lint` |
| Tests, including README doctests | `just test` |
| Public API documentation | `just doc` |
| Security claims | [SECURITY.md](SECURITY.md) |
| Secrets | gitleaks pre-commit hook and CI |
| Advisories in dependencies | `cargo audit` in CI |
| Coverage, advisories, outdated dependencies | `just evidence` and `vibecheck` |
| Commit format | Conventional Commits, checked by review |

`ci.yml` runs `just lint`, `just test`, `just doc` and `cargo audit` on pull requests and pushes to `main`. It runs on the GitHub mirror, which cannot commit back to the Forgejo home, so the maintainer runs `just evidence` before each release: it writes `audit/evidence/` and `vibecheck` writes `audit/status.json` (the README badge).
