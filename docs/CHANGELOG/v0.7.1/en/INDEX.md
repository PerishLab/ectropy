# Ectropy v0.7.1

A Svelte attribute name may now carry a directive prefix and a custom property
name together. `style:--ruled={value}` and the same shape on any directive were
previously terrain the scan could not read, so the `coverage` law refused the
file, correctly, because a clean result cannot cover what was never parsed.

An attribute name allowed exactly one separator before each identifier. A
separator may repeat now, so directive prefixes, data attributes, event
modifiers and custom property names all pass through one rule.

The law catalog and the configuration schema are unchanged.
