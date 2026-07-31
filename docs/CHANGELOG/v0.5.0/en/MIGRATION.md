# Migrating to Ectropy v0.5.0

## Name oversized combination models

Run Ectropy after upgrading. Existing `ectropy.toml` files need no schema edit:
`[limit] combination` defaults to three when omitted. Repositories may record
the value explicitly when they keep a complete local limit ledger.

Resolve each new `combination` finding by naming the missing model. Temporal
facts become an explicit state and transition; orthogonal facts become a
classification, pattern, decision table, or domain type. Moving the unchanged
Boolean expression into a helper does not resolve the finding.

Repositories scanning broad source sets should also inspect new findings from
plain `.css` files, which are now recognized alongside `.scss` files.

## Remove the retired option

Replace `ectropy --strict <root>` with `ectropy <root>`. The option was inert in
v0.4 and is invalid in v0.5. `--debt` remains invalid.

No stored-data or managed-skill migration is required. Upgrade the managed
skill with the stable binary after promotion, or stage the exact beta skill in
an isolated path while evaluating the candidate.
