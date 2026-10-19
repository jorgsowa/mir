===description===
After `$r = &$a['k']` the base has been auto-vivified, so later reads of a
nullable base are not PossiblyNullArrayAccess; null-only and by-ref argument
bases are also legal write contexts.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function setIt(mixed &$x): void { $x = 1; }

function afterBind(?array $a): mixed {
    $r = &$a['k'];
    $r = 1;
    return $a['other'];
}

function nullOnly(): void {
    $a = null;
    $r = &$a['k'];
    $r = 1;
}

function byRefArgument(?array $a): void {
    setIt($a['k']);
    setIt($a['x']['y']);
}

function tracedType(?array $a): void {
    $r = &$a['k'];
    /** @trace $a */
    $a;
}
===expect===
Trace@24:4-24:7: Type of $a is array
