===description===
`is_countable()` true branch drops a final class that cannot be Countable.
===file===
<?php
final class Box {}

/** @param Box|array<int,int> $x */
function f($x): void {
    if (is_countable($x)) {
        /** @mir-check $x is array<int, int> */
        echo count($x);
    }
}
