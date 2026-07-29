# Ectropy v0.4.0

## Every finding is an error

Ectropy now has one refusal model: exit zero means exactly `clean`, and every
reported finding exits one. Word, dispatch, receiver, parameter, burr, shadow,
and coverage findings are no longer tolerated as debt or blindspots.

The former `--strict` spelling remains as a hidden inert compatibility option
for the v0.4 line and will be removed in v0.5.0. The former `--debt` option is
invalid.

## A versioned operating skill

Each release now carries an `ectropy-skill.tar.gz` artifact. The new
`ectropy skill` commands install, inspect, upgrade, list, and uninstall the
managed Ectropy operating brief. Stable and beta release smokes exercise the
skill lifecycle on Linux, macOS, and Windows.

## Honest native Windows scanning

Repository-relative paths are normalized to the slash dialect used by
`ectropy.toml` before scan, module, boundary, grant, and ban globs are applied.
Configured include sets no longer match zero files silently on Windows.

The Rust adapter also recognizes zero-hash raw and byte-raw strings, including
Windows path literals ending in a backslash. A native Windows guard now runs
the workspace tests and a PowerShell manager smoke for every change.

## One installed version by default

The Unix and PowerShell managers remove older installed versions after the new
binary is linked and answers `--version`. Pass `--retain` to keep existing
versions. Released artifacts remain immutable and can be fetched again with an
explicit version.

Stable releases now require English and Chinese index and migration notes
before publication begins.
