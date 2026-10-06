===description===
`is_countable()` narrows a non-final class to `Box&Countable`, which `count()` accepts alongside the array atom.
===file===
<?php
class Box {}

/** @param Box|array<int,int> $x */
function f($x): void {
    if (is_countable($x)) {
        echo count($x);
    }
}

/** @param array<int,int>|(Countable&Stringable) $x */
function g($x): void {
    echo count($x);
}
