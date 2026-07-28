# Adapter contract

There is no plugin crate. A language is a thin adapter inside `kernel` plus data,
never a new crate. Adding one is a bounded, declared change.

## What a language provides

- A grammar written in a g4 (ANTLR-ish) subset, embedded in `grammar` as a
  compile-time resource. `grammar` owns a fully self-built dynamic lexer and
  parser that interpret this format; there is no ANTLR or JVM dependency. The
  grammar is structural only: it brackets scopes, distinguishes literals from
  scopes, and marks comments and words.
- A namespace-atom spec: what counts as one vocabulary atom in this language, so
  the single-word law can resolve names against the language namespace. The first
  rule splits identifiers on snake_case and camelCase boundaries; the living
  vocabulary then reconciles which multi-atom names are legitimate.

## What kernel provides

The constitutional checks (block depth, path depth, comments, single word) are
language-agnostic and run once on the uniform structure tree. Caller-declared
module roots provide the language-independent coordinate system for path depth;
they are not adapter or package-manager knowledge. The checks are not
reimplemented per language. The adapter is only the map from native constructs
to the shared kind vocabulary plus the namespace-atom spec.

## Two layers of law

Law is two layers, like a constitution and the local/domain law under it:

- The constitution (universal): block depth, path depth, comments, single word.
  Every language obeys it; it lives in kernel.
- Per-language law (local/domain): a language's own hints, edge cases, and the
  alignment that makes the constitution land correctly on that language. It must
  align to the constitution, never contradict it.

Language-specific calibration is per-language law, not constitution: what counts as
a scope for the block-depth law in this language (e.g. whether `unsafe`/`async`
blocks or match arm blocks add depth), lifetime-vs-char-literal lexing, the
literal-vs-pattern distinction, and language-native checks (e.g. "no unwrap outside
tests"). These do not belong hardcoded in kernel. The per-language layer is a growth
point: it materializes when the first real local law is needed; until then kernel
carries only the universal constitution and languages contribute grammar + spec
data.

## The tsx dialect

`.tsx` routes through the TypeScript grammar extended with JSX rules; the
extension is per-language law, and the kernel never learns React exists.

- Every JSX element — paired, self-closing, or fragment — is one `markup` node;
  children nest. Markup depth is a separate axis from scope depth, judged by the
  markup law at its own limit. Brace containers re-enter normal parsing: arrow
  bodies inside them parse as scopes, so the block law resumes on the outer
  scope count and JSX layers add zero block depth.
- The `use` prefix is the ecosystem's namespace dialect for hooks, so the
  adapter transcribes it: for any declared word matching `use[A-Z]...`, the
  kernel judges the atom after `use`. `useCart` is judged as `Cart` and passes;
  `useUserCart` is judged as `UserCart` and is still compound, still debt.
  PascalCase component names get no exemption: their compoundness is judged
  normally, and the structural pressure toward `user/Card.tsx` is intended.
- Style syntax is marked for the grant law: a `style=` attribute name, a
  `<style>` element tag, an import whose module string ends in `.scss` or
  `.css`, and a `.scss` file itself (one `style` node spanning the whole file,
  with no inner parsing).
- SCSS is intentionally only a whole-file style marker today; selectors,
  declarations, nesting, and comments inside it are not structurally parsed.
- Markdown exposes heading scopes only. Inline syntax, lists, code fences, and
  prose comments do not become shared structure nodes.
- Honest limitations: closing tag names are not checked against opening ones
  (the PEG holds no backreference), so a mismatched closer still parses as one
  element. Generic type arguments in expression position (`f<T>(x)`) are
  disambiguated from JSX by backtracking, which relies on closers staying
  balanced; an ill-formed file with a stray closer can absorb neighbours into
  one bogus element before coverage catches the rest.

## Extension point, not an ABI

The adapter is an internal trait, resolved at compile time. It is not a stable
cross-crate ABI and carries no dynamic-loading surface. Adding `rust`, `md`, or
`ts` is the same shape of change: a grammar resource, a namespace-atom spec, and a
kind map.
