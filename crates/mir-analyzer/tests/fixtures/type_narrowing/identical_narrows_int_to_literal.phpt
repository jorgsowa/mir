===description===
=== narrows broad int/string to the specific literal in the true branch
===file===
<?php
/** @return 1 */
function narrowToOne(int $x): int {
    if ($x === 1) {
        return $x;
    }
    return 1;
}

/** @return "foo" */
function narrowToFoo(string $x): string {
    if ($x === "foo") {
        return $x;
    }
    return "foo";
}
===expect===

