===description===
`sort($f->counts['x']['y'])` mutates a readonly array property's contents
through a TWO-level array-index-into-property by-ref argument, but
`check_byref_arg_purity`'s `ArrayAccess` arm only ever unwrapped one level
before checking the base, silently skipping deeper nesting.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Frozen {
    public readonly array $counts;
}

function tick(Frozen $f): void {
    sort($f->counts['x']['y']);
//       ^^^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Frozen::$counts outside of constructor
}
