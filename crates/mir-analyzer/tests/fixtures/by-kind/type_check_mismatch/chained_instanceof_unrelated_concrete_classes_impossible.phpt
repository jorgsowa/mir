===description===
`$x instanceof A && $x instanceof B` for two unrelated CONCRETE classes
(no common interface) narrows to just B — PHP's single inheritance makes
"also an A" impossible, so the atom is dropped rather than intersected
===config===
suppress=UnusedParam
===file===
<?php
class A {}
class B {}

/** @param A|B $x */
function f($x): void {
    if ($x instanceof A && $x instanceof B) {
        /** @mir-check $x is B */
        echo get_class($x);
    }
}
===expect===
