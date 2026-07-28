# Structure JSON

`grammar` parses a source into a concrete tree; `kernel` lifts that into a uniform
structure tree that every rule and every language adapter reads. The structure
tree is the one contract between the parser substrate and the checks. Rules never
touch lexer or parser internals.

## Node

A node is language-agnostic:

- `kind` — the structural role, drawn from a shared kind vocabulary (`root`,
  `block`, `word`, `path`, ...), not a language-specific token name.
- `depth` — block depth from the root, the value the block law reads.
- `start` and `end` — byte range in the source, for reporting.
- `kids` — child nodes.

## Kind vocabulary

The kind set is small and shared across languages. A language adapter maps its
native constructs onto these kinds; it does not invent per-language kinds. Adding
a kind is a contract change, reviewed against `principles.md`.

- `root` — the whole source.
- `scope` — a real nesting scope (a function body, a control-flow block). Only
  `scope` counts toward the block-depth law.
- `item` — a named declaration (a function, a type, a module).
- `literal` — a value form, including struct literals. A struct literal brace is
  a `literal`, not a `scope`, so it does not inflate block depth. The shadow law
  reads its top-level fields without changing the tree contract.
- `comment` — explanation text; denied by default.
- `word` — a name, split into vocabulary atoms for the single-word law.
- `test` — a test-marker construct (`#[test]`, `#[cfg(test)]`, `Deno.test`).
  The grant law confines it to declared test territory; `cfg(not(test))` is
  production gating and stays unmarked.
- `probe` — the subject of a literal-equality branch (`if x == LIT`). The
  dispatch law reads consecutive probe runs; a repeated subject is debt.
- `receiver` — the first parameter of a free function (`name: Type`, self
  excluded). The receiver law groups them per file; a fourth function on one
  receiver is debt. Lifetimes and whitespace are erased before grouping.
- `param` — each parameter after the first (`name: Type`). The param law
  counts receiver plus params per function; more than four is debt. Pattern
  parameters stay unmarked, so the count only ever understates.
  The burr law reads the bound name on both receiver and param nodes; a named
  underscore binding is debt while the lone discard `_` stays unmarked.
- `markup` — one markup element (a JSX element, self-closing tag, or fragment).
  Markup depth is its own axis: a markup node's `depth` counts markup ancestors,
  and only markup counts toward the markup law. Code scopes inside markup stay
  on the block law's axis and continue the outer scope count.
- `style` — styling syntax: a `style=`, `class=`, or `className=` attribute
  name, a `<style>` element tag, the module string of a `.scss`/`.css` import,
  or a whole stylesheet file. Grant and ban laws assign its territory.
- `environment` — a direct process-environment access. The grant law denies it
  outside explicitly declared territory.

## Stability

The structure tree is the expensive-to-change surface. Rules, adapters, and the
cli all depend on it, so its shape is settled before breadth of grammar coverage
grows. See `adapters.md` for how a language plugs in.
