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

`environment` and `embed` are sealed: they refuse everywhere a grant does not
name. Grants name production resources; tests never hold one, so the rule
needs no test identity. A Rust test marker still outranks a grant: `embed`
inside an item carrying `#[test]` or `#[cfg(test)]` refuses in a granted file.

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
- `tuple` and `position` expose an anonymous positional product and its ordered
  fields without assigning semantic meaning to those positions.
- `record`, `label`, and `field` expose a named product declaration, its identity,
  and its named members. An exact field-name set is its schema; types and
  declaration names are not.
- `markup` carries element depth independently of code scopes.
- `style`, `environment`, and `embed` expose syntax whose lawful territory is
  assigned by grants or bans. `embed` binds a repository file's content or
  location into code.
- `link` exposes a path value that wires another source file in as code: a
  Rust `#[path]` value or a TypeScript import specifier.

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
rectified before it can look like disjunction. Rust macro input inside a function, and braced macro input at module level, is read by the statement reader; tokens that form no Rust structure carry no evidence. TypeScript regular expressions
are opaque only where lexical expression-start rules prove them to be literals.

A braceless TypeScript body is a one-statement scope, read as the same
statement inside braces is, when it belongs to an `if` whose head is
`name === literal`, to the `else` of such an `if` or of a braced branch, or to
a `for` or `while`. The statement extends through every brace-delimited form
it holds (object literal, function or arrow body, class expression, binding
pattern, type literal) to its `;`, or, where `;` is omitted, to the next `if`,
`else`, loop, `do`, `switch`, `try` or declaration keyword. A loop head's
binding is not a declaration, and an `if` that is a braceless loop body links
no ladder outside that body. Such a body adds a block level as its braced
spelling does.

TSX extends the TypeScript grammar without teaching kernel about React. Paired
elements, self-closing elements, and fragments become `markup`; brace
containers re-enter normal parsing so code scope continues independently.
Element text is text: the lexer gives markup text its own context, so quotes,
backticks, `//`, `/*` and slashes there are characters rather than string,
comment or regular-expression openers. In TSX an element starts only where an
expression can start, and a `<` that opens type parameters (`<T,>`,
`<T extends …>`, `<T>(…)`) stays code; in Svelte everything outside script,
style and `{ … }` is markup text. Template literal `${ … }` interpolations
re-enter expression reading the same way brace containers do; template text
stays literal, and an interpolated import specifier is still not `link`
evidence. The
ecosystem `use` prefix is treated as a namespace dialect, while PascalCase
components receive no compound-name exemption.

Svelte extends the same TypeScript grammar without teaching kernel about
Svelte. Script declarations, legacy reactive statements, runes, template
expressions, control blocks, snippets, special tags, components, and HTML
comments enter the shared tree through their structural roles. Template
expressions and attribute values re-enter TypeScript expression reading the way
TSX brace containers do, so arrows and function values there are read; a quoted
single-expression attribute value (`on:click="{…}"`) is the same expression,
while a quoted value that mixes text with expressions stays one string literal.
A legacy reactive block `$: { … }` is a scope, and snippet parameters count as
a function's do. Control blocks become `scope`; an `{#if}` block reads like a
script `if`: each branch is its own `scope`, and an `{#if}` or `{:else if}`
head of the form `name === literal` emits `probe`, so an equality ladder
reaches `dispatch`. Elements become `markup`, and component filenames remain
path atoms subject to the same single-word law as every other source file. An
attribute name carries directive prefixes and custom property names alike, so a
directive that binds a custom property is read rather than left unparsed.

JavaScript enters through the same grammars. `.mts` and `.cts` are TypeScript;
`.js`, `.mjs`, `.cjs` and `.jsx` are read through the JSX grammar, as the
TypeScript compiler reads every JavaScript file kind. CommonJS `require("…")`
is a call, not `link` evidence. Recognized extensions and the grammar that
reads each are one table, so a recognized file always has a reader, and one
outside every scan include is a `coverage` finding.

Python enters the same grammar substrate through virtual newline, indent, and
dedent evidence carrying original byte spans. Its adapter maps suites,
declarations, parameters, docstrings, dictionaries, Boolean words, equality
probes, and the explicit `os.environ` surface into shared kinds. An `if` or
`elif` head of the form `name == literal`, bare or parenthesized, emits
`probe`, and `elif` links a ladder as `else if` does, so an equality ladder
reaches `dispatch` as in Rust and TypeScript. It does not infer test identity
from frameworks or names, invoke a Python runtime, or treat `.pyi` as source.

Rust and Python tuple expressions, patterns, and types map into shared `tuple`
and `position` evidence. TypeScript maps tuple types and destructuring, while
ordinary arrays and function argument lists remain distinct syntax. The tuple
law measures only visible arity and does not infer semantic exceptions.

