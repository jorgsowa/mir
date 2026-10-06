===description===
InvalidNamedArguments fires when a named argument is passed to a @no-named-arguments function.
===file===
<?php
/**
 * @no-named-arguments
 */
function sum(int ...$values): int {
    return array_sum($values);
}

sum(a: 1);
//  ^^^^ InvalidNamedArguments: sum() does not accept named arguments
