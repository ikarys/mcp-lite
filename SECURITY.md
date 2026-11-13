# Security

## Reporting a vulnerability

Write a private vulnerability report on GitHub (https://github.com/ikarys/mcp-lite/security/advisories/new) or to bikarys@proton.me. We acknowledge within 7 days. Do not open a public issue for a vulnerability.

## What this software protects, and what it does not

mcp-lite treats MCP servers and their output as untrusted. It does not sandbox stdio servers; the caller passes a `spawn` closure that should sandbox them. It does not store or encrypt credentials; credentials are returned in clear and the caller seals them. It does not check the OAuth `state`; the caller keys the pending flow by `state`. Tool output is data for a model, not instructions.

## Threat model: known vulnerability classes and their coverage

The classes relevant to this kind of software, from the OWASP Top 10, the CWE Top 25 or the ecosystem's list, each with what covers it. A class with no coverage is a known gap and says so.

| Class | Where it could happen here | Covered by (tool rule or test) |
|---|---|---|
| Hostile server loops or floods tool listing | `src/client.rs` | `src/client.rs::a_repeated_tools_list_cursor_errors` |
| Hanging server | `src/client.rs` | `src/client.rs::a_hanging_stdio_server_times_out` |
| Binary or huge content in model input | `src/client.rs` | `src/client.rs::tool_text_renders_non_text_content_parts` |
| Secrets in logs (`Debug`) | `src/oauth/protocol.rs`, `src/oauth/flow.rs` | `src/oauth/protocol.rs::mcp_credentials_debug_redacts_secret_fields`, `src/oauth/flow.rs::pending_debug_hides_the_verifier` |
| OAuth code interception | `src/oauth/protocol.rs` | PKCE S256; `src/oauth/protocol.rs::pkce_matches_rfc_7636_appendix_b` |
| Token sent to wrong header/override | `src/client.rs` | `src/client.rs::a_token_overrides_a_manual_authorization_header_in_any_case`, `src/client.rs::an_empty_token_sends_no_authorization_header` |
| Vulnerable dependencies | `Cargo.toml` | `cargo audit` in CI |
| Secrets in repository | repository | gitleaks pre-commit hook and CI |
| Command injection | `src/client.rs` stdio spawn | stdio args are passed as argv, no shell; untested beyond `src/client.rs::a_stdio_client_lists_and_calls_tools` |

## Claims and their tests

Each claim this project makes about its security points to the test that proves it. A claim without a test says so.

| Claim | Test |
|---|---|
| Tool listing stops when the server repeats a cursor | `src/client.rs::a_repeated_tools_list_cursor_errors` |
| A hanging stdio server times out | `src/client.rs::a_hanging_stdio_server_times_out` |
| Non-text content parts are rendered as text in tool output | `src/client.rs::tool_text_renders_non_text_content_parts` |
| `McpCredentials` `Debug` redacts secret fields | `src/oauth/protocol.rs::mcp_credentials_debug_redacts_secret_fields` |
| `Pending` `Debug` hides the PKCE verifier | `src/oauth/flow.rs::pending_debug_hides_the_verifier` |
| PKCE verifier and challenge match RFC 7636 Appendix B | `src/oauth/protocol.rs::pkce_matches_rfc_7636_appendix_b` |
| A token overrides a manual Authorization header in any case | `src/client.rs::a_token_overrides_a_manual_authorization_header_in_any_case` |
| An empty token sends no Authorization header | `src/client.rs::an_empty_token_sends_no_authorization_header` |
