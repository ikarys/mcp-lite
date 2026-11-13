# AGENTS

mcp-lite is a small synchronous Model Context Protocol (MCP) client for Rust.
It provides a lazy tool layer that keeps model prompts stable.

## Layout

- `src/server.rs`: server configuration types.
- `src/lazy.rs`: lazy tool layer without I/O.
- `src/client.rs` and `src/error.rs`: behind the `client` feature.
- `src/oauth/`: behind the `oauth` feature.

## Rules

- The core never does I/O.
- No async runtime.
- No new dependency without a stated reason.
- The serde formats of `McpServer`, `McpTool`, and `McpCredentials`, and the output of `meta_tool_definitions`, `catalog`, and `tool_schemas`, are stable; consumers store them and cache prompts on them. Changing them is a breaking change.

## Vibecoding Convention (level 2)

This project follows the Vibecoding Convention at level 2; see `VIBECODING.md`.

Before calling a change done, these must pass:
- `just test`
- `just lint`
- `just secrets`

Rules while editing:
- Every commit with AI-written content ends with the attribution trailer this
  project uses, naming the tool and model, for example `Assisted-by: <Tool>:<model>`.
- Deleting, skipping or weakening a test requires the reason in the commit message.
- A new dependency is checked against the registry (exists under that exact name,
  established, the one the code needs) and taken at the registry's current
  version, not the one you remember; the check is noted in the commit. Existing
  dependencies are not held back without a reason written in `VIBECODING.md`.
- No secret in any file, prompt or config.
- Every input is hostile: arguments, files, environment, network, model output.
  Follow the project's security checklist in `SECURITY.md`; a change that touches
  parsing, SQL or command building, authentication, crypto, credentials or file
  import names the vulnerability class it guards against and its test.
- Signal over noise. A comment states a reason, a constraint or a trap, never
  what the line does. A docstring tells the caller what the signature does
  not. The public API is documented. No TODO, placeholder, commented-out or
  dead code in a delivered change. Documentation proportionate to the
  project: a README says what it is, how to install, how to use, where to go
  next; nothing written to fill space.

Never without a human: merge, release, change the CI, delete data.
