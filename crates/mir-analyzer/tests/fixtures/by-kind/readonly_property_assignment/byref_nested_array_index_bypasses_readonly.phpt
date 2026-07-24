===description===
`sort($f->counts['x']['y'])` mutates a readonly array property's contents
through a TWO-level array-index-into-property by-ref argument, but
`check_byref_arg_purity`'s `ArrayAccess` arm only ever unwrapped one level
before checking the base, silently skipping deeper nesting.
===config===
suppress=MissingConstructor,MixedArgument,MixedArrayAccess
===file===
<?php
class Frozen {
    public readonly array $counts;
}

function tick(Frozen $f): void {
    sort($f->counts['x']['y']);
}
===expect===
ReadonlyPropertyAssignment@7:9-7:29: Cannot assign to readonly property Frozen::$counts outside of constructor
