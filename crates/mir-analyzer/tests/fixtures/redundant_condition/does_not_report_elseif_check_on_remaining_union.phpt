===source===
<?php
// The elseif condition is NOT redundant when additional union members remain
// after the if condition narrows the type. Here $x is string|int|null: the if
// handles null, leaving string|int in the elseif — so is_string() is still
// meaningful (it distinguishes string from int).
function foo(string|int|null $x): void {
    if ($x === null) {
        // $x is null
    } elseif (is_string($x)) {
        // $x could still be int here; is_string() is not redundant
    }
}
===expect===
