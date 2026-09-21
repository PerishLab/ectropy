# Agents

This repository is guarded by its own laws. An agent maintaining it holds to the
same constitution ectropy enforces on others.

## Laws in practice

- KISS: refuse anonymous models. Each concrete law names one mechanically
  visible projection; `kiss` is never a finding or suppression label.
- Single word: each name is one vocabulary atom. Maintain vocabulary deltas
  alongside code diffs (`ectropy.toml`).
- Block depth <= 4. Path depth <= 4 from a declared module root. Flatten before
  nesting deeper.
- No comments by default. Move explanation into a name, a test, the vocabulary,
  or an admitted document. Boundary exemptions live in `ectropy.toml`.

## Architecture

Three crates, no plugin crate: `grammar` (substrate + embedded g4), `kernel`
(structure tree + checks + thin adapters), `cli`. Dependency direction is
`cli -> kernel -> grammar`. [DESIGN.md](DESIGN.md) owns the structure-tree and
adapter contracts; do not reach around them. Add a language as grammar and
namespace data inside the existing crates, never as a plugin crate.

## Operating

- Never commit on `main`. Branch before committing.
- Guard hooks come from `plumb configuration install`; a commit carries the
  staged-tree Guard proof, and `plumb land` is the landing authority.

## Release

- Plumb owns repository governance, release markers, and landing; wharf
  builds, binds, and distributes each release. Read their current help and
  rules; do not restate a release workflow or changelog shape here.
- `plumb.toml` and `ectropy.toml` are this repository's own declarations,
  layered over the base Plumb carries in its binary. Where they depart from
  that base, `plumb doctor` says so as a noted finding.
- The repository carries no workflow and no release notes. `skills/ectropy` is
  the skill source; release notes and skill generations live on Depot.
