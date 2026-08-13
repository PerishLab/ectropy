# Ectropy v0.7.0

Ectropy now admits `.svelte` files as first-class source terrain. The embedded
grammar maps TypeScript scripts, reactive statements, runes, template
expressions, control blocks, snippets, elements, comments, and style blocks
into the existing shared structure tree without adding a framework kind.

Svelte component filenames are path vocabulary atoms. A compound component
name such as `UserCard.svelte` is therefore reported by the same `word` law as
a compound Rust, TypeScript, or TSX filename.

The law catalog and configuration schema are unchanged. Grammar breadth has
widened; unsupported admitted syntax still produces `coverage` rather than a
false clean result.
