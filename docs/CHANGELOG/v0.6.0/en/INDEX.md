# Ectropy v0.6.0

## The laws have a catalog

The sixteen laws lived as string literals scattered across their own decision
points, with a second hand-written list inside boundary validation. The two had
drifted. They now collapse into one catalog carrying each law's name, what it
refuses, and whether it admits a boundary exemption; boundary validation derives
from that catalog.

The new `ectropy law` prints the whole table and `ectropy law <name>` reads one
entry. Query the catalog instead of copying the list.

## Three laws admit no boundary exemption

`coverage`, `grant`, and `ban` now refuse to appear in `[[boundary]] allow`.

`coverage` refuses because a clean result cannot cover terrain nobody parsed.
`grant` and `ban` refuse because their own `paths` already carry the territory:
widen the grant, or narrow the ban, rather than reaching for a second path
mechanism.

This corrects a silent failure. `allow = ["grant"]` and `allow = ["coverage"]`
were accepted by configuration validation while the decision points never
consulted the exemption, so the findings still fired — the declaration existed
and the behavior did not. `allow = ["ban"]` reported a generic unknown law.

The two errors are now separate: an unknown name still reports `unknown boundary
law`, while a known but sealed law reports `boundary law X admits no exemption`
and states why it is sealed.

## The written law caught up

`docs/principles.md` said this of bans only; it now says it of all three, and
restates grants as widened rather than exempted. `docs/stable.md` adds
`ectropy law` to the promised surface.

## The operating brief points instead of copying

The skill's SKILL.md no longer reproduces the enumeration of sixteen laws and
points at `ectropy law` instead, removing one more copy that could drift from
the binary.
