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

Operator flows run through `runseal`:

- `runseal :init` — validate the repo and install versioned git hooks.
- `runseal :guard` — format, lint, test, check wrappers, and self-check.
- `runseal :land` — land the current topic branch on Forgejo through
  `tea login add --name ectropy --url https://git.perish.top --token <token>`.

The repository guard runs the workspace binary directly, so a fresh checkout
does not require a previously published ectropy.

## Install

Unix:

```sh
curl -fsSL https://releases.ectropy.perish.uk/manage.sh | sh
```

That canonical command installs the stable consensus into the default user
paths. Every non-stable release exists only as an exact seal. Resolve its fixed
manager from that seal and give it an isolated seat:

```sh
seal=https://releases.ectropy.perish.uk/v1/releases/beta/v0.4.0-beta.1/seal.json
manager=$(curl -fsSL "$seal" | jq -er '.managers.unix.url')
curl -fsSL "$manager" | sh -s -- install \
  --install-root "$RUNNER_TEMP/ectropy-beta/install" \
  --bin-dir "$RUNNER_TEMP/ectropy-beta/bin"
```

The exact manager carries the release authority, channel, and version.
