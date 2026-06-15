===description===
!isset($x) || RHS — if-body must NOT see $x as definitely-assigned when $x was only possibly-assigned
===file===
<?php
/** @param bool $cond */
function test(bool $cond): void {
    if ($cond) {
        $x = "hello";
    }
    // $x is only possibly-assigned here
    if (!isset($x) || strlen($x) > 3) {
        echo $x;
    }
}
===expect===
PossiblyUndefinedVariable@9:13-9:15: Variable $x might not be defined
