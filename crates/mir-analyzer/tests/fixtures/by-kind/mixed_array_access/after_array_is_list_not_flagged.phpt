===description===
MixedArrayAccess does NOT fire after array_is_list() because that check narrows mixed to list<mixed>, removing the mixed atom.
===config===
suppress=UnusedVariable,MixedArgument,MixedAssignment
===file===
<?php
function foo(mixed $a): void {
    if (array_is_list($a)) {
        $v = $a[0];
    }
}
===expect===
