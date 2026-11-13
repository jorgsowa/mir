===description===
An object offset on a `mixed` container isn't invalid: it may be a `WeakMap` or other
`ArrayAccess` object.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Driver {}
final class Stream {}

function stream(Driver $d): Stream {
    static $map;
    $map ??= new \WeakMap();
    return $map[$d] ??= new Stream();
}

/** @param array<string, int> $a */
function onArray(array $a, Driver $d): int {
    return $a[$d];
//            ^^ InvalidArrayOffset: Array offset expects 'array-key', got 'Driver'
}
