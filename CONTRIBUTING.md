# Contributing

Contributions are licensed under AGPL-3.0-only; inbound is outbound.

AI-assisted contributions are accepted. Disclose the tool and model with an `Assisted-by: <tool>:<model>` trailer; do not use `Co-Authored-By` for a model. The author reads and answers for every line.

Use Conventional Commits. A pull request must show `just check` passing and name the tests it adds. Deleting, skipping, or weakening a test requires its reason in the commit message. A new dependency requires a stated reason.

Run `just setup` to install the pre-commit secrets hook.
