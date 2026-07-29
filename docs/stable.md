# Stable

v0.1.0 establishes the clean ectropy contract. Stable is a promise: the
surfaces below hold, and the guard's version policy prices every change — an unchanged kernel keeps
to patch bumps, a changed kernel pays a minor or higher. What is not listed
here is not promised.

## Laws

Fifteen laws judge a codebase. Every finding is an error and fails the run;
anything that cannot justify refusal does not emit.

- `word` — a declared name is one vocabulary atom; an unregistered compound is
  an error.
- `path` — directory nesting past four levels below the owning module root is an
  error.
- `file` — a file running past three hundred lines is an error; the fix is a
  move, not a squeeze.
- `fanout` — a directory holding more than ten children is an error.
- `block` — scope nesting past four is an error.
- `markup` — element nesting past eight in one element tree is an error; markup
  is its own axis, neither scope nor literal.
- `comment` — comments are denied by default; each is an error.
- `grant` — granted syntax outside its declared territory is an error; test and
  style are the open classes (undeclared means unrestricted), environment is
  sealed (undeclared means denied everywhere) — its refusal routes to the
  config cascade.
- `ban` — banned syntax inside a declared path is an error. Matching bans are
  cumulative, override matching grants, and cannot be boundary-exempted.
- `dispatch` — a repeated equality subject across adjacent branches is a table
  refusing to exist.
- `receiver` — a fourth free function in one file on one receiver names an
  object that does not exist yet. The fourth and every later finding
  names the total group and every member function in source order.
- `param` — a function holding more than four parameters, receiver included, is
  a struct refusing a name.
- `burr` — a receiver or parameter whose bound name begins with an underscore
  accepts work it does not use and needs local responsibility judgment. The
  scan remains blocked until the contract is corrected or a justified boundary
  records why it must remain. The lone discard `_` and unmarked patterns stay
  outside the law.
- `shadow` — at least four fifths of a literal copied bare from one root, or
  three fields copied whole, signals twin structures that should derive or
  compose. A real layer border must be declared as a justified boundary.
- `coverage` — an unparsed region is an error because the checker cannot claim
  a clean result for syntax it did not understand.

## Dialects

An ecosystem-forced shape is the adapter's to rectify, never the kernel's: the
web dialect reads tsx and scss natively, React's use- prefix is translated as
a namespace rather than exempted, and every carve-out is recorded in
`docs/adapters.md`.

## Structure

The structure tree is the contract between the parser substrate and every
check (`structure.md`). A node is `kind`, `depth`, `span`, `kids`; the kind
vocabulary is small and shared across languages, and adding a kind is a
reviewed contract change. `ectropy shape` emits one valid
`{"path", "tree"}` JSON line per file in sorted path order.

## Configuration

`ectropy.toml`: `[scan]` include/exclude globs, `[module]` roots, `[limit]`
block/path/param/markup/file/fanout, `[comment]` allow, `[word]` single,
`[[boundary]]` paths/allow/note, `[[grant]]` syntax/paths, `[[ban]]`
syntax/paths, and
`[[vocabulary.term]]` name/description. Both vocabulary fields are required and
non-empty; duplicate names are rejected.

Every path list reads one glob dialect: `*` matches within a segment, `**`
matches across segments, and a bare directory names that directory alone — a
subtree is spelled `dir/**`. `[scan]`, `[[boundary]]`, `[[grant]]`, and
`[[ban]]` match a whole path against it; `[module]` roots use it to find where
a root ends.

## Surface

`ectropy [root]` scans one tree, defaulting to `.`. `ectropy shape [root]`
prints structure JSON and `ectropy vocabulary [root]` prints the living
dictionary per module root. Exit 0 means exactly `clean`; every finding exits
1. Invalid roots, malformed or unreadable configuration, source IO failures,
and invalid schema references exit 2 and never print `clean`.

The v0.4 line accepts the former `--strict` spelling as a hidden inert
compatibility option. It changes no result and retires in v0.5.0. The former
`--debt` option is invalid.

`ectropy skill` installs, inspects, upgrades, lists, and removes the versioned
operating brief shipped with each release.

`ectropy cookbook` prints the embedded entry ledger with every EXIT clause;
`ectropy cookbook <entry>` prints one trigger, move, evidence, and EXIT.
An unknown entry exits 2 and lists the available names.

## Not promised

Grammar coverage breadth: Rust, TypeScript (tsx included), CSS, SCSS, and
Markdown scan today. Stylesheets are whole-file style markers and Markdown exposes heading
scopes rather than full syntax. New grammars widen coverage without ceremony. The `fanout`
count sees only the scan set — assets and empty directories are invisible, so
the width it reports only ever understates. The beta channel promises nothing —
it exists to break first.
