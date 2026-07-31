---
name: ectropy
description: Operate Ectropy, the errors-only structural and vocabulary checker. Use when configuring ectropy.toml, interpreting or repairing findings, maintaining grammars or laws, running shape or vocabulary inspection, managing the Ectropy skill, or changing a repository guarded by Ectropy.
---

# Ectropy

Ectropy refuses structural entropy it can detect honestly. Every finding is an
error. Exit zero means exactly `clean`; a signal that cannot justify refusal
does not emit.

## Upstream

Repository: https://git.perish.top/PerishFire/ectropy

Report false findings, coverage gaps, and unclear law there. Use
`ectropy skill status` to inspect the current stable brief without changing an
installation. Use `ectropy skill upgrade --dry-run` before moving a managed
installation.

## Principles

**KISS.** Refuse anonymous models. A short expression or flat file is not
simple when it carries an unnamed combination space, table, object, parameter
structure, or translation. Each law names one detectable projection; KISS is
the principle above them and never a finding of its own.

**Structure carries explanation.** Prefer a smaller unit, a name, a test, a
vocabulary entry, or a document over prose embedded beside code.

**Vocabulary stays alive.** A name is one atom in its language namespace.
Register an established compound meaning; do not freeze a global dictionary.

**Refusal is singular.** Ectropy has no warning, debt, or promoted mode. A
finding blocks the scan until the code moves or a justified path boundary
records why the law does not apply there.

**Coverage is honesty.** Unparsed syntax is an error because Ectropy cannot
claim a clean result for terrain it did not understand.

**Adapters rectify dialects.** Ecosystem spelling belongs in a language
adapter. The kernel never hard-codes framework identity or package names.

## Laws

- `word` refuses an unregistered compound declared name.
- `path` refuses nesting past four levels below the owning module root.
- `file` refuses a source file longer than three hundred lines.
- `fanout` refuses a scanned directory with more than ten children.
- `block` refuses scope depth past four.
- `markup` refuses element depth past eight on its independent axis.
- `comment` refuses comments unless policy allows them.
- `grant` refuses reserved syntax outside granted paths.
- `ban` refuses banned syntax inside declared paths.
- `dispatch` refuses adjacent equality branches repeating one subject.
- `combination` refuses a Boolean expression combining more than three
  decision atoms.
- `receiver` refuses a fourth free function sharing one receiver in a file.
- `param` refuses a function with more than four parameters.
- `burr` refuses a named underscore parameter accepted but unused.
- `shadow` refuses a literal densely copied bare from one source root.
- `coverage` refuses an unparsed region.

## Workflow

1. Enter the repository and read its instructions.
2. Run `plumb doctor .` before changing repository shape.
3. Read `ectropy.toml`; do not infer policy from output alone.
4. Run `ectropy .`.
5. Repair the structure named by each law.
6. Use `ectropy cookbook <entry>` when a finding names a cookbook move.
7. Add a boundary only for mechanically identifiable territory where the law
   genuinely does not apply; keep paths narrow and write the reason.
8. Update `[[vocabulary.term]]` only for an established compound atom.
9. Re-run Ectropy and the repository's complete guard.

Never suppress a finding merely to restore exit zero. A boundary changes where
a law applies; it is not a tolerated diagnostic.

## Configuration

`ectropy.toml` owns scan globs, module roots, limits, comment and word policy,
syntax grants and bans, justified boundaries, and vocabulary terms. Unknown
fields, malformed globs, empty boundary reasons, and unknown law names refuse
configuration.

Path lists use one dialect: `*` stays within one segment and `**` crosses
segments. A bare directory names only that directory.

## Standing

Ectropy mechanizes:

- all sixteen laws above over the scanned source set;
- configuration parsing and schema validation;
- deterministic finding order, law digest, and hot-file digest;
- exit 0 for `clean`, exit 1 for findings, and exit 2 for operational errors;
- structure JSON and living vocabulary inspection.

Ectropy does not mechanize:

- whether a boundary rationale is truthful;
- whether a registered compound is real product vocabulary;
- parser breadth beyond the syntax represented by current adapters;
- repository shape, dependency policy, release law, or landing procedure.

Those remain with the repository and Plumb. Do not claim them as Ectropy
checks.

## Invocation

```bash
ectropy [ROOT]
ectropy shape [ROOT]
ectropy vocabulary [ROOT]
ectropy cookbook [ENTRY]
ectropy skill install
ectropy skill status
ectropy skill upgrade --dry-run
ectropy skill upgrade
ectropy skill stage --channel beta --version <exact> --path <isolated>/ectropy
ectropy skill list
ectropy skill uninstall
```

Managed install, status, and upgrade accept stable only. Install may take
`--path` ending in `ectropy`; replacement requires both the state record and
the in-path marker, and an unowned path refuses.

`skill stage` is the separate candidate path. It requires a non-stable channel,
an exact immutable version, and a new explicit path ending in `ectropy`. It
writes a staged marker without reading or writing the managed ledger. Stable
refuses staging, and non-stable releases refuse every managed operation.

The v0.4 line accepts hidden `--strict` only as an inert compatibility spelling.
It changes no result and retires in v0.5.0. `--debt` is invalid.
