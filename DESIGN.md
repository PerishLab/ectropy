# Design

Ectropy is an errors-only structural checker. It owns a grammar-first parser
substrate and presents one language-independent structure tree to laws. It does
not compile target languages, depend on their toolchains for semantic
validation, or claim repository and release policy that belongs to Plumb.

## Constitution

Executable terrain outweighs prose around it. A mechanically detectable signal
belongs under a named law, and a clean result is valid only when all admitted
terrain was understood. Every finding is therefore an error. Judgment that
cannot honestly justify a refusal remains outside the checker.

KISS refuses anonymous models. A short expression is not simple while it hides
a combination space, table, object, parameter structure, state transition, or
translation. KISS stands above the individual laws and is never itself a
finding or suppression label.

A word is one vocabulary atom in its language namespace, not necessarily one
English word. Single-word pressure points downward to namespace rules and
upward to the product vocabulary. Paths below a module root are names on the
same terms, so fusing a compound directory into a compound file cannot evade
the law. The vocabulary stays live: configuration records selected product
terms but is not an exhaustive dictionary.

Deep blocks and deep paths are the same signal at different scales: one unit is
carrying more than one idea. Markup has its own depth axis because wrapper
composition is independent of code scope. Excessive directory width signals an
unnamed grouping. Oversized files are moved, not squeezed.

Comments are explanation that failed to become structure. They are denied by
default so pressure moves into names, types, tests, vocabulary, or an admitted
document. Migration follows the target paradigm rather than preserving source
accidents through a one-to-one translation: establish verification first, then
redesign responsibilities into the target's native shape.

Boundaries exempt a named law only on mechanically identifiable territory and
must carry a truthful reason. They never change the scan set. Coverage cannot
be exempted because unread syntax invalidates a clean result. Grants and bans
also admit no boundary: their own paths already define their territory. Grants
reserve a syntax class to named terrain; bans deny a class within named terrain
and win when both match.

## Product shape

The repository has three crates with one dependency direction:
`cli -> kernel -> grammar`.

- `grammar` owns the parser substrate and embedded g4 resources. It produces a
  concrete tree without an ANTLR, JVM, or target-toolchain dependency.
- `kernel` lifts concrete trees into the shared structure tree, hosts thin
  adapters, and runs language-independent laws.
- `cli` owns command parsing, deterministic reporting, skill operations, and
  the embedded cookbook delivered by the binary.

There is no plugin crate and no adapter ABI. A new language contributes data
and a thin map inside the existing crates; it does not create a new dependency
direction or dynamically loaded surface.

## Structure tree

The structure tree is the contract between parser substrate, adapters, and
laws. Every node carries a shared `kind`, its law-specific `depth`, a byte span,
and child nodes. Laws never reach into lexer or parser internals. Adding a kind
is a reviewed contract change because every adapter and law shares the same
vocabulary.

The core kinds express roles, not target syntax:

- `root`, `scope`, `item`, and `literal` distinguish the source, real nesting,
  declarations, and value forms. Literal braces do not inflate scope depth.
- `comment`, `word`, and `test` expose explanation, names, and test markers to
  their respective laws.
- `probe`, `decision`, and `atom` represent adjacent literal dispatch and one
  connected Boolean combination without exporting a language expression tree.
- `receiver` and `param` expose function responsibility. Shared receivers
  reveal an undeclared object; excessive parameters reveal an undeclared
  bundle; a named underscore binding reveals accepted but unused work.
- `markup` carries element depth independently of code scopes.
- `style` and `environment` expose syntax whose lawful territory is assigned by
  grants or bans.

Module roots provide the coordinate system for path depth. They neither select
files nor change law. Each scanned file belongs to the most specific matching
root; without a declaration the repository root is implicit.

The tree deliberately contains only evidence the checker can prove without
symbol resolution. A law whose evidence needs package identity or type meaning
does not belong in the shared kernel.

## Adapter contract

A language supplies an embedded structural grammar, a namespace-atom
specification, and maps from native constructs into shared kinds. It may also
rectify syntax whose surface spelling would otherwise lie to a shared law. The
constitutional checks run once over the resulting tree and are not
reimplemented per language.

Boolean conjunction and disjunction become connected `decision` trees;
parentheses and negation are transparent while calls, assignments, branches,
and statement boundaries start independent decisions. Rust closure spelling is
rectified before it can look like disjunction. TypeScript regular expressions
are opaque only where lexical expression-start rules prove them to be literals.

TSX extends the TypeScript grammar without teaching kernel about React. Paired
elements, self-closing elements, and fragments become `markup`; brace
containers re-enter normal parsing so code scope continues independently. The
ecosystem `use` prefix is treated as a namespace dialect, while PascalCase
components receive no compound-name exemption.

Svelte extends the same TypeScript grammar without teaching kernel about
Svelte. Script declarations, legacy reactive statements, runes, template
expressions, control blocks, snippets, special tags, components, and HTML
comments enter the shared tree through their structural roles. Control blocks
become `scope`, elements become `markup`, and component filenames remain path
atoms subject to the same single-word law as every other source file.

Style attributes, style elements, Svelte style blocks, stylesheet imports, and
whole CSS or SCSS files become `style` evidence. CSS and SCSS internals are
intentionally not parsed today. Markdown currently exposes heading scopes
only. These are honest coverage limits, not promises that malformed syntax is
acceptable.

Per-language calibration may decide how native constructs map into the common
constitution, but it cannot contradict that constitution. Framework identity,
ecosystem dependency rules, and repository shape remain outside adapters.

## Law and configuration surface

The law catalog is the source of truth for the current named refusals and their
boundary eligibility. `ectropy law` exposes it directly; documents do not copy
an inventory that can drift. Configuration selects scan globs, module roots,
limits, comment and word policy, syntax grants and bans, justified boundaries,
and vocabulary terms. Unknown fields, unknown laws, malformed globs, and empty
reasons are operational errors.

All path lists use one dialect: `*` stays within a segment, `**` crosses
segments, and a bare directory names only that directory. Scan, boundary,
grant, and ban patterns match whole paths; module patterns locate roots.

Exit zero means exactly `clean`, exit one carries findings, and exit two means
the operation itself was invalid. Invalid configuration, unreadable source, or
unparsed admitted syntax can never print `clean`. Output ordering and digests
are deterministic so the same terrain produces the same proof.

The CLI exposes scans, structure and vocabulary projections, the law catalog,
the bounded cookbook, and stable skill management. Cookbook entries are
temporary procedural product assets: each names its trigger, a judgment-guided
move, evidence, and the condition that deletes the entry.

## Stability boundary

Stable covers the law catalog, configuration schema, shared tree contract, CLI
semantics, deterministic output, and error model. A changed kernel contract pays
a minor-or-higher version; a patch may preserve it. Grammar breadth is not a
stable promise and may widen as coverage becomes honest. The beta channel is a
place to break before stable and carries no compatibility promise.

Current known understatements remain explicit: stylesheet internals and most
Markdown syntax are opaque, the fanout law sees only scanned files, adapters do
not perform symbol resolution, and malformed TSX can sometimes distort a local
parse before coverage refuses the unread region. These are coverage work, not
authorization to claim more than the tree proves.
