# Migrating to Ectropy v0.6.1

This release requires nothing of callers or `ectropy.toml`.

## Managed skill

Upgrade a managed skill with the stable binary once stable is activated. To
evaluate the candidate, stage the exact beta skill into an isolated path
rather than replacing a managed seat. No stored data requires migration.

Existing Claude and Codex seats need no action beyond that upgrade.

If `~/.grok` already exists, the next `ectropy skill install` will attempt
`~/.grok/skills/ectropy`. A directory already there that is not in the
Ectropy ledger is refused. Remove that directory, then install. `--force`
cannot claim it.

Installations that do not have a Grok home remain unchanged.
