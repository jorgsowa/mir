===description===
InvalidNamedArguments does NOT fire when passing positional arguments to a @no-named-arguments function.
===file===
<?php
/**
 * @no-named-arguments
 */
function sum(int ...$values): int {
    return array_sum($values);
}

sum(1, 2, 3);
===expect===
