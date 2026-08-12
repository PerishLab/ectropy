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
- The complete direct guard in `.forgejo/workflows/guard.yml` must pass before
  landing.
- `plumb land` is the landing authority.

## Release

- Stable Plumb owns the complete binary release mechanism. This repository
  declares the product and skill in `plumb.toml`; its exact/stable workflows
  are thin Actions callers.
- Every release produces immutable content-addressed objects and one exact seal
  at `v1/releases/<channel>/<version>/seal.json`.
- Non-stable releases stop at their exact seal and install only through its
  generated manager into explicit isolated paths.
- Stable promotion proves an exact non-stable seal from the same commit. Only
  stable activation updates `v1/channels/stable.json` and the canonical root
  managers.
- Release jobs bind build, publication, smoke, and stable tagging to one
  resolved commit. Publication and activation use separate credentials.
- Generated managers and capsules are release outputs, not repository files.
- Managed skill install, status, and upgrade are stable-only. Exact non-stable
  briefs use `skill stage` at a new explicit path and never enter the ledger.
- A stable release requires
  `docs/CHANGELOG/v<version>/{en,zh}/{INDEX.md,MIGRATION.md}`, enforced by the
  stable capsule compiler before anything irreversible.
  `plumb doctor` does not check this: a changelog is owed by a release, not by a
  working tree. A release requiring nothing of anyone still writes MIGRATION.md
  saying so. Follow the Plumb skill and its release-local CHANGELOG contract.
