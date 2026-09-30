===description===
@pure implies no side effects at all — mutating a by-reference parameter
writes caller-visible state through the reference, same as a global/
static-variable write, but had no purity check anywhere.
===file===
<?php
/** @pure */
function mutateByRef(int &$x): void {
    $x = 42;
//  ^^^^^^^ ImpureByRefAssignment: Assigning to by-reference parameter $x in a @pure function
}
===expect===
