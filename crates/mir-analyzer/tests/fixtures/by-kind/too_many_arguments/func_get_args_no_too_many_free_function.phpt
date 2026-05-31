===description===
Free function using func_get_args() accepts extra positional args without TooManyArguments
===file===
<?php
function joinAll(string $separator) {
    $parts = func_get_args();
    array_shift($parts); // remove the separator
    return implode($separator, $parts);
}

// All of these should be accepted — extra args consumed by func_get_args().
joinAll(', ', 'a', 'b', 'c');
joinAll('-', 'x');
===expect===
