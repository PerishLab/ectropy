# receiver

## Trigger

The `receiver` error reports four or more functions in one root taking the same binding as their first seat.

## Move

The shared seat is a receiver nobody declared. Read whether the functions need the value's state or only its identity. State-holding groups become a type constructed from the binding, with the functions as its methods; identity-only groups become one namespace, the functions as statics, so callers keep the names they already import. The fix removes the parameter — never rename it, and never thread it one level deeper. Prefer the namespace shape the codebase already uses over a new one.

## Evidence

The sealkit ops plane carried fifteen such groups: `Bucket` and `Project` took constructor state, while the command, address and both filesystem seats became static namespaces matching the shape `lib/std/fs.ts` already held. Folding a crowded file into a type costs one level of depth and a few lines — `publish.ts` crossed the file and block limits on the way and had to split into a directory before the fold would hold.

## EXIT

Remove this entry when the receiver remedy becomes mechanical or the receiver law retires.
