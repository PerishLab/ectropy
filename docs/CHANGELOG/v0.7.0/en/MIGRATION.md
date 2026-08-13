# Migrating to Ectropy v0.7.0

Repositories that already include `.svelte` files in their scan patterns will
now have those files inspected. Rename compound component files to one
vocabulary atom, remove comments, and place component style blocks only where
the repository's style grants or bans allow them.

No `ectropy.toml` schema migration is required. The TypeScript and TSX adapters
retain their existing behavior in this release.
