# fanout

## Trigger

The `fanout` law fires on a Rust integration-test directory.

## Move

Group related cases behind explicit `[[test]]` entries whose drivers are per-group `main.rs` files. Put shared fixtures in a `world.rs` each group imports; do not trade directory width for hidden module depth.

## Evidence

The keel v0.5.0 test regroup reduced discovery width while keeping shared fixtures visible to every explicit test target.

## EXIT

Remove this entry when plumb owns the integration-test grouping shape and its fanout check.
