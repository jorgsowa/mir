===description===
Returning a @param-typed mixed value from a function with a concrete declared return type fires MixedReturnStatement
===file===
<?php
/**
 * @param mixed $x
 */
function pass($x): int {
    return $x;
}
===expect===
MixedReturnStatement@6:4-6:14: Cannot return a mixed type from function with declared return type 'int'
