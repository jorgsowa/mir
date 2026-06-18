===description===
Truthy check on string narrows to non-empty-string; falsy branch is ''|'0'.
A nullable string truthy check additionally removes null in the true branch.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param string $x */
function test_string(string $x): void {
    if ($x) {
        /** @mir-check $x is non-empty-string */
        $_ = $x;
    } else {
        /** @mir-check $x is ''|'0' */
        $_ = $x;
    }
}

/** @param string|null $y */
function test_nullable_string(string|null $y): void {
    if ($y) {
        /** @mir-check $y is non-empty-string */
        $_ = $y;
    } else {
        /** @mir-check $y is ''|'0'|null */
        $_ = $y;
    }
}
===expect===
