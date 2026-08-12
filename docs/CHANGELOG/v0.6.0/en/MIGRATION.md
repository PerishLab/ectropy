# Migrating to Ectropy v0.6.0

## Check `[[boundary]] allow` for grant or coverage

This is the only breaking change in this release. Sweep every repository's
`ectropy.toml` before upgrading:

```sh
grep -n 'allow' ectropy.toml
```

If any `[[boundary]]` allows `"grant"` or `"coverage"`, v0.6.0 fails the whole
configuration load and exits 2:

```
ectropy: boundary law `grant` admits no exemption: reserved syntax outside granted paths; widen the grant itself
```

In v0.5 and earlier those two names were accepted and did nothing at all — the
findings fired regardless. Removing them therefore changes no scan result; it
only makes the configuration loadable again. What was actually wanted is one of:

- To widen a `grant`: edit `[[grant]] paths` so the territory is declared.
- To narrow a `ban`: edit `[[ban]] paths`.
- To exempt `coverage`: there is no such route. Teach the parser that syntax, or
  drop the file from `[scan] include`. Claiming clean requires having read it.

`allow = ["ban"]` already failed, only with a generic unknown-law message; the
refusal now states its reason. The other thirteen laws are unchanged.

## Stop copying the law list

If a repository document or agent brief reproduces the enumeration of sixteen
laws, point it at `ectropy law` instead. The catalog is the single truth, and a
copy drifts — that drift is what this release repairs.

## Managed skill

Upgrade a managed skill with the stable binary once stable is activated. To
evaluate the candidate, stage the exact beta skill into an isolated path rather
than replacing a managed seat. No stored data requires migration.
