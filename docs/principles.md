# Principles

ectropy reduces semantic entropy in a codebase by pushing explanation pressure
out of prose and into structure, tests, vocabulary, and docs. These are the laws
the checker enforces on itself and on every codebase it guards.

## Terrain

Agents learn first from the executable terrain they enter. A shape in the tree
outweighs prose around it, while a mechanized law travels with every edit. An
honestly detectable signal belongs under law. A procedure awaiting automation
belongs in the cookbook only while it names the condition that deletes it;
judgment with no writable exit belongs permanently in principles.

## KISS

KISS refuses anonymous models. A short expression is not simple when it carries
an unnamed combination space, table, object, parameter structure, or
translation. Each law names one mechanically detectable projection of that
absence and pushes the model into structure. KISS is the principle above those
laws, never a finding of its own.

## Single word

A name should be one vocabulary atom in the current language namespace, not one
English word. Single-word pressure points downward to the language namespace
mechanisms that let a short name resolve, and upward to a living business
vocabulary that gives the atom meaning. Path segments below the owning module
root are names too: a compound directory or file stem is a word error, so
flattening directories by fusing their names into a file cannot silently evade
either law.

## Block depth <= 4

Nesting deeper than four blocks is a signal that a unit is carrying more than one
idea. Extract until the shape is flat.

## Markup depth <= 8

Markup nesting past the limit is an atomic component refusing to exist: extract
and name the wrapper instead of indenting further. Markup counts its own axis;
code scopes inside markup stay on the block law's axis.

## Module roots

A caller declares the module roots that own its scanned files. A module root is
language-independent and package-manager-independent: it establishes the depth
coordinate system, but does not change scan inclusion or exempt any law. Without
a declaration, the repository root is the implicit module root. With declared
roots, every scanned file belongs to the most specific matching root.

## Path depth <= 4

Directory nesting deeper than four levels below the owning module root is the
same signal at the file-tree scale. A path is a name; keep it a short one.

## Comments denied by default

A comment is explanation that failed to become structure. Deny comments by default
so the pressure moves into a better name, a test, a vocabulary entry, or a doc.

## Dispatch

The second equality against one subject makes sibling branches a table, and a
table is one match, not a ladder of ifs. A decision is a thing; give it one
node. Guard chains over different subjects, comparisons between variables,
and branches separated by other work are not tables and stay untouched.

## Combination

A Boolean expression combines at most three decision atoms. The fourth opens an
anonymous combination model: temporal facts become an explicit state and
transition, while orthogonal facts become a classification, pattern, decision
table, or domain type. Extracting the same expression behind a predicate name
does not change its model and remains a finding at the new site.

## Receiver

The first parameter is the receiver. A fourth free function in one file
taking the same receiver — same name, same type — names an object that does
not exist yet: build it once and let methods read what they share. Methods
are exempt; they have already declared their subject. The finding names the
whole group in source order because the evidence is the group, not only the
member where the threshold crossed.

## Parameters

A function takes at most four parameters, counting methods past their self.
The fifth is a struct refusing to exist: name the bundle and pass it whole.

## Burr

A named underscore parameter says a function accepts work its body does not
use. That signal needs local judgment: either the callee should consume the
work or the caller declared too much contract. Refuse the scan and follow the
burr cookbook; never prescribe one cure mechanically. A case that must remain
is a justified boundary, not a tolerated finding. The lone `_` is an honest
discard, and pattern parameters remain outside the shared tree.

## Shadow

A literal that copies at least four fifths of its fields bare from one root is
a bridge between twin structures. Three fields copied whole carry the same
signal; smaller pairs do not. Derive or compose the destination instead of
maintaining both declarations by hand. An honest layer border may keep the
bridge only through an explicit justified boundary.

## Migration

A migration answers to the target paradigm, not to a one-to-one map of source
shapes. Mechanical translation preserves old accidents under new syntax. Build
the verification net first, then redesign responsibilities into the target's
native structure. Success means source accidents disappear, not that every old
line gains a counterpart.

## Boundary laws

Vendored infra and other mechanically identifiable territory may exempt named
laws through `[[boundary]]`. An exemption is declared, path-scoped, and
justified in `ectropy.toml`; it never changes which files are scanned.

Not every law admits one. `coverage` refuses the route because a clean result
cannot cover terrain nobody parsed, and `grant` and `ban` refuse it because
their own declarations already carry the territory. The catalog holds which
laws are sealed, and configuration refuses a sealed name rather than accepting
an exemption that would never fire. Read it with `ectropy law`.

## Syntax grants

The dual of a boundary: a `[[grant]]` reserves a syntax class to declared
territory, and that syntax appearing anywhere else is an error. Test syntax is
the first class: a grant is widened, never exempted, so with one declared
product files hold zero test code and
`ls` is the audit. Territory rides each toolchain's own test-discovery
contract — cargo's `tests/` directory, Deno's `*.test.ts` suffix — one truth
source per language. No grant declared for a class means that class is
unrestricted.

Environment is the third class and the first sealed one: the surface a
process inherits without declaring it — `std::env` in rust, `Deno.env` and
`process.env` in typescript — read directly, bypasses every config layer, so
the class is denied everywhere until a `[[grant]]` names its territory
(tests, typically). The refusal is a ramp, not a wall: it routes to the
config cascade, where the same value arrives named, typed, and overridable
(plumb `docs/config.md`). The rust claim is path-text: `std::env` spelled
directly or inside a braced `use std::{..}` group; a rebound alias is a
coverage gap to close, not author license.

## Syntax bans

A `[[ban]]` denies one syntax class in declared paths. Bans are cumulative,
independent of grants, and win when both match: permission elsewhere cannot
weaken a local prohibition. They are not boundary laws and cannot be exempted.
The configuration names only shared syntax classes that the structure tree can
prove without package identity or symbol resolution. Ecosystem dependencies
remain repository-shape policy outside ectropy.

## The vocabulary never freezes

Single-word only works against a living vocabulary. Agents maintain vocabulary
deltas alongside code diffs. Selected compound meanings are described by
`[[vocabulary.term]]` entries in `ectropy.toml`; source mining remains the live
inventory and the configuration is not an exhaustive dictionary.
