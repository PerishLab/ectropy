# Migrating to Ectropy v0.4.0

## Resolve every reported finding

Repositories that previously relied on debt-only success must now resolve each
finding before Ectropy exits zero. Repair the named structure, or declare a
narrow justified boundary only where that law genuinely does not apply.

Remove `--debt` from scripts. `--strict` may remain temporarily but changes no
result in v0.4 and will be removed in v0.5.0.

Windows users should rerun Ectropy after upgrading. A configured include set
that previously matched no files because of Windows path separators now scans
the intended tree and may reveal real findings.

## Choose whether to retain installed versions

Manager installs now leave one version under the install root. Pass `--retain`
when old version directories must remain available offline. Otherwise, rollback
uses `install --version <older>` and downloads the immutable artifact again.

No `ectropy.toml` schema or stored data migration is required. Installing the
managed Ectropy skill is optional.
