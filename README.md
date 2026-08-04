# ectropy

A self-contained structural checker for reducing semantic entropy in codebases.
Ectropy names the organizing tendency that pushes a system away from disorder.

ectropy owns a grammar-first parser substrate and delivers a uniform structure
tree to language-agnostic checks. It does not compile target languages or depend
on their toolchains for semantic validation.

## Shape

- `crates/grammar` — parser substrate; `g4` grammars embedded as compile-time
  resources; produces a concrete tree.
- `crates/kernel` — lifts the concrete tree into the structure tree, runs the
  constitutional checks, hosts thin per-language adapters.
- `crates/cli` — the `ectropy` binary.

## Laws

See `docs/principles.md`. In short: single word, block depth <= 4, path depth
<= 4 from caller-declared module roots, comments denied by default, boundary
laws are declared in `ectropy.toml`, and the vocabulary never freezes.
Selected compound terms and their descriptions live in that same file.

## Cookbook

`ectropy cookbook` lists the embedded procedural entries with the condition
that retires each one. `ectropy cookbook <entry>` prints its trigger, move,
evidence, and EXIT for agents that have only the installed binary.

## Operating

The complete guard is the direct command sequence in
`.forgejo/workflows/guard.yml`. It formats, lints, checks, tests, runs Plumb
Doctor, and checks both the repository and the Ectropy skill with the workspace
binary. Land a passing topic branch with `plumb land`.

## Install

Unix:

```sh
curl -fsSL https://releases.ectropy.perish.uk/manage.sh | sh
```

That canonical command installs the stable consensus into the default user
paths. Every non-stable release exists only as an exact seal. Resolve its fixed
manager from that seal and give it an isolated seat:

```sh
seal=https://releases.ectropy.perish.uk/v1/releases/beta/v0.5.0-beta.1/seal.json
manager=$(curl -fsSL "$seal" | jq -er '.managers.unix.url')
curl -fsSL "$manager" | sh -s -- install \
  --install-root "$RUNNER_TEMP/ectropy-beta/install" \
  --bin-dir "$RUNNER_TEMP/ectropy-beta/bin"
```

The exact manager carries the release authority, channel, and version.
