===description===
MixedArrayAccess does NOT fire after an instanceof check narrows mixed to a concrete object type.
===config===
suppress=UnusedVariable,MixedAssignment
===file===
<?php
function foo(mixed $a): void {
    if ($a instanceof ArrayAccess) {
        $v = $a[0];
    }
}
===expect===