Rust named-field structs and enum struct variants, Python classes with
annotated fields, and TypeScript type literals and interfaces map into shared
`record`, `label`, and `field` evidence. A struct variant is read the way a
named-field struct is: its field names are words, and its record is labeled by
the variant. TypeScript interface and type-literal member names are words, as
Rust field names are: a property, a method signature, an optional or `readonly`
member, separated by `;`, `,` or a newline. Type-literal and interface
signatures are not read as methods: a bodyless signature has no body scope, and
its parameters do not reach `param`/`receiver` (#84, function types stay
unread). Its name is `word` evidence, as a bodyless Rust trait fn's and a
TypeScript `abstract` method's already are.
Tuple structs, literals, index signatures, mapped types, and unannotated Python
assignments do not. A TypeScript object-literal key is a word only when its
value is a function literal (`k: () => …`, `k: function () { … }`), read as a
method key `k() {}` is (#73, #107). Any other value (`k: 1`, `k: ref`) makes
the key a use site, which stays unread as intended scope, the same as a Rust
struct-literal field `S { k: 1 }`. A reference-valued function key
(`followEnd: controller.followEnd`) is accepted as unread while its arrow
spelling is a word: telling the two apart needs symbol resolution (#108).
The schema law groups exact unordered field-name sets across
the admitted repository scan closure after path boundaries are applied. This is
the one repository-wide structural law; adapters still emit only per-file
evidence and never inspect another source.

Rust `include_str!`, `include_bytes!`, `include!`, and the
`CARGO_MANIFEST_DIR` token; TypeScript `import.meta.url`, `dirname`,
`filename`, and `glob` with `__dirname` and `__filename`; and Python `__file__`
become `embed` evidence. Rust `#[path]` is module wiring, a file compiled as
code rather than content or location bound as data, so it is not `embed`.
Framework conventions such as Vite `?raw` imports are not language dialect and
stay out. Runtime reads by relative path cannot be told apart from temporary
fixtures without symbol resolution.

Rust `#[path = "…"]` values and TypeScript, TSX, and Svelte string specifiers
after `from`, in a bare `import "…"`, and in a dynamic `import("…")` become
`link` evidence. The `reach` law refuses a link whose path has a `..` segment
followed later by a `src` segment: a relative path never climbs into a `src`
tree, and the recovery names the package through its `exports`. It holds
everywhere, admits no boundary, needs no grant or syntax class, and reads
neither package manifests nor test identity. Framework calls such as
`vi.mock("…")` carry framework identity rather than language syntax and stay
out, as do template specifiers that interpolate, since their path is not
visible.

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
the bounded Cookbook, and stable skill management. Cookbook entries use the
shared Plumb code-addressed model. Each exact code names its trigger, a
judgment-guided solution, evidence, and the condition that deletes the entry;
Human and JSON views project the same deterministically ordered content.

## Stability boundary

Stable covers the law catalog, configuration schema, shared tree contract, CLI
semantics, deterministic output, and error model. A changed kernel contract pays
a minor-or-higher version; a patch may preserve it. Grammar breadth is not a
stable promise and may widen as coverage becomes honest. The beta channel is a
place to break before stable and carries no compatibility promise.

Current known understatements remain explicit. They are coverage work, not
authorization to claim more than the tree proves:

- Stylesheet internals and most Markdown syntax are opaque, and the fanout law
  sees only scanned files.
- `reach` does not read `vi.mock("…")` or CommonJS `require("…")` arguments
  or interpolated template specifiers, and adapters do not perform symbol
  resolution.
- Malformed TSX can sometimes distort a local parse before coverage refuses the
  unread region. Valid TSX can too: element text that opens with `(` right
  after a bare tag name (`<b>(it's)</b>`) lexes as type parameters, so quotes
  in that text still open strings.
- A labeled bare block (`name: { … }`) inside a TypeScript function body reads
  as an object literal, so its own brace adds no block level.
- `export default function () { … }` reads as a call, and a braceless `if`
  branch whose head is not `name === literal` is not a one-statement scope (at
  module level the statement reads as a call), so neither body adds a block
  level.
- A ladder links only through `else if`, `elif` or an adjacent `if`. A Rust or
  TypeScript comment between a branch's `}` and `else if` breaks the link, and
  adjacent Svelte blocks (`{/if}{#if …}`) never link, so neither reaches
  `dispatch`. A parenthesized Rust head (`if (a == 1)`) emits no `probe`.
- A quoted Svelte attribute value that mixes text with expressions stays one
  string literal, so its expressions are not read.
- Rust macro input in parentheses or brackets at module level reads as an
  expression, so items inside it give no name or parameter evidence.
- A type literal written as a function type's return in a type alias, a type
  member, a variable annotation or an `as` cast (`type T = () => { k: V }`)
  reads as a scope, so its member names are not `word` evidence.
